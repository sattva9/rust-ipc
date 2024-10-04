use std::io::{stdin, stdout, Read, Write};
use std::str::FromStr;

use ipc::get_payload;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let data_size = usize::from_str(&args[1]).unwrap();

    let (request_data, response_data) = get_payload(data_size);
    let error = "Error".to_string().as_bytes().to_vec();

    let mut buf = vec![0; data_size];

    loop {
        let read_result = stdin().read_exact(&mut buf);
        if read_result.is_ok() {
            let output = if buf == request_data {
                &response_data
            } else if buf == response_data {
                &request_data
            } else {
                &error
            };
            stdout().write(output).unwrap();
        }
    }
}
