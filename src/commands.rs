use structopt::StructOpt;
use std::net::SocketAddr;

#[derive(Debug, StructOpt)]
struct Commands {
    #[structopt(short, long)]
    type: ProgramType,
}

#[derive(Debug, StructOpt)]
enum ProgramType {
    #[structopt(short, long)]
    Server {
        data: String,
        addr: SocketAddr,
        reduce_tasks: u32,
    },
    #[structopt(short, long)]
    Worker {
        coordinator_addr: String
    }
}
