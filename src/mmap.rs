use std::{
    fs::OpenOptions,
    path::PathBuf,
    process::{Child, Command},
    thread::sleep,
    time::{Duration, Instant},
};

use memmap2::MmapMut;
use raw_sync::{
    events::{BusyEvent, EventImpl, EventInit, EventState},
    Timeout,
};

use crate::ExecutionResult;

pub struct MmapWrapper {
    pub mmap: MmapMut,
    pub owner: bool,
    pub our_event: Box<dyn EventImpl>,
    pub their_event: Box<dyn EventImpl>,
    pub data_start: usize,
}

impl MmapWrapper {
    pub fn new(owner: bool) -> Self {
        let path: PathBuf = "/tmp/mmap_data.txt".into();
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)
            .unwrap();
        file.set_len(8).unwrap();

        let mut mmap = unsafe { MmapMut::map_mut(&file).unwrap() };
        let bytes = mmap.as_mut();

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

        Self {
            mmap,
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

    pub fn write(&mut self, data: &[u8]) {
        let bytes = self.mmap.as_mut();

        for i in 0..data.len() {
            bytes[i + self.data_start] = data[i];
        }
    }

    pub fn read(&self) -> &[u8] {
        &self.mmap.as_ref()[self.data_start..self.data_start + 4]
    }
}

pub struct MmapRunner {
    child_proc: Option<Child>,
    wrapper: MmapWrapper,
}

impl MmapRunner {
    pub fn new(start_child: bool) -> Self {
        let wrapper = MmapWrapper::new(true);

        let exe = crate::executable_path("mmap_consumer");
        let child_proc = if start_child {
            let res = Some(Command::new(exe).spawn().unwrap());
            // Clumsy sleep here but it allows the child proc to spawn without it having to offer
            // us a ready event
            sleep(Duration::from_secs(2));
            res
        } else {
            None
        };
        Self {
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
            let res = ExecutionResult::new(format!("Memory mapped file"), elapsed, n);
            res.print_info();
        }
    }
}

impl Drop for MmapRunner {
    fn drop(&mut self) {
        if let Some(ref mut child) = self.child_proc {
            child.kill().expect("Unable to kill child process")
        }
    }
}
