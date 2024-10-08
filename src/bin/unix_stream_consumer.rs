use std::io::{Read, Write};
use std::str::FromStr;

use ipc::{cpu_warmup, get_payload};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let data_size = usize::from_str(&args[1]).unwrap();

    core_affinity::set_for_current(core_affinity::CoreId { id: 0 });

    let mut wrapper = ipc::unix_stream::UnixStreamWrapper::unix_connect();

    let (request_data, response_data) = get_payload(data_size);

    cpu_warmup();

    let mut buf = vec![0; data_size];
    while let Ok(_) = wrapper.stream.read_exact(&mut buf) {
        #[cfg(debug_assertions)]
        if buf.ne(&request_data) {
            panic!("Didn't receive valid request")
        }

        wrapper.stream.write(&response_data).unwrap();
    }
}
