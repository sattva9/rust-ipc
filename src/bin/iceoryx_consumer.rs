use ipc::get_payload;
use std::str::FromStr;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let data_size = usize::from_str(&args[1]).unwrap();

    let wrapper = ipc::iceoryx::IceoryxWrapper::new(false, data_size);
    let (request_data, response_data) = get_payload(data_size);

    loop {
        if let Some(recv_payload) = wrapper.subscriber.receive().unwrap() {
            if !recv_payload.eq(&request_data) {
                panic!("Received unexpected payload")
            }

            let sample = wrapper.publisher.loan_slice_uninit(data_size).unwrap();
            let sample = sample.write_from_slice(response_data.as_slice());
            sample.send().unwrap();
        }
    }
}
