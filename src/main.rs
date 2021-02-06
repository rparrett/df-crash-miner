use crate::files::*;
use crate::gen::*;
use anyhow::Result;

#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate lazy_static;

mod files;
mod gen;
mod util;

#[tokio::main]
async fn main() -> Result<()> {
    ensure_dirs()?;

    let _ = get_latest(false).await?;

    let concurrency = 4;

    ensure_worker_dirs(concurrency, false)?;

    let mut handles = vec![];
    for n in 0..concurrency {
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
