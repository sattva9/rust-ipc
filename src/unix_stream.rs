use std::{
    io::{Read, Write},
    os::unix::net::{UnixListener, UnixStream},
    process::{Child, Command},
    thread::sleep,
    time::{Duration, Instant},
};

use crate::{get_payload, ExecutionResult, KB};

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
    data_size: usize,
    request_data: Vec<u8>,
    response_data: Vec<u8>,
}

impl UnixStreamRunner {
    pub fn new(start_child: bool, data_size: usize) -> Self {
        let unix_listener = UnixListener::bind(UNIX_SOCKET_PATH).unwrap();
        let exe = crate::executable_path("unix_stream_consumer");
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

        let wrapper = UnixStreamWrapper::from_listener(unix_listener);

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
                format!("Unix TCP Socket - {}KB", self.data_size / KB).to_string(),
                elapsed,
                n,
            );
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
