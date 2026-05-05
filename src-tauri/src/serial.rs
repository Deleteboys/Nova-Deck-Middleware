use std::io::{ErrorKind, Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::TryRecvError;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;
use log::{error, info};
use crate::action::manager::ActionManager;
use crate::action::tracker::InputTracker;
use crate::commands::send_to_pico;
use crate::protocol::{HostToPico, PicoToHost, VibrationPattern};
use serialport::{available_ports, SerialPortType};
use tauri::{AppHandle, Emitter};

pub fn start_serial_thread(
    app: AppHandle,
    rx: mpsc::Receiver<HostToPico>,
    action_manager: Arc<Mutex<ActionManager>>,
    is_device_connected: Arc<AtomicBool>,
) {
    let mut current_port: Option<Box<dyn serialport::SerialPort>> = None;
    let mut current_port_name: Option<String> = None;
    let mut accumulator: Vec<u8> = Vec::new();
    let mut serial_buf = [0u8; 1024];

    let mut tracker = InputTracker::new();

    loop {
        if current_port.is_none() {
            if let Some(port_name) = find_pico_port() {
                match serialport::new(&port_name, 115200)
                    .timeout(Duration::from_millis(10))
                    .open()
                {
                    Ok(mut port) => {
                        port.write_data_terminal_ready(true).ok();
                        port.write_request_to_send(true).ok();
                        thread::sleep(Duration::from_millis(500));
                        port.clear(serialport::ClearBuffer::All).ok();

                        info!("Serial service connected on {}", port_name);
                        is_device_connected.store(true, Ordering::Relaxed);
                        let _ = app.emit("pico-connection", true);

                        let mut buf = [0u8; 64];
                        if let Ok(slice) = postcard::to_slice(&HostToPico::GetConfig, &mut buf) {
                            if let Err(e) = port.write_all(slice).and_then(|_| port.flush()) {
                                error!("Initial config request failed: {}", e);
                            }
                        }
                        if let Ok(slice) = postcard::to_slice(&HostToPico::GetVersion, &mut buf) {
                            let _ = port.write_all(slice).and_then(|_| port.flush());
                        }

                        current_port = Some(port);
                        current_port_name = Some(port_name);
                        accumulator.clear();
                    }
                    Err(e) => {
                        error!("Failed to open port {}: {}", port_name, e);
                        thread::sleep(Duration::from_millis(1000));
                        continue;
                    }
                }
            } else {
                thread::sleep(Duration::from_millis(500));
                continue;
            }
        }

        if let Some(port) = current_port.as_mut() {
            match rx.try_recv() {
                Ok(cmd) => {
                    let mut buf = [0u8; 64];
                    if let Ok(slice) = postcard::to_slice(&cmd, &mut buf) {
                        if let Err(e) = port.write_all(slice).and_then(|_| port.flush()) {
                            error!("Send error: {}", e);
                            is_device_connected.store(false, Ordering::Relaxed);
                            let _ = app.emit("pico-connection", false);
                            current_port = None;
                            current_port_name = None;
                            accumulator.clear();
                            thread::sleep(Duration::from_millis(500));
                            continue;
                        }
                    }
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    error!("Serial service stopping: command channel closed");
                    break;
                }
            }

            match port.read(&mut serial_buf) {
                Ok(bytes_read) if bytes_read > 0 => {
                    accumulator.extend_from_slice(&serial_buf[..bytes_read]);
                    loop {
                        match postcard::take_from_bytes::<PicoToHost>(&accumulator) {
                            Ok((msg, rest)) => {
                                let _ = app.emit("pico-event", msg.clone());

                                if let PicoToHost::Version { version } = &msg {
                                    let _ = app.emit("pico-version", version.as_str());
                                }

                                if let PicoToHost::ButtonChanged { id, .. } = &msg {
                                    if let Ok(manager) = action_manager.lock() {
                                        let expects_double_click = manager.has_double_press(*id);
                                        tracker.set_double_click_enabled(*id, expects_double_click);
                                    }
                                }

                                if let Some(logical_trigger) = tracker.process_event(msg) {
                                    if let Ok(manager) = action_manager.lock() {
                                        manager.handle_trigger(logical_trigger);
                                    }
                                }

                                accumulator = rest.to_vec();
                            }
                            Err(postcard::Error::DeserializeUnexpectedEnd) => break,
                            Err(_) => {
                                if !accumulator.is_empty() {
                                    accumulator.remove(0);
                                } else {
                                    break;
                                }
                            }
                        }
                    }
                }
                Ok(_) => {}
                Err(e) if e.kind() == ErrorKind::TimedOut || e.kind() == ErrorKind::WouldBlock => {}
                Err(e) => {
                    let port_label = current_port_name.as_deref().unwrap_or("unknown");
                    info!("Connection to {} lost: {}", port_label, e);
                    is_device_connected.store(false, Ordering::Relaxed);
                    let _ = app.emit("pico-connection", false);
                    current_port = None;
                    current_port_name = None;
                    accumulator.clear();
                    thread::sleep(Duration::from_millis(500));
                    continue;
                }
            }
            if let Some(ready_id) = tracker.check_long_press_feedback() {
                if let Err(e) = write_to_pico(port, &HostToPico::Vibrate { pattern: VibrationPattern::Medium }) {
                    error!("Fehler beim Senden des Feedbacks: {}", e);
                }
            }
        }

        if let Some(logical_trigger) = tracker.update() {
            if let Ok(manager) = action_manager.lock() {
                manager.handle_trigger(logical_trigger);
            }
        }

        thread::sleep(Duration::from_millis(2));
    }

    is_device_connected.store(false, Ordering::Relaxed);
}

fn write_to_pico(
    port: &mut Box<dyn serialport::SerialPort>,
    cmd: &HostToPico,
) -> Result<(), std::io::Error> {
    let mut buf = [0u8; 64];

    // Serialisierung
    let slice = postcard::to_slice(cmd, &mut buf)
        .map_err(|e| std::io::Error::new(ErrorKind::InvalidData, e))?;

    // Schreiben & Flushen
    port.write_all(slice)?;
    port.flush()?;

    Ok(())
}

fn find_pico_port() -> Option<String> {
    available_ports()
        .unwrap_or_default()
        .into_iter()
        .find(|p| {
            if let SerialPortType::UsbPort(info) = &p.port_type {
                return info.vid == 0xc0de && info.pid == 0xcafe;
            }
            false
        })
        .map(|p| p.port_name)
}
