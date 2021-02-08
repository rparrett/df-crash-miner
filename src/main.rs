use crate::files::*;
use crate::gen::*;
use anyhow::Result;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_BORDERS_ONLY;
use comfy_table::Table;
use crossbeam_queue::ArrayQueue;
use dashmap::DashMap;
use glob::glob;
use std::path::PathBuf;
use std::sync::Arc;
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
    /// Discover new crashes
    Crash {
        /// World gen params file
        #[structopt(short, long, parse(from_os_str))]
        params: PathBuf,
    },
    /// Reproduce crashes with param files from the crashes directory
    Repro {
        /// Number of times to re-run each world gen
        #[structopt(short, long, default_value = "4")]
        num: usize,
    },
    /// Download the latest version of Dwarf Fortress
    Update,
}

#[tokio::main]
async fn main() -> Result<()> {
    let opt = Opt::from_args();

    ensure_dirs()?;

    match opt.cmd {
        Command::Update => {
            let _ = get_latest(false).await?;
            ensure_worker_dirs(opt.concurrency, false)?;
        }
        Command::Crash { params } => {
            ensure_worker_dirs(opt.concurrency, false)?;

            if files::base_dir().map_or(false, |base| !base.join("params").join(&params).is_file())
            {
                println!("Invalid params file.");
                return Ok(());
            }

            let mut handles = vec![];
            for n in 0..opt.concurrency {
                let params = PathBuf::from("params").join(&params);

                handles.push(tokio::spawn(async move {
                    loop {
                        let f = gen_world(format!("{}", n), &params, true);
                        tokio::select! {
                            res = f => {
                                match res {
                                    Ok(r) => {
                                        println!("{}", r);
                                    },
                                    Err(e) => {
                                        println!("Error generating world.");
                                        println!("{}", e);
                                    }
                                }
                            }
                            _ = tokio::signal::ctrl_c() => { println!("Worker caught ctr-c. Quitting."); break }
                        }
                    }
                }));
            }

            futures::future::join_all(handles).await;
        }
        Command::Repro { num } => {
            ensure_worker_dirs(opt.concurrency, false)?;

            let paths: Vec<_> = glob(
                files::base_dir()?
                    .join("crashes")
                    .join("*.txt")
                    .to_str()
                    .unwrap(),
            )?
            .filter_map(Result::ok)
            .collect();

            if paths.is_empty() {
                bail!("No crashes in the crashes directory.")
            }

            let queue = ArrayQueue::new(paths.len() * num);

            for path in paths {
                for _ in 0..num {
                    let _ = queue.push(path.clone()).unwrap();
                }
            }

            let queue_arc = Arc::new(queue);

            let repro_stats = Arc::new(DashMap::<PathBuf, (u32, u32)>::new());

            let mut handles = vec![];
            for n in 0..opt.concurrency {
                let queue_ours = queue_arc.clone();
                let repro_stats_ours = repro_stats.clone();

                handles.push(tokio::spawn(async move {
                    loop {
                        let params = queue_ours.pop();
                        let param = match params {
                            Some(p) => p,
                            None => break,
                        };

                        let f = gen_world(format!("{}", n), &param, false);

                        tokio::select! {
                            res = f => {
                                match res {
                                    Ok(r) => {
                                        println!("{}", r);
                                        match r.result {
                                            WorldGenResult::Crash => {
                                                let mut e = repro_stats_ours.entry(param).or_insert((0, 0));
                                                (*e).0 += 1
                                            },
                                            WorldGenResult::Success => {
                                                let mut e = repro_stats_ours.entry(param).or_insert((0, 0));
                                                (*e).1 += 1
                                            },
                                            _ => {}

                                        }

                                    },
                                    Err(e) => {
                                        println!("Error generating world.");
                                        println!("{}", e);
                                    }
                                }
                            }
                            _ = tokio::signal::ctrl_c() => { println!("Worker caught ctr-c. Quitting."); break }
                        }
                    }
                }));
            }

            futures::future::join_all(handles).await;

            let mut table = Table::new();
            table.set_header(vec!["Params", "Crash", "Success"]);
            table.load_preset(UTF8_BORDERS_ONLY);
            table.apply_modifier(UTF8_ROUND_CORNERS);

            for k in repro_stats.iter() {
                let (k, v) = k.pair();
                table.add_row(vec![
                    format!("{}", k.file_name().unwrap().to_string_lossy()),
                    format!("{}", v.0),
                    format!("{}", v.1),
                ]);
            }

            println!("{}", table);
        }
    }

    Ok(())
}
