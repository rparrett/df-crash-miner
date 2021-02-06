use crate::files::base_dir;
use anyhow::Result;
use glob::glob;
use std::fs;
use tokio::process::Command;
use regex::Regex;

#[derive(Debug)]
pub struct WorldGenResults {
    result: WorldGenResult,
    seeds: WorldGenSeeds,
}

#[derive(Debug)]
pub enum WorldGenResult {
    Abort,
    Crash,
    Success
}

#[derive(Debug)]
pub struct WorldGenSeeds {
    seed: String,
    history_seed: String,
    name_seed: String,
    creature_seed: String,
}

fn cleanup(worker: &String) -> Result<()> {
    for path in glob(
        base_dir()?
            .join(worker.clone())
            .join("region*")
            .to_str()
            .unwrap(),
    )?
    .filter_map(Result::ok)
    {
        let _ = fs::remove_file(&path);
    }

    for path in glob(
        base_dir()?
            .join(worker.clone())
            .join("data")
            .join("save")
            .join("region*")
            .to_str()
            .unwrap(),
    )?
    .filter_map(Result::ok)
    {
        let _ = fs::remove_dir_all(&path);
    }

    let _ = fs::remove_file(base_dir()?.join(worker.clone()).join("gamelog.txt"));
    let _ = fs::remove_file(base_dir()?.join(worker.clone()).join("errorlog.txt"));

    Ok(())
}

pub fn get_gen_results(worker: &String) -> Result<WorldGenResults> {
    let gamelog = fs::read_to_string(base_dir()?.join(worker).join("gamelog.txt"))?;
        
    let reg = Regex::new(r"(?m)Seed: (\w+)\s*History Seed: (\w+)\s* Name Seed: (\w+)\s*Creature Seed: (\w+)").unwrap();

    let cap = reg.captures_iter(&gamelog).next();
    if cap.is_none() {
        bail!("no seeds in gamelog");
    }
    let cap = cap.unwrap();

    let seeds = WorldGenSeeds {
        seed: cap[1].to_string(),
        history_seed: cap[2].to_string(),
        name_seed: cap[3].to_string(),
        creature_seed: cap[4].to_string(),
    };

    let result = if gamelog.contains(&"World exported") {
        WorldGenResult::Success
    } else if gamelog.contains(&"aborted") {
        WorldGenResult::Abort
    } else {
        WorldGenResult::Crash
    };

    Ok(WorldGenResults {
        seeds,
        result
    })
}

pub async fn gen_world(worker: String, template: String) -> Result<()> {
    cleanup(&worker)?;

    let _ = Command::new(base_dir()?.join(worker.clone()).join("df"))
        .current_dir(base_dir()?.join(worker.clone()))
        .arg("-gen")
        .arg("0")
        .arg("RANDOM")
        .arg("CRASH")
        .output()
        .await?;

    // stdout and error code and friends seem unreliable for determining
    // if we crashed.
    //
    // instead, we'll read from gamelog.txt

    let res = get_gen_results(&worker);

    println!("{:?}", res);

    Ok(())
}
