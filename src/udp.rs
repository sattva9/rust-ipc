use crate::ExecutionResult;

use std::net::UdpSocket;
use std::process::{Child, Command};
use std::thread::sleep;
use std::time::{Duration, Instant};

pub struct UdpStreamWrapper {
    pub our_port: u16,
    pub server: bool,
    pub socket: UdpSocket,
}

impl UdpStreamWrapper {
    pub fn from_port(port: u16) -> Self {
        let socket = UdpSocket::bind(format!("127.0.0.1:{}", port)).unwrap();
        let our_port = socket.local_addr().unwrap().port();
        Self {
            our_port,
            socket,
            server: false,
        }
    }

    pub fn new() -> UdpStreamWrapper {
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        let our_port = socket.local_addr().unwrap().port();
        Self {
            our_port,
            server: true,
            socket,
        }
    }
}

pub struct UdpRunner {
    child_proc: Option<Child>,
    wrapper: UdpStreamWrapper,
}

impl UdpRunner {
    pub fn new(start_child: bool) -> UdpRunner {
        let wrapper = UdpStreamWrapper::new();
        let their_port = portpicker::pick_unused_port().unwrap();
        let exe = crate::executable_path("udp_consumer");
        let child_proc = if start_child {
            Some(
                Command::new(exe)
                    .args(&[wrapper.our_port.to_string(), their_port.to_string()])
                    .spawn()
                    .unwrap(),
            )
        } else {
            None
        };
        // Awkward sleep to make sure the child proc is ready
        sleep(Duration::from_millis(100));
        wrapper
            .socket
            .connect(format!("127.0.0.1:{}", their_port))
            .expect("Child process can't connect");
        Self {
            child_proc,
            wrapper,
        }
    }

    pub fn run(&mut self, n: usize, print: bool) {
        let start = Instant::now();
        let mut buf = [0u8; 4];
        for _ in 0..n {
            self.wrapper.socket.send(b"ping").unwrap();
            self.wrapper.socket.recv(&mut buf).unwrap();
            if !buf.eq(b"pong") {
                panic!("Sent ping didn't get pong")
            }
        }
        if print {
            let elapsed = start.elapsed();
            let res = ExecutionResult::new("UDP".to_string(), elapsed, n);
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
