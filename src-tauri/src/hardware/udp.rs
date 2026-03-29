use std::net::UdpSocket;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::time::Duration;
use crossbeam_channel::Sender;
use crate::protocol::{Frame, FrameParser, FRAME_SIZE};

const DEFAULT_LISTEN_PORT: u16 = 49152;
const DEFAULT_DEVICE_PORT: u16 = 49153;

/// UDP transport for ESP32 wireless devices.
pub struct UdpDevice {
    pub device_id: u8,
    pub device_addr: String,
    pub listen_port: u16,
    pub device_port: u16,
    running: Arc<AtomicBool>,
    socket: Option<Arc<UdpSocket>>,
}

impl UdpDevice {
    pub fn new(device_id: u8, device_addr: String) -> Self {
        Self {
            device_id,
            device_addr,
            listen_port: DEFAULT_LISTEN_PORT,
            device_port: DEFAULT_DEVICE_PORT,
            running: Arc::new(AtomicBool::new(false)),
            socket: None,
        }
    }

    /// Send a frame to the ESP32 device.
    pub fn send_frame(&self, frame: Frame) -> Result<(), String> {
        if let Some(ref socket) = self.socket {
            let addr = format!("{}:{}", self.device_addr, self.device_port);
            socket.send_to(&frame.to_bytes(), &addr)
                .map_err(|e| e.to_string())?;
            Ok(())
        } else {
            Err("UDP socket not open".into())
        }
    }

    /// Start UDP listener thread. Incoming frames arrive via `input_tx`.
    pub fn start(&mut self, input_tx: Sender<Frame>) -> Result<std::thread::JoinHandle<()>, String> {
        let bind_addr = format!("0.0.0.0:{}", self.listen_port);
        let socket = UdpSocket::bind(&bind_addr)
            .map_err(|e| format!("Failed to bind UDP {}: {}", bind_addr, e))?;
        socket.set_read_timeout(Some(Duration::from_millis(10)))
            .map_err(|e| e.to_string())?;

        let socket = Arc::new(socket);
        self.socket = Some(socket.clone());
        let running = self.running.clone();

        running.store(true, Ordering::Relaxed);

        Ok(std::thread::spawn(move || {
            let mut parser = FrameParser::new();
            let mut buf = [0u8; 128];

            log::info!("UDP listener started on {}", bind_addr);

            while running.load(Ordering::Relaxed) {
                match socket.recv_from(&mut buf) {
                    Ok((n, _addr)) => {
                        let frames = parser.feed(&buf[..n]);
                        for frame in frames {
                            let _ = input_tx.send(frame);
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut => {}
                    Err(e) => {
                        log::error!("UDP recv error: {}", e);
                    }
                }
            }

            log::info!("UDP listener stopped");
        }))
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }
}
