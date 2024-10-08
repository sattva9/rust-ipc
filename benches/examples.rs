use divan::Bencher;
use ipc::cpu_warmup;

// This affects the number cycles of to execute each method for. In the Divan output, the
// time per function will be displayed for the total number of cycles, but the throughput
// will be displayed per cycle. So to get timing per cycle, do t/N. A workaround for this
// hopefully will be found
const N: usize = 1000;
const KB: usize = 1024;
const LENS: &[usize] = &[1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024];

fn main() {
    divan::main();
}

#[divan::bench(args = LENS)]
fn stdin_stdout(bencher: Bencher, data_size: usize) {
    let n = N;
    let mut pipe_runner = ipc::pipes::PipeRunner::new(data_size * KB);

    core_affinity::set_for_current(core_affinity::CoreId { id: 1 });
    cpu_warmup();

    bencher
        .counter(n)
        .bench_local(move || pipe_runner.run(n, false));
}

#[divan::bench(args = LENS)]
fn tcp_nodelay(bencher: Bencher, data_size: usize) {
    let n = N;
    let mut tcp_runner = ipc::tcp::TcpRunner::new(true, true, data_size * KB);

    core_affinity::set_for_current(core_affinity::CoreId { id: 1 });
    cpu_warmup();

    bencher
        .counter(divan::counter::ItemsCount::new(n))
        .bench_local(move || {
            tcp_runner.run(n, false);
        });
}

#[divan::bench(args = LENS)]
fn tcp_yesdelay(bencher: Bencher, data_size: usize) {
    let n = N;
    let mut tcp_runner = ipc::tcp::TcpRunner::new(true, false, data_size * KB);

    core_affinity::set_for_current(core_affinity::CoreId { id: 1 });
    cpu_warmup();

    bencher
        .counter(divan::counter::ItemsCount::new(n))
        .bench_local(move || {
            tcp_runner.run(n, false);
        });
}

#[divan::bench(args = LENS)]
fn udp(bencher: Bencher, data_size: usize) {
    let n = N;
    let mut udp_runner = ipc::udp::UdpRunner::new(true, data_size * KB);

    core_affinity::set_for_current(core_affinity::CoreId { id: 1 });
    cpu_warmup();

    bencher
        .counter(divan::counter::ItemsCount::new(n))
        .bench_local(move || {
            udp_runner.run(n, false);
        });
}

#[divan::bench(args = LENS)]
fn shared_memory(bencher: Bencher, data_size: usize) {
    let n = N;
    let mut shmem_runner = ipc::shmem::ShmemRunner::new(true, data_size * KB);

    core_affinity::set_for_current(core_affinity::CoreId { id: 1 });
    cpu_warmup();

    bencher
        .counter(divan::counter::ItemsCount::new(n))
        .bench_local(move || {
            shmem_runner.run(n, false);
        });
}

#[divan::bench(args = LENS)]
fn memory_mapped_file(bencher: Bencher, data_size: usize) {
    let n = N;
    let mut mmap_runner = ipc::mmap::MmapRunner::new(true, data_size * KB);

    core_affinity::set_for_current(core_affinity::CoreId { id: 1 });
    cpu_warmup();

    bencher
        .counter(divan::counter::ItemsCount::new(n))
        .bench_local(move || {
            mmap_runner.run(n, false);
        });
}

#[divan::bench(args = LENS)]
fn unix_stream(bencher: Bencher, data_size: usize) {
    let n = N;
    let mut unix_tcp_runner = ipc::unix_stream::UnixStreamRunner::new(true, data_size * KB);

    core_affinity::set_for_current(core_affinity::CoreId { id: 1 });
    cpu_warmup();

    bencher
        .counter(divan::counter::ItemsCount::new(n))
        .bench_local(move || {
            unix_tcp_runner.run(n, false);
        });
}

#[divan::bench(args = LENS)]
fn unix_datagram(bencher: Bencher, data_size: usize) {
    let n = N;
    let mut unix_udp_runner = ipc::unix_datagram::UnixDatagramRunner::new(true, data_size * KB);

    core_affinity::set_for_current(core_affinity::CoreId { id: 1 });
    cpu_warmup();

    bencher
        .counter(divan::counter::ItemsCount::new(n))
        .bench_local(move || {
            unix_udp_runner.run(n, false);
        });
}

#[divan::bench(args = LENS)]
fn iceoryx(bencher: Bencher, data_size: usize) {
    let n = N;
    let mut unix_udp_runner = ipc::iceoryx::IceoryxRunner::new(true, data_size * KB);

    core_affinity::set_for_current(core_affinity::CoreId { id: 1 });
    cpu_warmup();

    bencher
        .counter(divan::counter::ItemsCount::new(n))
        .bench_local(move || {
            unix_udp_runner.run(n, false);
        });
}
