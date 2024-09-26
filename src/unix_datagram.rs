use std::{
    os::unix::net::UnixDatagram,
    process::{Child, Command},
    thread::sleep,
    time::{Duration, Instant},
};

use crate::ExecutionResult;

const UNIX_DATAGRAM_SOCKET_1: &str = "/tmp/unix_datagram1.sock";
const UNIX_DATAGRAM_SOCKET_2: &str = "/tmp/unix_datagram2.sock";

pub struct UnixDatagramWrapper {
    pub socket: UnixDatagram,
    pub peer_socket_path: String,
}

impl UnixDatagramWrapper {
    pub fn new(is_child: bool) -> Self {
        let (socket_path, peer_socket_path) = if is_child {
            (UNIX_DATAGRAM_SOCKET_1, UNIX_DATAGRAM_SOCKET_2)
        } else {
            (UNIX_DATAGRAM_SOCKET_2, UNIX_DATAGRAM_SOCKET_1)
        };
        let socket = UnixDatagram::bind(socket_path).unwrap();

        Self {
            socket,
            peer_socket_path: peer_socket_path.to_string(),
        }
    }

    pub fn connect_to_peer(&self) {
        self.socket.connect(&self.peer_socket_path).unwrap();
    }
}

pub struct UnixDatagramRunner {
    child_proc: Option<Child>,
    wrapper: UnixDatagramWrapper,
}

impl UnixDatagramRunner {
    pub fn new(start_child: bool) -> Self {
        let is_child = false;
        let wrapper = UnixDatagramWrapper::new(is_child);

        let exe = crate::executable_path("unix_datagram_consumer");
        let child_proc = if start_child {
            Some(Command::new(exe).spawn().unwrap())
        } else {
            None
        };

        // Awkward sleep to make sure the child proc is ready
        sleep(Duration::from_millis(500));
        wrapper.connect_to_peer();

        Self {
            child_proc,
            wrapper,
        }
    }

    pub fn run(&mut self, n: usize, print: bool) {
        let start = Instant::now();
        // TODO: Decide whether this can be done without copying from the socket
        let mut buf = [0u8; 4];
        for _ in 0..n {
            self.wrapper.socket.send(b"ping").unwrap();
            self.wrapper.socket.recv_from(&mut buf).unwrap();
            if !buf.eq(b"pong") {
                panic!(
                    "Sent ping didn't get pong. {}",
                    String::from_utf8_lossy(&buf)
                )
            }
        }
        if print {
            let elapsed = start.elapsed();
            let res = ExecutionResult::new(format!("Unix DATAGRAM Socket").to_string(), elapsed, n);
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
