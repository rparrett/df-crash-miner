use crate::files::base_dir;
use crate::util;
use anyhow::Result;
use chrono::Utc;
use glob::glob;
use regex::Regex;
use std::fmt;
use std::fs;
use std::io::prelude::*;
use std::path::PathBuf;
use tokio::process::Command;

#[derive(Debug)]
pub struct WorldGenResults {
    pub result: WorldGenResult,
    pub seeds: WorldGenSeeds,
}

#[derive(Debug)]
pub enum WorldGenResult {
    Abort,
    Crash,
    Success,
}

#[derive(Debug, Clone)]
pub struct WorldGenSeeds {
    seed: String,
    history_seed: String,
    name_seed: String,
    creature_seed: String,
}

impl fmt::Display for WorldGenResults {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?} [{} / {} / {} / {}]",
            self.result,
            self.seeds.seed,
            self.seeds.history_seed,
            self.seeds.name_seed,
            self.seeds.creature_seed
        )
    }
}

lazy_static! {
    static ref SEEDS_RE: Regex = Regex::new(
        r"(?m)Seed: (\w+)\s*History Seed: (\w+)\s* Name Seed: (\w+)\s*Creature Seed: (\w+)"
    )
    .unwrap();
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

    let cap = SEEDS_RE.captures_iter(&gamelog).next();
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

    Ok(WorldGenResults { seeds, result })
}

pub fn log_crash(worker: &String, seeds: &WorldGenSeeds) -> Result<()> {
    let platform = if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "unknown"
    };

    let template_lines = util::read_lines(
        base_dir()?
            .join(worker)
            .join("data")
            .join("init")
            .join("world_gen.txt"),
    )?;
    let time = Utc::now().format("%Y%m%d-%H%M%S-%f");
    let crash_file_name = format!("{}-{}.txt", platform, time);
    let mut crash_file = fs::File::create(base_dir()?.join("crashes").join(crash_file_name))?;

    for (i, line) in template_lines.enumerate() {
        if let Ok(line) = line {
            if i == 2 {
                writeln!(crash_file, "\t[SEED:{}]", seeds.seed)?;
                writeln!(crash_file, "\t[HISTORY_SEED:{}]", seeds.history_seed)?;
                writeln!(crash_file, "\t[NAME_SEED:{}]", seeds.name_seed)?;
                writeln!(crash_file, "\t[CREATURE_SEED:{}]", seeds.creature_seed)?;
            }

            writeln!(crash_file, "{}", line)?;
        }
    }

    Ok(())
}

pub async fn gen_world(
    worker: String,
    params: &PathBuf,
    log_crashes: bool,
) -> Result<WorldGenResults> {
    cleanup(&worker)?;

    fs::copy(
        base_dir()?.join(params),
        base_dir()?
            .join(&worker)
            .join("data")
            .join("init")
            .join("world_gen.txt"),
    )?;

    // TODO command is different on windows

    let cmd = if cfg!(target_os = "windows") {
        "Dwarf Fortress.exe"
    } else {
        "df"
    };

    let _ = Command::new(base_dir()?.join(&worker).join(cmd))
        .current_dir(base_dir()?.join(&worker))
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

    if log_crashes {
        if let Ok(r) = &res {
            match r.result {
                WorldGenResult::Crash => log_crash(&worker, &r.seeds)?,
                _ => {}
            }
        }
    }

    res
}
