use std::net::SocketAddr;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Commands {
    #[structopt(subcommand)]
    pub program_type: ProgramType,
}

#[derive(Debug, StructOpt)]
pub enum ProgramType {
    Server {
        #[structopt(short, long)]
        path: String,
        #[structopt(short, long)]
        addr: SocketAddr,
        #[structopt(short, long)]
        reduce_tasks: u32,
        #[structopt(short, long)]
        dead_task_delta: u32,
    },
    Worker {
        #[structopt(short, long)]
        coordinator_addr: String,
    },
}

#[macro_export]
macro_rules! map_reduce {
    ($worker:ty) => {
        use rmr::cli::{Commands, ProgramType};
        use rmr::coordinator::MRCoordinator;
        use rmr::worker::MRWorker;
        use structopt::StructOpt;
        #[tokio::main]
        async fn main() -> Result<(), Box<dyn std::error::Error>> {
            let command = Commands::from_args();
            match command.program_type {
                ProgramType::Server {
                    path,
                    addr,
                    reduce_tasks,
                    dead_task_delta,
                } => {
                    MRCoordinator::new(path, reduce_tasks, dead_task_delta, addr)
                        .run()
                        .await?
                }
                ProgramType::Worker { coordinator_addr } => {
                    MRWorker::<$worker>::new(coordinator_addr)
                        .await?
                        .run()
                        .await?
                }
            }
            Ok(())
        }
    };
}
