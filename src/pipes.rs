use crate::{get_payload, ExecutionResult, KB};
use std::io::{Read, Write};
use std::process::{Child, Command, Stdio};
use std::time::Instant;

pub struct PipeRunner {
    pipe_proc: Child,
    data_size: usize,
    request_data: Vec<u8>,
    response_data: Vec<u8>,
}

impl PipeRunner {
    pub fn new(data_size: usize) -> PipeRunner {
        // let output_dir = PathBuf::from(env::var("CARGO_TARGET_DIR").unwrap());
        // let output_dir = PathBuf::new();
        // let exe = output_dir.join("pipes_consumer.exe");
        let exe = crate::executable_path("pipes_consumer");

        let (request_data, response_data) = get_payload(data_size);

        PipeRunner {
            pipe_proc: Command::new(exe)
                .args(&[data_size.to_string()])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .unwrap(),
            data_size,
            request_data,
            response_data,
        }
    }

    pub fn run_inner(&mut self, n: usize) {
        if let Some(ref mut pipes_input) = self.pipe_proc.stdin {
            if let Some(ref mut pipes_output) = self.pipe_proc.stdout {
                let mut buf = vec![0; self.data_size];
                for _ in 0..n {
                    pipes_input.write(&self.request_data).unwrap();
                    pipes_output.read_exact(&mut buf).unwrap();

                    #[cfg(debug_assertions)]
                    if buf.ne(&self.response_data) {
                        panic!("Unexpected response {}", String::from_utf8_lossy(&buf))
                    }
                }
            }
        }
    }

    pub fn run(&mut self, n: usize, print: bool) {
        let instant = Instant::now();
        self.run_inner(n);
        let elapsed = instant.elapsed();
        if print {
            let res = ExecutionResult::new(
                format!("Stdin/stdout - {}KB", self.data_size / KB),
                elapsed,
                n,
            );
            res.print_info()
        }
    }
}

impl Drop for PipeRunner {
    fn drop(&mut self) {
        self.pipe_proc.kill().unwrap();
    }
}
