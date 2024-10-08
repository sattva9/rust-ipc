use clap::Parser;
use ipc::iceoryx::IceoryxRunner;
use ipc::mmap::MmapRunner;
use ipc::pipes::PipeRunner;
use ipc::shmem::ShmemRunner;
use ipc::tcp::TcpRunner;
use ipc::udp::UdpRunner;
use ipc::unix_datagram::UnixDatagramRunner;
use ipc::unix_stream::UnixStreamRunner;
use ipc::{cpu_warmup, KB};

fn main() {
    let args = Cli::parse();
    match args.method {
        Method::Stdout => {
            for data_size in 1..=args.kb_max {
                let data_size = 2u64.pow(data_size as u32) as usize * KB;
                let mut pr = PipeRunner::new(data_size);

                core_affinity::set_for_current(core_affinity::CoreId { id: 1 });
                cpu_warmup();

                pr.run(args.number, true);
            }
        }
        Method::Shmem => {
            for data_size in 1..=args.kb_max {
                let data_size = 2u64.pow(data_size as u32) as usize * KB;
                let mut runner = ShmemRunner::new(args.start_child, data_size);

                core_affinity::set_for_current(core_affinity::CoreId { id: 1 });
                cpu_warmup();

                runner.run(args.number, true);
            }
        }
        Method::Tcp => {
            for data_size in 1..=args.kb_max {
                let data_size = 2u64.pow(data_size as u32) as usize * KB;
                let mut runner = TcpRunner::new(args.start_child, true, data_size);

                core_affinity::set_for_current(core_affinity::CoreId { id: 1 });
                cpu_warmup();

                runner.run(args.number, true);
            }
        }
        Method::Udp => {
            for data_size in 1..=args.kb_max {
                let data_size = 2u64.pow(data_size as u32) as usize * KB;
                let mut runner = UdpRunner::new(true, data_size);

                core_affinity::set_for_current(core_affinity::CoreId { id: 1 });
                cpu_warmup();

                runner.run(args.number, true);
                drop(runner);
            }
        }
        Method::Iceoryx => {
            for data_size in 1..=args.kb_max {
                let data_size = 2u64.pow(data_size as u32) as usize * KB;
                let mut runner = IceoryxRunner::new(true, data_size);

                core_affinity::set_for_current(core_affinity::CoreId { id: 1 });
                cpu_warmup();

                runner.run(args.number, true);
            }
        }
        Method::Mmap => {
            for data_size in 1..=args.kb_max {
                let data_size = 2u64.pow(data_size as u32) as usize * KB;
                let mut runner = MmapRunner::new(true, data_size);

                core_affinity::set_for_current(core_affinity::CoreId { id: 1 });
                cpu_warmup();

                runner.run(args.number, true);
            }
        }
        Method::Unixstream => {
            for data_size in 1..=args.kb_max {
                let data_size = 2u64.pow(data_size as u32) as usize * KB;
                let mut runner = UnixStreamRunner::new(true, data_size);

                core_affinity::set_for_current(core_affinity::CoreId { id: 1 });
                cpu_warmup();

                runner.run(args.number, true);
            }
        }
        Method::Unixdatagram => {
            for data_size in 1..=args.kb_max {
                let data_size = 2u64.pow(data_size as u32) as usize * KB;
                let mut runner = UnixDatagramRunner::new(true, data_size);

                core_affinity::set_for_current(core_affinity::CoreId { id: 1 });
                cpu_warmup();

                runner.run(args.number, true);
            }
        }
    }
}

#[derive(Debug, Default, Copy, Clone, clap::ValueEnum)]
enum Method {
    #[default]
    Stdout,
    Shmem,
    Tcp,
    Udp,
    Iceoryx,
    Mmap,
    Unixstream,
    Unixdatagram,
}

#[derive(Parser, Debug)]
struct Cli {
    #[arg(short, long)]
    number: usize,

    #[clap(short, long, default_value_t, value_enum)]
    method: Method,

    #[arg(short, long, action, default_value_t = true)]
    start_child: bool,

    #[arg(short, long, action, default_value_t = 10)]
    kb_max: usize,
}
