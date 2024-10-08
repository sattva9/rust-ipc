use ipc::{cpu_warmup, get_payload};
use std::str::FromStr;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let data_size = usize::from_str(&args[1]).unwrap();

    core_affinity::set_for_current(core_affinity::CoreId { id: 0 });

    let wrapper = ipc::iceoryx::IceoryxWrapper::new(false, data_size);
    let (request_data, response_data) = get_payload(data_size);

    cpu_warmup();

    loop {
        if let Some(recv_payload) = wrapper.subscriber.receive().unwrap() {
            #[cfg(debug_assertions)]
            if recv_payload.ne(&request_data) {
                panic!("Received unexpected payload")
            }

            let sample = wrapper.publisher.loan_slice_uninit(data_size).unwrap();
            let sample = sample.write_from_slice(response_data.as_slice());
            sample.send().unwrap();
        }
    }
}
