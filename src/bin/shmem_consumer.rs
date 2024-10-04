use ipc::get_payload;
use raw_sync::Timeout;
use std::str::FromStr;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let handle = &args[1];
    let data_size = usize::from_str(&args[2]).unwrap();

    // First two bytes is the producer busy event, second two bytes is the consumer busy event.
    // The rest is our message
    let mut wrapper = ipc::shmem::ShmemWrapper::new(Some(handle.clone()), data_size);
    let (request_data, response_data) = get_payload(data_size);

    loop {
        if wrapper.their_event.wait(Timeout::Infinite).is_ok() {
            let _data = wrapper.read();
            if wrapper.read() == &request_data {
                wrapper.signal_start();
                wrapper.write(&response_data);
                wrapper.signal_finished();
            } else {
                panic!("Didn't receive valid request")
            }
        }
    }
}
