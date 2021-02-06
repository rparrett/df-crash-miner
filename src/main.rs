use crate::files::*;
use crate::gen::*;
use anyhow::Result;

#[macro_use]
extern crate anyhow;

mod files;
mod gen;

#[tokio::main]
async fn main() -> Result<()> {
    let latest = get_latest(false).await?;
    // ensure_worker_dirs(8)?;
    // ensure_output_dir()?;

    let mut handles = vec![];
    for n in 0..4 {
        handles.push(tokio::spawn(async move {
            loop {
                let f = gen_world(format!("{}", n), "CRASH".to_string());
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
