use std::path::PathBuf;
use std::time::Duration;

pub mod iceoryx;
pub mod mmap;
pub mod pipes;
pub mod shmem;
pub mod tcp;
pub mod udp;
pub mod unix_datagram;
pub mod unix_stream;

use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;

pub const KB: usize = 1024;

pub fn generate_random_data(data_size: usize, seed: u64) -> Vec<u8> {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789";
    let mut rng = StdRng::seed_from_u64(seed);

    (0..data_size)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx]
        })
        .collect()
}

pub fn get_payload(data_size: usize) -> (Vec<u8>, Vec<u8>) {
    let request_data = generate_random_data(data_size, 1);
    let response_data = generate_random_data(data_size, 2);
    (request_data, response_data)
}

pub fn cpu_warmup() {
    let warmup = std::time::Instant::now();
    loop {
        if warmup.elapsed() > std::time::Duration::from_millis(1000) {
            break;
        }
    }
}

pub struct ExecutionResult {
    name: String,
    elapsed: Duration,
    cycles: usize,
}

impl ExecutionResult {
    fn new(name: String, elapsed: Duration, cycles: usize) -> ExecutionResult {
        ExecutionResult {
            name,
            elapsed,
            cycles,
        }
    }

    fn print_info(&self) {
        let duration = humantime::Duration::from(self.elapsed);
        let ps = 1_000_000f32 * (self.cycles as f32) / (duration.as_micros() as f32);
        let per_op =
            humantime::Duration::from(Duration::from_nanos((1_000_000_000f32 / ps) as u64));
        println!(
            "IPC method - {}\n\t{} cycles completed in {} \n\t{} per second\n\t{} per operation",
            self.name, self.cycles, duration, ps, per_op
        );
    }
}

fn executable_path(name: &str) -> PathBuf {
    #[cfg(target_os = "windows")]
    let exe = name.to_owned() + ".exe";
    #[cfg(target_family = "unix")]
    let exe = name.to_owned();

    #[cfg(debug_assertions)]
    let out = PathBuf::from("./target/debug/").join(exe);
    #[cfg(not(debug_assertions))]
    let out = PathBuf::from("./target/release/").join(exe);

    out
}
