use std::io::{Read, Write};
use std::str::FromStr;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let port = u16::from_str(&args[1]).unwrap();
    let nodelay = bool::from_str(&args[2]).unwrap();
    let mut wrapper = ipc::tcp::TcpStreamWrapper::from_port(port, nodelay);
    let mut buf = [0u8; 4];
    while let Ok(_) = wrapper.stream.read(&mut buf) {
        if buf.eq(b"ping") {
            wrapper.stream.write(b"pong").unwrap();
        } else {
            panic!("Received unknown value {:?}", buf)
        }
    }
}
