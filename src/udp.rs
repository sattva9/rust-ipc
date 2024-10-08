use crate::{get_payload, ExecutionResult, KB};

use std::net::UdpSocket;
use std::process::{Child, Command};
use std::thread::sleep;
use std::time::{Duration, Instant};

const MAX_CHUNK_SIZE: usize = 8 * KB;

pub struct UdpStreamWrapper {
    pub our_port: u16,
    pub server: bool,
    pub socket: UdpSocket,
    pub data_size: usize,
}

impl UdpStreamWrapper {
    pub fn from_port(port: u16, data_size: usize) -> Self {
        let socket = UdpSocket::bind(format!("127.0.0.1:{}", port)).unwrap();
        let our_port = socket.local_addr().unwrap().port();
        socket
            .set_read_timeout(Some(Duration::from_millis(100)))
            .unwrap();

        Self {
            our_port,
            socket,
            server: false,
            data_size,
        }
    }

    pub fn new(data_size: usize) -> UdpStreamWrapper {
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        let our_port = socket.local_addr().unwrap().port();
        socket
            .set_read_timeout(Some(Duration::from_millis(100)))
            .unwrap();

        Self {
            our_port,
            server: true,
            socket,
            data_size,
        }
    }

    pub fn send(&self, data: &Vec<u8>) {
        for chunk in data.chunks(MAX_CHUNK_SIZE) {
            loop {
                match self.socket.send(chunk) {
                    Ok(_) => break,
                    Err(_) => continue,
                }
            }
        }
    }

    pub fn recv(&self) -> Vec<u8> {
        let mut received_data = Vec::new();
        loop {
            let mut buf = vec![0; MAX_CHUNK_SIZE];
            match self.socket.recv(&mut buf) {
                Ok(size) => {
                    received_data.extend_from_slice(&buf[..size]);
                    if received_data.len() >= self.data_size {
                        break;
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    if !received_data.is_empty() {
                        break;
                    }
                }
                Err(e) => panic!("Error receiving data {e}"),
            }
        }
        received_data
    }
}

pub struct UdpRunner {
    child_proc: Option<Child>,
    wrapper: UdpStreamWrapper,
    data_size: usize,
    request_data: Vec<u8>,
    #[allow(unused)]
    response_data: Vec<u8>,
}

impl UdpRunner {
    pub fn new(start_child: bool, data_size: usize) -> UdpRunner {
        let wrapper = UdpStreamWrapper::new(data_size);
        let their_port = portpicker::pick_unused_port().unwrap();
        let exe = crate::executable_path("udp_consumer");
        let child_proc = if start_child {
            Some(
                Command::new(exe)
                    .args(&[
                        wrapper.our_port.to_string(),
                        their_port.to_string(),
                        data_size.to_string(),
                    ])
                    .spawn()
                    .unwrap(),
            )
        } else {
            None
        };
        // Awkward sleep to make sure the child proc is ready
        sleep(Duration::from_secs(2));
        wrapper
            .socket
            .connect(format!("127.0.0.1:{}", their_port))
            .expect("Child process can't connect");

        let (request_data, response_data) = get_payload(data_size);

        Self {
            child_proc,
            wrapper,
            data_size,
            request_data,
            response_data,
        }
    }

    pub fn run(&mut self, n: usize, print: bool) {
        let start = Instant::now();
        for _ in 0..n {
            self.wrapper.send(&self.request_data);
            let _response = self.wrapper.recv();
            // if !response.eq(&self.response_data) {
            //     panic!("Sent request didn't get expected response")
            // }
        }
        if print {
            let elapsed = start.elapsed();
            let res = ExecutionResult::new(format!("UDP - {}KB", self.data_size / KB), elapsed, n);
            res.print_info();
        }
    }
}

impl Drop for UdpRunner {
    fn drop(&mut self) {
        if let Some(ref mut c) = self.child_proc {
            c.kill().unwrap();
        }
    }
}
