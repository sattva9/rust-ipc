use std::str::FromStr;

use ipc::{cpu_warmup, get_payload};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let our_port = u16::from_str(&args[2]).unwrap();
    let their_port = u16::from_str(&args[1]).unwrap();
    let data_size = usize::from_str(&args[3]).unwrap();

    core_affinity::set_for_current(core_affinity::CoreId { id: 0 });

    let socket_wrapper = ipc::udp::UdpStreamWrapper::from_port(our_port, data_size);
    socket_wrapper
        .socket
        .connect(format!("127.0.0.1:{}", their_port))
        .unwrap();

    let (_request_data, response_data) = get_payload(data_size);

    cpu_warmup();

    loop {
        let _request = socket_wrapper.recv();
        socket_wrapper.send(&response_data);
        // if request.eq(&request_data) {
        //     socket_wrapper.send(&response_data);
        // } else {
        //     panic!("Received unknown value")
        // }
    }
}
