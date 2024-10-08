use crate::{get_payload, ExecutionResult, KB};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command};
use std::thread::sleep;
use std::time::{Duration, Instant};

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
            server: false,
            stream,
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
    data_size: usize,
    request_data: Vec<u8>,
    response_data: Vec<u8>,
}

impl TcpRunner {
    pub fn new(start_child: bool, tcp_nodelay: bool, data_size: usize) -> TcpRunner {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let exe = crate::executable_path("tcp_consumer");
        let child_proc = if start_child {
            let res = Some(
                Command::new(exe)
                    .args(&[
                        port.to_string(),
                        tcp_nodelay.to_string(),
                        data_size.to_string(),
                    ])
                    .spawn()
                    .unwrap(),
            );
            // Awkward sleep to wait for consumer to be ready
            sleep(Duration::from_secs(2));
            res
        } else {
            None
        };

        let stream = TcpStreamWrapper::from_listener(listener, tcp_nodelay);

        let (request_data, response_data) = get_payload(data_size);

        Self {
            child_proc,
            wrapper: stream,
            tcp_nodelay,
            data_size,
            request_data,
            response_data,
        }
    }

    pub fn run(&mut self, n: usize, print: bool) {
        let start = Instant::now();
        let mut buf = vec![0; self.data_size];
        for _ in 0..n {
            self.wrapper.stream.write(&self.request_data).unwrap();
            self.wrapper.stream.read_exact(&mut buf).unwrap();

            #[cfg(debug_assertions)]
            if buf.ne(&self.response_data) {
                panic!("Sent request didn't get response")
            }
        }
        if print {
            let elapsed = start.elapsed();
            let res = ExecutionResult::new(
                format!(
                    "TCP - nodelay={} - {}KB",
                    self.tcp_nodelay,
                    self.data_size / KB
                )
                .to_string(),
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
