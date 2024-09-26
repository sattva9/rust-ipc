fn main() {
    let is_child = true;
    let wrapper = ipc::unix_datagram::UnixDatagramWrapper::new(is_child);
    wrapper.connect_to_peer();

    let mut buf = [0u8; 4];
    while let Ok(_) = wrapper.socket.recv(&mut buf) {
        if buf.eq(b"ping") {
            wrapper.socket.send(b"pong").unwrap();
        } else {
            panic!("Received unknown value {:?}", buf)
        }
    }
}
