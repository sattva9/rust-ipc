use std::{
    os::unix::net::UnixDatagram,
    process::{Child, Command},
    thread::sleep,
    time::{Duration, Instant},
};

use crate::{get_payload, ExecutionResult, KB};

const MAX_CHUNK_SIZE: usize = 64 * KB;
const UNIX_DATAGRAM_SOCKET_1: &str = "/tmp/unix_datagram1.sock";
const UNIX_DATAGRAM_SOCKET_2: &str = "/tmp/unix_datagram2.sock";

pub struct UnixDatagramWrapper {
    pub socket: UnixDatagram,
    pub peer_socket_path: String,
    pub data_size: usize,
}

impl UnixDatagramWrapper {
    pub fn new(is_child: bool, data_size: usize) -> Self {
        let (socket_path, peer_socket_path) = if is_child {
            (UNIX_DATAGRAM_SOCKET_1, UNIX_DATAGRAM_SOCKET_2)
        } else {
            (UNIX_DATAGRAM_SOCKET_2, UNIX_DATAGRAM_SOCKET_1)
        };
        let socket = UnixDatagram::bind(socket_path).unwrap();

        Self {
            socket,
            peer_socket_path: peer_socket_path.to_string(),
            data_size,
        }
    }

    pub fn connect_to_peer(&self) {
        self.socket.connect(&self.peer_socket_path).unwrap();
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
            let size = self.socket.recv(&mut buf).unwrap();
            received_data.extend_from_slice(&buf[..size]);
            if received_data.len() == self.data_size {
                break;
            }
        }
        received_data
    }
}

pub struct UnixDatagramRunner {
    child_proc: Option<Child>,
    wrapper: UnixDatagramWrapper,
    data_size: usize,
    request_data: Vec<u8>,
    #[allow(unused)]
    response_data: Vec<u8>,
}

impl UnixDatagramRunner {
    pub fn new(start_child: bool, data_size: usize) -> Self {
        let is_child = false;
        let wrapper = UnixDatagramWrapper::new(is_child, data_size);

        let exe = crate::executable_path("unix_datagram_consumer");
        let child_proc = if start_child {
            let res = Some(
                Command::new(exe)
                    .args(&[data_size.to_string()])
                    .spawn()
                    .unwrap(),
            );
            // Awkward sleep to wait for consumer to be ready
            sleep(Duration::from_secs(2));
            res
        } else {
            None
        };

        wrapper.connect_to_peer();

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
        }
        if print {
            let elapsed = start.elapsed();
            let res = ExecutionResult::new(
                format!("Unix DATAGRAM Socket - {}KB", self.data_size / KB).to_string(),
                elapsed,
                n,
            );
            res.print_info();
        }
    }
}

impl Drop for UnixDatagramRunner {
    fn drop(&mut self) {
        if let Some(ref mut c) = self.child_proc {
            c.kill().unwrap();
        }
        let _ = std::fs::remove_file(UNIX_DATAGRAM_SOCKET_1);
        let _ = std::fs::remove_file(UNIX_DATAGRAM_SOCKET_2);
    }
}
