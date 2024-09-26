use std::io::{Read, Write};

fn main() {
    let mut wrapper = ipc::unix_stream::UnixStreamWrapper::unix_connect();
    let mut buf = [0u8; 4];
    while let Ok(_) = wrapper.stream.read(&mut buf) {
        if buf.eq(b"ping") {
            wrapper.stream.write(b"pong").unwrap();
        } else {
            panic!("Received unknown value {:?}", buf)
        }
    }
}
