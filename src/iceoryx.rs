use crate::ExecutionResult;
use iceoryx2::port::publisher::Publisher;
use iceoryx2::port::subscriber::Subscriber;
use iceoryx2::prelude::*;
use std::process::{Child, Command};
use std::thread::sleep;
use std::time::{Duration, Instant};

pub struct IceoryxWrapper {
    pub publisher: Publisher<zero_copy::Service, [u8; 4]>,
    pub subscriber: Subscriber<zero_copy::Service, [u8; 4]>,
}

impl IceoryxWrapper {
    pub fn new(is_producer: bool) -> IceoryxWrapper {
        const REQUEST_SERVICE_NAME: &'static str = "Request";
        const RESPONSE_SERVICE_NAME: &'static str = "Response";

        let request_name = ServiceName::new(REQUEST_SERVICE_NAME).unwrap();
        let request_service = zero_copy::Service::new(&request_name)
            .publish_subscribe()
            .open_or_create()
            .unwrap();

        let response_name = ServiceName::new(RESPONSE_SERVICE_NAME).unwrap();
        let response_service = zero_copy::Service::new(&response_name)
            .publish_subscribe()
            .open_or_create()
            .unwrap();

        let (publisher, subscriber) = if is_producer {
            (
                request_service.publisher().create().unwrap(),
                response_service.subscriber().create().unwrap(),
            )
        } else {
            (
                response_service.publisher().create().unwrap(),
                request_service.subscriber().create().unwrap(),
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
}

impl IceoryxRunner {
    pub fn new(start_child: bool) -> IceoryxRunner {
        let exe = crate::executable_path("iceoryx_consumer");
        let child_proc = if start_child {
            Some(Command::new(exe).spawn().unwrap())
        } else {
            None
        };
        // Awkward sleep again to wait for consumer to be ready
        sleep(Duration::from_millis(1000));

        let wrapper = IceoryxWrapper::new(true);
        Self {
            child_proc,
            wrapper,
        }
    }

    pub fn run(&mut self, n: usize, print: bool) {
        let start = Instant::now();
        for _ in 0..n {
            let sample = self.wrapper.publisher.loan_uninit().unwrap();
            let send_payload = sample.write_payload((*b"ping").into());
            send_payload.send().unwrap();

            // Waiting for response
            loop {
                if let Some(recv_payload) = self.wrapper.subscriber.receive().unwrap() {
                    if !recv_payload.eq(b"pong") {
                        panic!("Received unexpected payload")
                    }
                    break;
                }
            }
        }
        if print {
            let elapsed = start.elapsed();
            let res = ExecutionResult::new("Iceoryx".to_string(), elapsed, n);
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
