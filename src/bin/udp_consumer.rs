use std::str::FromStr;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let our_port = u16::from_str(&args[2]).unwrap();
    let their_port = u16::from_str(&args[1]).unwrap();
    let wrapper = ipc::udp::UdpStreamWrapper::from_port(our_port);
    wrapper
        .socket
        .connect(format!("127.0.0.1:{}", their_port))
        .unwrap();
    let mut buf = [0u8; 4];
    loop {
        wrapper.socket.recv(&mut buf).unwrap();
        if buf.eq(b"ping") {
            wrapper.socket.send(b"pong").unwrap();
        } else {
            panic!("Received unknown value {:?}", buf)
        }
    }
}
