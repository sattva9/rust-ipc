// This is in place to allow me to test a local Divan change
#[cfg(feature = "dev-divan")]
extern crate divan_dev as divan;

use divan::Bencher;

// This affects the number cycles of to execute each method for. In the Divan output, the
// time per function will be displayed for the total number of cycles, but the throughput
// will be displayed per cycle. So to get timing per cycle, do t/N. A workaround for this
// hopefully will be found
const N: usize = 1000;

fn main() {
    divan::main();
}

// This creates the return array "holding the response" before passing it to the function
#[divan::bench]
fn stdin_stdout_no_preallocate(bencher: Bencher) {
    let n = N;
    let mut pipe_runner = ipc::pipes::PipeRunner::new(false);
    bencher
        .counter(n)
        .bench_local(move || pipe_runner.run(n, false));
}

// This is a test as to whether it's more efficient to preallocate the return array
#[divan::bench]
fn stdin_stdout(bencher: Bencher) {
    let n = N;
    let mut pipe_runner = ipc::pipes::PipeRunner::new(false);
    let mut return_buffer = pipe_runner.prepare();
    bencher
        .counter(divan::counter::ItemsCount::new(n))
        .bench_local(move || pipe_runner.run_inner(n, &mut return_buffer));
}

#[divan::bench]
fn tcp_nodelay(bencher: Bencher) {
    let n = N;
    let mut tcp_runner = ipc::tcp::TcpRunner::new(true, true);
    bencher
        .counter(divan::counter::ItemsCount::new(n))
        .bench_local(move || {
            tcp_runner.run(n, false);
        });
}

#[divan::bench]
fn tcp_yesdelay(bencher: Bencher) {
    let n = N;
    let mut tcp_runner = ipc::tcp::TcpRunner::new(true, false);
    bencher
        .counter(divan::counter::ItemsCount::new(n))
        .bench_local(move || {
            tcp_runner.run(n, false);
        });
}

#[divan::bench]
fn udp(bencher: Bencher) {
    let n = N;
    let mut udp_runner = ipc::udp::UdpRunner::new(true);
    bencher
        .counter(divan::counter::ItemsCount::new(n))
        .bench_local(move || {
            udp_runner.run(n, false);
        });
}

#[divan::bench]
fn shared_memory(bencher: Bencher) {
    let n = N;
    let mut shmem_runner = ipc::shmem::ShmemRunner::new(true);
    bencher
        .counter(divan::counter::ItemsCount::new(n))
        .bench_local(move || {
            shmem_runner.run(n, false);
        });
}

#[divan::bench]
fn memory_mapped_file(bencher: Bencher) {
    let n = N;
    let mut mmap_runner = ipc::mmap::MmapRunner::new(true);
    bencher
        .counter(divan::counter::ItemsCount::new(n))
        .bench_local(move || {
            mmap_runner.run(n, false);
        });
}

#[divan::bench]
fn unix_stream(bencher: Bencher) {
    let n = N;
    let mut unix_tcp_runner = ipc::unix_stream::UnixStreamRunner::new(true);
    bencher
        .counter(divan::counter::ItemsCount::new(n))
        .bench_local(move || {
            unix_tcp_runner.run(n, false);
        });
}

#[divan::bench]
fn unix_datagram(bencher: Bencher) {
    let n = N;
    let mut unix_udp_runner = ipc::unix_datagram::UnixDatagramRunner::new(true);
    bencher
        .counter(divan::counter::ItemsCount::new(n))
        .bench_local(move || {
            unix_udp_runner.run(n, false);
        });
}

#[divan::bench]
fn iceoryx(bencher: Bencher) {
    let n = N;
    let mut unix_udp_runner = ipc::iceoryx::IceoryxRunner::new(false);
    bencher
        .counter(divan::counter::ItemsCount::new(n))
        .bench_local(move || {
            unix_udp_runner.run(n, false);
        });
}
