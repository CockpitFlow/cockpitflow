use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::time::Duration;
use crossbeam_channel::{Sender, Receiver, bounded};
use crate::protocol::{Frame, FrameParser, FRAME_SIZE};

/// A serial device connection (Arduino over USB).
pub struct SerialDevice {
    pub port_name: String,
    pub baud_rate: u32,
    pub device_id: u8,
    running: Arc<AtomicBool>,
    output_tx: Sender<Frame>,
    output_rx: Receiver<Frame>,
}

impl SerialDevice {
    pub fn new(port_name: String, baud_rate: u32, device_id: u8) -> Self {
        let (output_tx, output_rx) = bounded(256);
        Self {
            port_name,
            baud_rate,
            device_id,
            running: Arc::new(AtomicBool::new(false)),
            output_tx,
            output_rx,
        }
    }

    /// Queue a frame to send to this device.
    pub fn send_frame(&self, frame: Frame) -> Result<(), String> {
        self.output_tx.send(frame).map_err(|e| e.to_string())
    }

    /// Start the serial I/O thread. Returns incoming frames via `input_tx`.
    pub fn start(&self, input_tx: Sender<Frame>) -> Result<std::thread::JoinHandle<()>, String> {
        let port = serialport::new(&self.port_name, self.baud_rate)
            .timeout(Duration::from_millis(10))
            .open()
            .map_err(|e| format!("Failed to open {}: {}", self.port_name, e))?;

        let running = self.running.clone();
        let output_rx = self.output_rx.clone();
        let port_name = self.port_name.clone();

        running.store(true, Ordering::Relaxed);

        Ok(std::thread::spawn(move || {
            let mut port = port;
            let mut parser = FrameParser::new();
            let mut read_buf = [0u8; 128];

            log::info!("Serial thread started for {}", port_name);

            while running.load(Ordering::Relaxed) {
                // Read incoming data
                match port.read(&mut read_buf) {
                    Ok(n) if n > 0 => {
                        log::trace!("{}: read {} bytes", port_name, n);
                        let frames = parser.feed(&read_buf[..n]);
                        for frame in frames {
                            log::debug!("{}: frame hdr={:#x} dev={} pin={} val={}", port_name, frame.header, frame.device_id, frame.pin, frame.value);
                            let _ = input_tx.send(frame);
                        }
                    }
                    Ok(_) => {}
                    Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {}
                    Err(e) => {
                        log::error!("Serial read error on {}: {}", port_name, e);
                        break;
                    }
                }

                // Write outgoing frames
                while let Ok(frame) = output_rx.try_recv() {
                    let bytes = frame.to_bytes();
                    if let Err(e) = port.write_all(&bytes) {
                        log::error!("Serial write error on {}: {}", port_name, e);
                    }
                }
            }

            log::info!("Serial thread stopped for {}", port_name);
        }))
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
}

/// List available serial ports with rich device info.
/// For flashable boards, also probes firmware for custom name/ID.
pub fn list_ports() -> Vec<SerialPortInfo> {
    serialport::available_ports()
        .unwrap_or_default()
        .into_iter()
        .map(|p| {
            match &p.port_type {
                serialport::SerialPortType::UsbPort(info) => {
                    let (kind, friendly) = identify_usb_device(info.vid, info.pid);
                    let flashable = is_flashable_board(info.vid, info.pid);

                    let product = info.product.clone()
                        .unwrap_or_else(|| friendly.1.into());

                    SerialPortInfo {
                        name: p.port_name,
                        port_type: "USB".into(),
                        vid: Some(format!("{:04X}", info.vid)),
                        pid: Some(format!("{:04X}", info.pid)),
                        manufacturer: info.manufacturer.clone().or_else(|| Some(friendly.0.into())),
                        product: Some(product),
                        serial_number: info.serial_number.clone(),
                        device_kind: kind.into(),
                        flashable,
                        firmware_id: None,
                    }
                }
                serialport::SerialPortType::BluetoothPort => SerialPortInfo {
                    name: p.port_name,
                    port_type: "Bluetooth".into(),
                    device_kind: "bluetooth".into(),
                    ..Default::default()
                },
                serialport::SerialPortType::PciPort => SerialPortInfo {
                    name: p.port_name,
                    port_type: "PCI".into(),
                    device_kind: "internal".into(),
                    ..Default::default()
                },
                _ => SerialPortInfo {
                    name: p.port_name,
                    port_type: "Unknown".into(),
                    device_kind: "unknown".into(),
                    ..Default::default()
                },
            }
        })
        .collect()
}

/// Identify a USB device by VID/PID. Returns (kind, (manufacturer, product)).
fn identify_usb_device(vid: u16, pid: u16) -> (&'static str, (&'static str, &'static str)) {
    match (vid, pid) {
        // ─── Espressif (ESP32 boards) ───
        (0x303A, _)                     => ("esp32",    ("Espressif", "ESP32 (USB native)")),
        (0x10C4, 0xEA60)               => ("esp32",    ("Silicon Labs", "CP2102 (ESP32)")),
        (0x1A86, 0x7523)               => ("arduino",  ("CH340", "Arduino/ESP32 (CH340) — select board in Setup")),
        (0x1A86, 0x55D4)               => ("arduino",  ("CH9102", "Arduino/ESP32 (CH9102) — select board in Setup")),

        // ─── Arduino ───
        (0x2341, 0x0043)               => ("arduino",  ("Arduino", "Arduino Uno")),
        (0x2341, 0x0042)               => ("arduino",  ("Arduino", "Arduino Mega 2560")),
        (0x2341, 0x0010)               => ("arduino",  ("Arduino", "Arduino Mega 2560")),
        (0x2341, 0x003D)               => ("arduino",  ("Arduino", "Arduino Due")),
        (0x2341, 0x8036)               => ("arduino",  ("Arduino", "Arduino Leonardo")),
        (0x2341, 0x0036)               => ("arduino",  ("Arduino", "Arduino Leonardo")),
        (0x2341, 0x8037)               => ("arduino",  ("Arduino", "Arduino Micro")),
        (0x2341, 0x0058)               => ("arduino",  ("Arduino", "Arduino Nano Every")),
        (0x2341, 0x0070)               => ("arduino",  ("Arduino", "Arduino Nano ESP32")),
        (0x2341, _)                     => ("arduino",  ("Arduino", "Arduino (Unknown Model)")),

        // ─── FTDI (common on clones & breakouts) ───
        (0x0403, 0x6001)               => ("arduino",  ("FTDI", "FT232R (Arduino/ESP)")),
        (0x0403, 0x6010)               => ("arduino",  ("FTDI", "FT2232 (Dual)")),
        (0x0403, 0x6015)               => ("arduino",  ("FTDI", "FT-X (Arduino)")),
        (0x0403, _)                     => ("serial",   ("FTDI", "FTDI Serial")),

        // ─── Audio / Headsets (NOT flashable) ───
        (0x0D8C, _)                     => ("audio",    ("C-Media", "USB Audio Device")),
        (0x08BB, _)                     => ("audio",    ("Texas Instruments", "USB Audio")),
        (0x1235, _)                     => ("audio",    ("Focusrite", "Audio Interface")),
        (0x1397, _)                     => ("audio",    ("BEHRINGER", "Audio Interface")),
        (0x046D, _)                     => ("peripheral", ("Logitech", "Peripheral")),
        (0x1532, _)                     => ("peripheral", ("Razer", "Peripheral")),

        // ─── Generic CH340 (could be Arduino or ESP32) ───
        (0x1A86, _)                     => ("arduino",  ("QinHeng", "CH340/CH341 Serial")),

        // ─── Unknown ───
        _                               => ("unknown",  ("Unknown", "USB Serial Device")),
    }
}

/// Check if a USB device is a flashable microcontroller board.
fn is_flashable_board(vid: u16, pid: u16) -> bool {
    matches!(identify_usb_device(vid, pid).0, "esp32" | "arduino")
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SerialPortInfo {
    pub name: String,
    pub port_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manufacturer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serial_number: Option<String>,
    /// "esp32", "arduino", "audio", "peripheral", "bluetooth", "unknown"
    pub device_kind: String,
    /// Whether this device can be flashed with firmware
    #[serde(default)]
    pub flashable: bool,
    /// Firmware-reported device ID (if CockpitFlow firmware is installed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub firmware_id: Option<u8>,
}

/// Public probe for Tauri command.
pub fn probe_firmware_info(port_name: &str) -> Result<crate::ProbeResult, String> {
    use std::io::{Read, Write};

    let mut port = serialport::new(port_name, 115200)
        .timeout(Duration::from_millis(800))
        .open()
        .map_err(|e| format!("Cannot open {}: {}", port_name, e))?;

    // Send info request
    let _ = port.write_all(b"\x01?\n");
    let _ = port.flush();

    std::thread::sleep(Duration::from_millis(300));

    let mut buf = [0u8; 512];
    let mut response = String::new();

    loop {
        match port.read(&mut buf) {
            Ok(n) if n > 0 => {
                response.push_str(&String::from_utf8_lossy(&buf[..n]));
                if response.contains('}') { break; }
            }
            _ => break,
        }
    }

    // Parse JSON response
    if let Some(start) = response.find('{') {
        if let Some(end) = response[start..].find('}') {
            let json_str = &response[start..=start + end];
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(json_str) {
                return Ok(crate::ProbeResult {
                    name: val.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()),
                    device_id: val.get("id").and_then(|i| i.as_u64()).map(|i| i as u8),
                    wifi_connected: val.get("wifi").and_then(|w| w.as_bool()).unwrap_or(false),
                    wifi_ip: val.get("ip").and_then(|i| i.as_str()).map(|s| s.to_string()),
                });
            }
        }
    }

    Ok(crate::ProbeResult { name: None, device_id: None, wifi_connected: false, wifi_ip: None })
}

/// Probe a serial port for CockpitFlow firmware.
/// Sends the `?` info command and parses the JSON response.
/// Returns (custom_name, device_id) if firmware responds.
fn probe_firmware(port_name: &str) -> Result<(Option<String>, Option<u8>), ()> {
    use std::io::{Read, Write};

    let mut port = serialport::new(port_name, 115200)
        .timeout(Duration::from_millis(800))
        .open()
        .map_err(|_| ())?;

    // Send info request: SOH + ? + newline
    let _ = port.write_all(b"\x01?\n");
    let _ = port.flush();

    // Wait for response
    std::thread::sleep(Duration::from_millis(300));

    let mut buf = [0u8; 512];
    let mut response = String::new();

    // Read available data
    loop {
        match port.read(&mut buf) {
            Ok(n) if n > 0 => {
                response.push_str(&String::from_utf8_lossy(&buf[..n]));
                if response.contains('}') { break; }
            }
            _ => break,
        }
    }

    // Parse JSON from response (firmware sends: {"id":1,"name":"My Device",...})
    if let Some(start) = response.find('{') {
        if let Some(end) = response[start..].find('}') {
            let json_str = &response[start..=start + end];
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(json_str) {
                let name = val.get("name").and_then(|n| n.as_str()).map(|s| s.to_string());
                let id = val.get("id").and_then(|i| i.as_u64()).map(|i| i as u8);

                // Build a friendly display name
                let display_name = match (&name, id) {
                    (Some(n), Some(i)) => Some(format!("{} (ID:{})", n, i)),
                    (Some(n), None) => Some(n.clone()),
                    (None, Some(i)) => Some(format!("CockpitFlow (ID:{})", i)),
                    _ => None,
                };

                return Ok((display_name, id));
            }
        }
    }

    // No valid JSON — check if we got heartbeat frames (0xCC = firmware running, just no config response)
    let raw = response.as_bytes();
    for i in 0..raw.len().saturating_sub(5) {
        if raw[i] == 0xCC {
            return Ok((Some("CockpitFlow Device".into()), Some(raw.get(i + 1).copied().unwrap_or(0))));
        }
    }

    Ok((None, None))
}
