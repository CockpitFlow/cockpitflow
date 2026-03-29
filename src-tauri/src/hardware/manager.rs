use std::collections::HashMap;
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::time::{Duration, Instant};
use crossbeam_channel::{Sender, Receiver, bounded};
use crate::config::{DeviceConfig, TransportConfig};
use crate::protocol::{Frame, HEADER_HEARTBEAT};
use crate::simconnect::{PinMapping, PinType, SimConnectClient, SimEvent, process_input};
use super::serial::SerialDevice;

/// Per-device runtime status, reported to the frontend.
#[derive(Debug, Clone, serde::Serialize)]
pub struct DeviceStatus {
    pub device_id: u8,
    pub name: String,
    pub port: String,
    pub connected: bool,
    pub last_heartbeat_ms: Option<u64>,
    pub frames_per_sec: f64,
    pub error: Option<String>,
}

/// Internal bookkeeping for a single active device connection.
struct ActiveDevice {
    config: DeviceConfig,
    serial: SerialDevice,
    thread_handle: Option<std::thread::JoinHandle<()>>,
    connected: bool,
    last_heartbeat: Option<Instant>,
    frame_count: u64,
    frame_count_snapshot: u64,
    fps_last_measured: Instant,
    frames_per_sec: f64,
    last_error: Option<String>,
}

/// Central device manager — owns all device connections and routes frames.
pub struct DeviceManager {
    /// Active device connections, keyed by device_id
    devices: HashMap<u8, ActiveDevice>,
    /// All registered pin mappings, keyed by (device_id, pin)
    mappings: HashMap<(u8, u8), PinMapping>,
    /// Channel receiving frames from ALL devices
    input_rx: Receiver<Frame>,
    /// Shared sender clone handed to each device thread
    input_tx: Sender<Frame>,
    /// Master running flag for the reconnect loop
    running: Arc<AtomicBool>,
    /// Handle to the background reconnect thread
    reconnect_handle: Option<std::thread::JoinHandle<()>>,
    /// Configurable interval between reconnection attempts (default 5s)
    reconnect_interval: Duration,
    /// Timestamp of the last reconnection attempt
    last_reconnect_attempt: Option<Instant>,
}

impl DeviceManager {
    pub fn new() -> Self {
        let (input_tx, input_rx) = bounded(1024);
        Self {
            devices: HashMap::new(),
            mappings: HashMap::new(),
            input_rx,
            input_tx,
            running: Arc::new(AtomicBool::new(false)),
            reconnect_handle: None,
            reconnect_interval: Duration::from_secs(5),
            last_reconnect_attempt: None,
        }
    }

    /// Set the interval between automatic reconnection attempts.
    pub fn set_reconnect_interval(&mut self, interval: Duration) {
        self.reconnect_interval = interval;
    }

    /// Get the sender that devices should use to push incoming frames.
    pub fn input_sender(&self) -> Sender<Frame> {
        self.input_tx.clone()
    }

    // ─── Mapping management (preserved from previous HardwareManager) ───

    pub fn add_mapping(&mut self, mapping: PinMapping) {
        self.mappings.insert((mapping.device_id, mapping.pin), mapping);
    }

    pub fn mapping_count(&self) -> usize { self.mappings.len() }

    pub fn remove_mapping(&mut self, device_id: u8, pin: u8) {
        self.mappings.remove(&(device_id, pin));
    }

    pub fn get_mappings(&self) -> Vec<PinMapping> {
        self.mappings.values().cloned().collect()
    }

    pub fn set_mappings(&mut self, mappings: Vec<PinMapping>) {
        self.mappings.clear();
        for m in mappings {
            self.mappings.insert((m.device_id, m.pin), m);
        }
    }

    // ─── Device connection management ───

    /// Connect a device by its config. Spawns a serial I/O thread.
    pub fn connect(&mut self, config: &DeviceConfig) -> Result<(), String> {
        let device_id = config.id;

        // Disconnect existing connection for this device_id if any
        if self.devices.contains_key(&device_id) {
            self.disconnect(device_id);
        }

        let (port_name, baud_rate) = match &config.transport {
            TransportConfig::Serial { port, baud_rate } => (port.clone(), *baud_rate),
            TransportConfig::Udp { .. } => {
                return Err("UDP transport not yet supported by DeviceManager".into());
            }
        };

        let serial = SerialDevice::new(port_name.clone(), baud_rate, device_id);
        let handle = serial.start(self.input_tx.clone())?;

        let now = Instant::now();
        self.devices.insert(device_id, ActiveDevice {
            config: config.clone(),
            serial,
            thread_handle: Some(handle),
            connected: true,
            last_heartbeat: None,
            frame_count: 0,
            frame_count_snapshot: 0,
            fps_last_measured: now,
            frames_per_sec: 0.0,
            last_error: None,
        });

        log::info!("Connected device {} ({}) on {}", device_id, config.name, port_name);
        Ok(())
    }

    /// Disconnect a single device by id.
    pub fn disconnect(&mut self, device_id: u8) {
        if let Some(mut dev) = self.devices.remove(&device_id) {
            dev.serial.stop();
            if let Some(handle) = dev.thread_handle.take() {
                let _ = handle.join();
            }
            log::info!("Disconnected device {}", device_id);
        }
    }

    /// Disconnect all devices and stop the reconnect loop.
    pub fn shutdown(&mut self) {
        self.running.store(false, Ordering::Relaxed);

        let ids: Vec<u8> = self.devices.keys().copied().collect();
        for id in ids {
            self.disconnect(id);
        }

        if let Some(handle) = self.reconnect_handle.take() {
            let _ = handle.join();
        }
    }

    /// Returns true if a device with this id is tracked and its thread is alive.
    pub fn is_connected(&self, device_id: u8) -> bool {
        self.devices.get(&device_id).map_or(false, |d| d.connected)
    }

    /// Collect status snapshots for all tracked devices.
    pub fn status(&self) -> Vec<DeviceStatus> {
        self.devices.values().map(|dev| {
            let port = match &dev.config.transport {
                TransportConfig::Serial { port, .. } => port.clone(),
                TransportConfig::Udp { address, .. } => address.clone(),
            };
            DeviceStatus {
                device_id: dev.config.id,
                name: dev.config.name.clone(),
                port,
                connected: dev.connected,
                last_heartbeat_ms: dev.last_heartbeat.map(|t| t.elapsed().as_millis() as u64),
                frames_per_sec: dev.frames_per_sec,
                error: dev.last_error.clone(),
            }
        }).collect()
    }

    /// Process all pending input frames:
    /// - Update per-device heartbeat/frame counters
    /// - Route mapped frames to SimConnect
    /// Call this in the main event loop (~30-60Hz).
    pub fn poll(&mut self, sim_client: Option<&SimConnectClient>) {
        let events = self.poll_and_collect_events();
        if let Some(client) = sim_client {
            for event in events {
                if let Err(e) = client.send_event(event) {
                    log::error!("Failed to send sim event: {}", e);
                }
            }
        }
    }

    /// Poll frames and return collected SimEvents (for external routing to X-Plane etc.)
    pub fn poll_and_collect_events(&mut self) -> Vec<SimEvent> {
        let now = Instant::now();
        let mut events = Vec::new();

        while let Ok(frame) = self.input_rx.try_recv() {
            let device_id = frame.device_id;

            // Update per-device stats
            if let Some(dev) = self.devices.get_mut(&device_id) {
                dev.frame_count += 1;
                if frame.header == HEADER_HEARTBEAT {
                    dev.last_heartbeat = Some(now);
                }
            }

            // Route if mapped
            let key = (frame.device_id, frame.pin);
            if let Some(mapping) = self.mappings.get(&key) {
                // Only trigger on press (value=1), not release
                if frame.header == 0xAA && (matches!(mapping.pin_type, PinType::DigitalInput) && frame.value != 1) {
                    continue;
                }
                if let Some(event) = process_input(mapping, frame.value) {
                    log::info!("Routing event: dev={} pin={} val={} -> {:?}", frame.device_id, frame.pin, frame.value, event);
                    events.push(event);
                }
            }
        }

        // Update fps counters
        for dev in self.devices.values_mut() {
            let elapsed = now.duration_since(dev.fps_last_measured).as_secs_f64();
            if elapsed >= 1.0 {
                let delta = dev.frame_count - dev.frame_count_snapshot;
                dev.frames_per_sec = delta as f64 / elapsed;
                dev.frame_count_snapshot = dev.frame_count;
                dev.fps_last_measured = now;
            }

            // Detect stale connections: no heartbeat in 5 seconds while supposedly connected
            if dev.connected {
                let stale = dev.last_heartbeat
                    .map_or(false, |hb| now.duration_since(hb) > Duration::from_secs(5));
                let thread_dead = dev.thread_handle.as_ref()
                    .map_or(true, |h| h.is_finished());

                if thread_dead {
                    dev.connected = false;
                    dev.last_error = Some("Serial thread exited".into());
                    log::warn!("Device {} thread exited unexpectedly", dev.config.id);
                } else if stale && dev.frame_count > 0 {
                    // Thread alive but no heartbeats — might be a cable issue
                    dev.last_error = Some("No heartbeat (>5s)".into());
                }
            }
        }

        events
    }

    /// Attempt to reconnect any devices whose threads have died.
    /// Respects `reconnect_interval` — skips if called too soon after the last attempt.
    /// Call periodically from the main event loop (e.g., every poll cycle).
    pub fn try_reconnect(&mut self) {
        let now = Instant::now();

        // Throttle reconnection attempts to the configured interval
        if let Some(last) = self.last_reconnect_attempt {
            if now.duration_since(last) < self.reconnect_interval {
                return;
            }
        }

        let stale_ids: Vec<(u8, DeviceConfig)> = self.devices.iter()
            .filter(|(_, dev)| !dev.connected)
            .map(|(id, dev)| (*id, dev.config.clone()))
            .collect();

        if stale_ids.is_empty() {
            return;
        }

        self.last_reconnect_attempt = Some(now);

        for (device_id, config) in stale_ids {
            let port_str = match &config.transport {
                TransportConfig::Serial { port, .. } => port.clone(),
                TransportConfig::Udp { address, .. } => address.clone(),
            };
            log::info!(
                "Reconnect attempt for device {} ({}) on {} (interval={}s)",
                device_id, config.name, port_str,
                self.reconnect_interval.as_secs()
            );

            // Remove the dead entry first
            if let Some(mut old) = self.devices.remove(&device_id) {
                old.serial.stop();
                if let Some(h) = old.thread_handle.take() {
                    let _ = h.join();
                }
            }
            // Try to re-establish
            match self.connect(&config) {
                Ok(()) => {
                    log::info!("Reconnect succeeded for device {} ({}) on {}", device_id, config.name, port_str);
                }
                Err(e) => {
                    log::warn!("Reconnect failed for device {} ({}): {} — will retry in {}s",
                        device_id, config.name, e, self.reconnect_interval.as_secs());
                    // Re-insert as disconnected so we try again later
                    let (port_name, baud_rate) = match &config.transport {
                        TransportConfig::Serial { port, baud_rate } => (port.clone(), *baud_rate),
                        TransportConfig::Udp { .. } => continue,
                    };
                    self.devices.insert(device_id, ActiveDevice {
                        config,
                        serial: SerialDevice::new(port_name, baud_rate, device_id),
                        thread_handle: None,
                        connected: false,
                        last_heartbeat: None,
                        frame_count: 0,
                        frame_count_snapshot: 0,
                        fps_last_measured: now,
                        frames_per_sec: 0.0,
                        last_error: Some(e),
                    });
                }
            }
        }
    }
}

impl Drop for DeviceManager {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// Thread-safe wrapper for DeviceManager, intended for storage in Tauri AppState.
pub struct SharedDeviceManager(pub Mutex<DeviceManager>);

impl SharedDeviceManager {
    pub fn new() -> Self {
        Self(Mutex::new(DeviceManager::new()))
    }
}
