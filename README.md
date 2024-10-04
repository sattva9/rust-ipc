# rust-ipc

This is a small proof of concept project for different approaches to Interprocess Communication in Rust.

This has been built upon the work by [3tilley](https://github.com/3tilley/rust-experiments/tree/master/ipc).

It accompanies a blog post [here](https://pranitha.rs/posts/rust-ipc-ping-pong/).

## Usage

To demo IPC, run the below, choosing a method from `tcp`, `udp`, `shmem`, `stdout`, `iceoryx`, `mmap`, `unixdatagram`, `unixstream`.

`cargo run --release -- -n 1000 --method stdout`

```bash
$ cargo build --release
$ cargo run --release -- -n 1000 --method mmap
    Finished release [optimized] target(s) in 0.06s
     Running `target/release/ipc -n 1000 --method mmap`
IPC method - Memory mapped file
	1000 cycles completed in 172us 459ns
	5813953.5 per second
	172ns per operation
```

If you want to run the benchmarks, run:

`cargo bench`

Note:
1. In the Divan output, the time per function will be displayed for the total number of cycles, but the throughput will be displayed per cycle. So to get timing per cycle, do t/N where N=1000(configurable).
2. Because the host process picks out an executable from the targets directory for the consumer, if you make changes to the consumers run `cargo build --release` to make sure they are reflected in the next execution. By default `cargo run` will only rebuild the `ipc` binrary, which only holds the producer code.

## License

[MIT](https://choosealicense.com/licenses/mit/)
