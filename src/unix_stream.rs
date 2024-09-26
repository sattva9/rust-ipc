use std::{
    io::{Read, Write},
    os::unix::net::{UnixListener, UnixStream},
    process::{Child, Command},
    time::Instant,
};

use crate::ExecutionResult;

const UNIX_SOCKET_PATH: &str = "/tmp/unix_stream.sock";

pub struct UnixStreamWrapper {
    pub stream: UnixStream,
}

impl UnixStreamWrapper {
    pub fn from_listener(listener: UnixListener) -> Self {
        let (stream, _socket) = listener.accept().unwrap();
        Self { stream }
    }

    pub fn unix_connect() -> Self {
        let stream = UnixStream::connect(UNIX_SOCKET_PATH).unwrap();
        Self { stream }
    }
}

pub struct UnixStreamRunner {
    child_proc: Option<Child>,
    wrapper: UnixStreamWrapper,
}

impl UnixStreamRunner {
    pub fn new(start_child: bool) -> Self {
        let unix_listener = UnixListener::bind(UNIX_SOCKET_PATH).unwrap();
        let exe = crate::executable_path("unix_stream_consumer");
        let child_proc = if start_child {
            Some(Command::new(exe).spawn().unwrap())
        } else {
            None
        };

        let wrapper = UnixStreamWrapper::from_listener(unix_listener);

        Self {
            child_proc,
            wrapper,
        }
    }

    pub fn run(&mut self, n: usize, print: bool) {
        let start = Instant::now();
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
            let res = ExecutionResult::new(format!("Unix TCP Socket").to_string(), elapsed, n);
            res.print_info();
        }
    }
}

impl Drop for UnixStreamRunner {
    fn drop(&mut self) {
        if let Some(ref mut c) = self.child_proc {
            c.kill().unwrap();
        }
        let _ = std::fs::remove_file(UNIX_SOCKET_PATH);
    }
}
