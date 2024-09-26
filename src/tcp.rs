use crate::ExecutionResult;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command};
use std::time::Instant;

pub struct TcpStreamWrapper {
    pub port: u16,
    pub server: bool,
    pub stream: TcpStream,
}

impl TcpStreamWrapper {
    pub fn from_port(port: u16, tcp_nodelay: bool) -> Self {
        let stream = TcpStream::connect(format!("127.0.0.1:{}", port)).unwrap();
        stream.set_nodelay(tcp_nodelay).unwrap();
        Self {
            port,
            stream,
            server: false,
        }
    }

    pub fn from_listener(tcp_listener: TcpListener, tcp_nodelay: bool) -> TcpStreamWrapper {
        let addr = tcp_listener.local_addr().unwrap();
        let (stream, _socket) = tcp_listener.accept().unwrap();
        stream.set_nodelay(tcp_nodelay).unwrap();
        Self {
            port: addr.port(),
            server: true,
            stream,
        }
    }
}

pub struct TcpRunner {
    child_proc: Option<Child>,
    wrapper: TcpStreamWrapper,
    tcp_nodelay: bool,
}

impl TcpRunner {
    pub fn new(start_child: bool, tcp_nodelay: bool) -> TcpRunner {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let exe = crate::executable_path("tcp_consumer");
        let child_proc = if start_child {
            Some(
                Command::new(exe)
                    .args(&[port.to_string(), tcp_nodelay.to_string()])
                    .spawn()
                    .unwrap(),
            )
        } else {
            None
        };
        let stream = TcpStreamWrapper::from_listener(listener, tcp_nodelay);
        Self {
            child_proc,
            wrapper: stream,
            tcp_nodelay,
        }
    }

    pub fn run(&mut self, n: usize, print: bool) {
        let start = Instant::now();
        // TODO: Decide whether this can be done without copying from the socket
        let mut buf = [0u8; 4];
        for _ in 0..n {
            self.wrapper.stream.write(b"ping").unwrap();
            self.wrapper.stream.read_exact(&mut buf).unwrap();
            if !buf.eq(b"pong") {
                panic!("Sent ping didn't get pong")
            }
        }
        if print {
            let elapsed = start.elapsed();
            let res = ExecutionResult::new(
                format!("TCP - nodelay={}", self.tcp_nodelay).to_string(),
                elapsed,
                n,
            );
            res.print_info();
        }
    }
}

impl Drop for TcpRunner {
    fn drop(&mut self) {
        if let Some(ref mut c) = self.child_proc {
            c.kill().unwrap();
        }
    }
}
