use crate::files::*;
use crate::gen::*;
use anyhow::Result;
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
    #[structopt(short, long, default_value = "4")]
    concurrency: usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    let opt = Opt::from_args();

    ensure_dirs()?;

    let _ = get_latest(false).await?;

    ensure_worker_dirs(opt.concurrency, false)?;

    let mut handles = vec![];
    for n in 0..opt.concurrency {
        handles.push(tokio::spawn(async move {
            loop {
                let f = gen_world(format!("{}", n), "long_history_pocket.txt".to_string());
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

    Ok(())
}
