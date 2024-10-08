use crate::{get_payload, ExecutionResult, KB};
use iceoryx2::port::publisher::Publisher;
use iceoryx2::port::subscriber::Subscriber;
use iceoryx2::prelude::*;
use std::process::{Child, Command};
use std::thread::sleep;
use std::time::{Duration, Instant};

pub struct IceoryxWrapper {
    pub publisher: Publisher<ipc::Service, [u8], ()>,
    pub subscriber: Subscriber<ipc::Service, [u8], ()>,
}

impl IceoryxWrapper {
    pub fn new(is_producer: bool, data_size: usize) -> IceoryxWrapper {
        let node = NodeBuilder::new().create::<ipc::Service>().unwrap();
        let request_name = ServiceName::new(&format!("Request")).unwrap();
        let request_service = node
            .service_builder(&request_name)
            .publish_subscribe::<[u8]>()
            .open_or_create()
            .unwrap();

        let response_name = ServiceName::new(&format!("Respose")).unwrap();
        let response_service = node
            .service_builder(&response_name)
            .publish_subscribe::<[u8]>()
            .open_or_create()
            .unwrap();

        let (publisher, subscriber) = if is_producer {
            (
                request_service
                    .publisher_builder()
                    .max_slice_len(data_size)
                    .create()
                    .unwrap(),
                response_service.subscriber_builder().create().unwrap(),
            )
        } else {
            (
                response_service
                    .publisher_builder()
                    .max_slice_len(data_size)
                    .create()
                    .unwrap(),
                request_service.subscriber_builder().create().unwrap(),
            )
        };

        IceoryxWrapper {
            publisher,
            subscriber,
        }
    }
}

pub struct IceoryxRunner {
    child_proc: Option<Child>,
    wrapper: IceoryxWrapper,
    request_data: Vec<u8>,
    response_data: Vec<u8>,
    data_size: usize,
}

impl IceoryxRunner {
    pub fn new(start_child: bool, data_size: usize) -> IceoryxRunner {
        let wrapper = IceoryxWrapper::new(true, data_size);
        let exe = crate::executable_path("iceoryx_consumer");

        let child_proc = if start_child {
            // None
            Some(
                Command::new(exe)
                    .args(&[data_size.to_string()])
                    .spawn()
                    .unwrap(),
            )
        } else {
            None
        };
        // Awkward sleep again to wait for consumer to be ready
        sleep(Duration::from_secs(2));

        let (request_data, response_data) = get_payload(data_size);

        Self {
            child_proc,
            wrapper,
            request_data,
            response_data,
            data_size,
        }
    }

    pub fn run(&mut self, n: usize, print: bool) {
        let start = Instant::now();
        for _ in 0..n {
            let sample = self
                .wrapper
                .publisher
                .loan_slice_uninit(self.data_size)
                .unwrap();
            let sample = sample.write_from_slice(self.request_data.as_slice());
            sample.send().unwrap();

            // Waiting for response
            loop {
                if let Some(recv_payload) = self.wrapper.subscriber.receive().unwrap() {
                    #[cfg(debug_assertions)]
                    if recv_payload.ne(&self.response_data) {
                        panic!("Sent request didn't get response")
                    }

                    break;
                }
            }
        }
        if print {
            let elapsed = start.elapsed();
            let res =
                ExecutionResult::new(format!("Iceoryx - {}KB", self.data_size / KB), elapsed, n);
            res.print_info();
        }
    }
}

impl Drop for IceoryxRunner {
    fn drop(&mut self) {
        if let Some(ref mut c) = self.child_proc {
            c.kill().unwrap();
        }
    }
}
