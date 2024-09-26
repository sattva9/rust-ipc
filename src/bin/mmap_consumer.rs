use raw_sync::Timeout;

fn main() {
    // First two bytes is the producer busy event, second two bytes is the consumer busy event.
    // The rest is our message
    let mut wrapper = ipc::mmap::MmapWrapper::new(false);

    loop {
        if wrapper.their_event.wait(Timeout::Infinite).is_ok() {
            let _data = wrapper.read();
            if wrapper.read() == b"ping" {
                wrapper.signal_start();
                wrapper.write(b"pong");
                wrapper.signal_finished();
            } else {
                panic!("Didn't receive ping")
            }
        }
    }
}
