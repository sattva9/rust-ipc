use std::io::{stdin, stdout, Read, Write};

fn main() {
    let mut arr = [0u8, 0, 0, 0, 0];
    loop {
        let read_result = stdin().read_exact(&mut arr);
        if read_result.is_ok() {
            let output = match &arr {
                b"ping\n" => b"pong\n",
                b"pong\n" => b"ping\n",
                _ => b"Error",
            };
            stdout().write(output).unwrap();
        }
    }
}
