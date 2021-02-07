use crate::files::*;
use crate::gen::*;
use anyhow::Result;
use std::path::PathBuf;
use structopt::StructOpt;

#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate lazy_static;

mod files;
mod gen;
mod util;

#[derive(StructOpt, Debug)]
#[structopt(name = "Dwarf Fortress Crash Miner")]
struct Opt {
    /// Number of world gens to run simultaneously
    #[structopt(short, long, default_value = "4")]
    concurrency: usize,
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt, Debug)]
enum Command {
    Crash {
        /// World gen params file
        #[structopt(short, long, parse(from_os_str))]
        params: PathBuf,
    },
    Repro,
}

#[tokio::main]
async fn main() -> Result<()> {
    let opt = Opt::from_args();

    ensure_dirs()?;

    let _ = get_latest(false).await?;

    ensure_worker_dirs(opt.concurrency, false)?;

    match opt.cmd {
        Command::Crash { params } => {
            let mut handles = vec![];
            for n in 0..opt.concurrency {
                let params = params.clone();

                handles.push(tokio::spawn(async move {
                    loop {
                        let f = gen_world(format!("{}", n), &params);
                        tokio::select! {
                            res = f => {
                                println!("{:?}", res);
                            }
                            _ = tokio::signal::ctrl_c() => { println!("ctrlc!"); break }
                        }
                    }
                }));
            }

            futures::future::join_all(handles).await;
        }
        Command::Repro => {
            unimplemented!();
        }
    }

    Ok(())
}
