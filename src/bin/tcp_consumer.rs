use std::io::{Read, Write};
use std::str::FromStr;

use ipc::get_payload;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let port = u16::from_str(&args[1]).unwrap();
    let nodelay = bool::from_str(&args[2]).unwrap();
    let data_size = usize::from_str(&args[3]).unwrap();
    let mut wrapper = ipc::tcp::TcpStreamWrapper::from_port(port, nodelay);

    let (request_data, response_data) = get_payload(data_size);

    let mut buf = vec![0; data_size];
    while let Ok(_) = wrapper.stream.read_exact(&mut buf) {
        if buf.eq(&request_data) {
            wrapper.stream.write(&response_data).unwrap();
        } else {
            panic!("Didn't receive valid request")
        }
    }
}
