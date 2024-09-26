use crate::ExecutionResult;
use raw_sync::events::{BusyEvent, EventImpl, EventInit, EventState};
use raw_sync::Timeout;
use shared_memory::{Shmem, ShmemConf};
use std::process::{Child, Command};
use std::thread::sleep;
use std::time::{Duration, Instant};

fn shmem_conf() -> ShmemConf {
    let shmem = ShmemConf::new().size(8);
    shmem
}

pub struct ShmemWrapper {
    pub shmem: Shmem,
    pub owner: bool,
    pub our_event: Box<dyn EventImpl>,
    pub their_event: Box<dyn EventImpl>,
    pub data_start: usize,
}

impl ShmemWrapper {
    pub fn new(handle: Option<String>) -> ShmemWrapper {
        let owner = handle.is_none();
        // If we've been given a memory handle, attach it, if not, create one
        let mut shmem = match handle {
            None => shmem_conf().create().unwrap(),
            Some(h) => shmem_conf()
                .os_id(&h)
                .open()
                .expect(&format!("Unable to open the shared memory at {}", h)),
        };
        let bytes = unsafe { shmem.as_slice_mut() };
        // The two events are locks - one for each side. Each side activates the lock while it's
        // writing, and then unlocks when the data can be read
        let ((our_event, lock_bytes_ours), (their_event, lock_bytes_theirs)) = unsafe {
            if owner {
                (
                    BusyEvent::new(bytes.get_mut(0).unwrap(), true).unwrap(),
                    BusyEvent::new(bytes.get_mut(2).unwrap(), true).unwrap(),
                )
            } else {
                (
                    // If we're not the owner, the events have been created already
                    BusyEvent::from_existing(bytes.get_mut(2).unwrap()).unwrap(),
                    BusyEvent::from_existing(bytes.get_mut(0).unwrap()).unwrap(),
                )
            }
        };
        // Confirm that we've correctly indexed two bytes for each lock
        assert!(lock_bytes_ours <= 2);
        assert!(lock_bytes_theirs <= 2);
        if owner {
            our_event.set(EventState::Clear).unwrap();
            their_event.set(EventState::Clear).unwrap();
        }
        ShmemWrapper {
            shmem,
            owner,
            our_event,
            their_event,
            data_start: 4,
        }
    }

    pub fn signal_start(&mut self) {
        self.our_event.set(EventState::Clear).unwrap()
    }
    pub fn signal_finished(&mut self) {
        self.our_event.set(EventState::Signaled).unwrap()
    }

    pub fn write(&mut self, data: &[u8; 4]) {
        let bytes = unsafe { self.shmem.as_slice_mut() };

        for i in 0..data.len() {
            bytes[i + self.data_start] = data[i];
        }
    }

    pub fn read(&self) -> &[u8] {
        unsafe { &self.shmem.as_slice()[self.data_start..self.data_start + 4] }
    }
}

// #[derive(Debug)]
pub struct ShmemRunner {
    pub child_proc: Option<Child>,
    pub wrapper: ShmemWrapper,
}

impl ShmemRunner {
    pub fn new(start_child: bool) -> ShmemRunner {
        let wrapper = ShmemWrapper::new(None);

        let id = wrapper.shmem.get_os_id();
        let exe = crate::executable_path("shmem_consumer");
        let child_proc = if start_child {
            let res = Some(Command::new(exe).args(&[id]).spawn().unwrap());
            // Clumsy sleep here but it allows the child proc to spawn without it having to offer
            // us a ready event
            sleep(Duration::from_secs(1));
            res
        } else {
            None
        };
        ShmemRunner {
            child_proc,
            wrapper,
        }
    }

    pub fn run(&mut self, n: usize, print: bool) {
        let instant = Instant::now();
        for _ in 0..n {
            // Activate our lock in preparation for writing
            self.wrapper.signal_start();
            self.wrapper.write(b"ping");
            // Unlock after writing
            self.wrapper.signal_finished();
            // Wait for their lock to be released so we can read
            if self.wrapper.their_event.wait(Timeout::Infinite).is_ok() {
                let str = self.wrapper.read();
                if str != b"pong" {
                    panic!("Sent ping didn't get pong")
                }
            }
        }
        let elapsed = instant.elapsed();

        if print {
            let res = ExecutionResult::new("Shared memory".to_string(), elapsed, n);
            res.print_info();
        }
    }
}

impl Drop for ShmemRunner {
    fn drop(&mut self) {
        if let Some(ref mut child) = self.child_proc {
            child.kill().expect("Unable to kill child process")
        }
    }
}
