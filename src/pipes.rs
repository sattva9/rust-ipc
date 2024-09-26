use crate::ExecutionResult;
use std::io::{Read, Write};
use std::process::{Child, Command, Stdio};
use std::time::Instant;

pub struct PipeRunner {
    pub pipe_proc: Child,
}

impl PipeRunner {
    pub fn new(_p0: bool) -> PipeRunner {
        // let output_dir = PathBuf::from(env::var("CARGO_TARGET_DIR").unwrap());
        // let output_dir = PathBuf::new();
        // let exe = output_dir.join("pipes_consumer.exe");
        let exe = crate::executable_path("pipes_consumer");
        PipeRunner {
            pipe_proc: Command::new(exe)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .unwrap(),
        }
    }

    pub fn prepare(&mut self) -> [u8; 5] {
        let return_value = [0u8, 0, 0, 0, 0];
        return_value
    }
    pub fn run_inner(&mut self, n: usize, return_value: &mut [u8; 5]) {
        if let Some(ref mut pipes_input) = self.pipe_proc.stdin {
            if let Some(ref mut pipes_output) = self.pipe_proc.stdout {
                for _ in 0..n {
                    pipes_input.write(b"ping\n").unwrap();
                    pipes_output.read_exact(return_value).unwrap();
                    if return_value != b"pong\n" {
                        panic!("Unexpected response")
                    }
                }
            }
        }
    }

    pub fn run(&mut self, n: usize, print: bool) {
        let mut return_buffer = self.prepare();
        let instant = Instant::now();
        self.run_inner(n, &mut return_buffer);
        let elapsed = instant.elapsed();
        if print {
            let res = ExecutionResult::new("Stdin/stdout".to_string(), elapsed, n);
            res.print_info()
        }
    }
}

impl Drop for PipeRunner {
    fn drop(&mut self) {
        self.pipe_proc.kill().unwrap();
    }
}
