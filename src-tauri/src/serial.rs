use std::io::{ErrorKind, Read, Write};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use serialport::{available_ports, SerialPortType};
use tauri::{AppHandle, Emitter};

use crate::protocol::{HostToPico, PicoToHost};

pub fn start_serial_thread(app: AppHandle, rx: mpsc::Receiver<HostToPico>) {
    let mut current_port: Option<Box<dyn serialport::SerialPort>> = None;
    let mut current_port_name: Option<String> = None;
    let mut accumulator: Vec<u8> = Vec::new();
    let mut serial_buf = [0u8; 1024];

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

                        println!("Serial service connected on {}", port_name);
                        let _ = app.emit("pico-connection", true);

                        current_port = Some(port);
                        current_port_name = Some(port_name);
                        accumulator.clear();
                    }
                    Err(e) => {
                        println!("Failed to open port {}: {}", port_name, e);
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
            if let Ok(cmd) = rx.try_recv() {
                let mut buf = [0u8; 64];
                if let Ok(slice) = postcard::to_slice(&cmd, &mut buf) {
                    if let Err(e) = port.write_all(slice).and_then(|_| port.flush()) {
                        println!("Send error: {}", e);
                        let _ = app.emit("pico-connection", false);
                        current_port = None;
                        current_port_name = None;
                        accumulator.clear();
                        thread::sleep(Duration::from_millis(500));
                        continue;
                    }
                }
            }

            match port.read(&mut serial_buf) {
                Ok(bytes_read) if bytes_read > 0 => {
                    accumulator.extend_from_slice(&serial_buf[..bytes_read]);
                    loop {
                        match postcard::take_from_bytes::<PicoToHost>(&accumulator) {
                            Ok((msg, rest)) => {
                                if let Err(e) = app.emit("pico-event", msg) {
                                    println!("Frontend event error: {}", e);
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
                    println!("Connection to {} lost: {}", port_label, e);
                    let _ = app.emit("pico-connection", false);
                    current_port = None;
                    current_port_name = None;
                    accumulator.clear();
                    thread::sleep(Duration::from_millis(500));
                    continue;
                }
            }
        }

        thread::sleep(Duration::from_millis(2));
    }
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
