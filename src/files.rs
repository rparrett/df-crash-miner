use crate::util;
use anyhow::Result;
use bzip2::read::BzDecoder;
use dirs::home_dir;
use regex::Regex;
use std::fs;
use std::io;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tar::Archive;
use zip_extensions::read::ZipArchiveExtensions;

pub fn ensure_worker_dirs(num: usize, force: bool) -> Result<()> {
    for n in 0..num {
        let dir = PathBuf::from(base_dir()?.join(format!("{}", n)));
        if !dir.is_dir() || force {
            extract(format!("{}", n))?;
        }
    }

    Ok(())
}

pub fn ensure_dirs() -> Result<()> {
    // TODO the most likely error is that these directories already
    // exist, which is okay to ignore. But we should not be ignoring
    // other errors.

    let _ = fs::create_dir_all(base_dir()?.join("templates"));
    let _ = fs::create_dir_all(base_dir()?.join("crashes"));

    Ok(())
}

fn extract(prefix: String) -> Result<()> {
    let path = archive_path()?;

    if cfg!(target_os = "windows") {
        // windows df archives come without a root folder

        let zip = fs::File::open(&path)?;
        let mut archive = zip::ZipArchive::new(zip)?;
        let target = base_dir()?.join(&prefix);
        archive.extract(&target)?;
    } else {
        let tar_bz = fs::File::open(&path)?;
        let tar = BzDecoder::new(tar_bz);
        let mut archive = Archive::new(tar);

        // macos/linux df archives contain a root folder that we want to rename
        let old_prefix = if cfg!(target_os = "macos") {
            "df_osx"
        } else if cfg!(target_os = "linux") {
            "df_linux"
        } else {
            panic!()
        };

        for entry in archive.entries()? {
            let mut file = entry?;

            let mut path = base_dir()?;
            path.push(prefix.as_str());
            path.push(file.path()?.strip_prefix(old_prefix)?.to_owned());
            file.unpack(&path)?;
        }
    }

    patches(&prefix)?;

    Ok(())
}

pub fn patches(prefix: &String) -> Result<()> {
    // fairly sure the only patch we're doing is only needed on macos
    if cfg!(not(target_os = "macos")) {
        return Ok(());
    }

    let init_lines = util::read_lines(
        base_dir()?
            .join(&prefix)
            .join("data")
            .join("init")
            .join("init.txt"),
    )?
    .collect::<Vec<_>>();

    let mut init_file = fs::File::create(
        base_dir()?
            .join(&prefix)
            .join("data")
            .join("init")
            .join("init.txt"),
    )?;

    for line in init_lines {
        if let Ok(line) = line {
            if line.contains("[PRINT_MODE:2D]") {
                writeln!(init_file, "[PRINT_MODE:STANDARD]")?;
            } else {
                writeln!(init_file, "{}", line)?;
            }
        }
    }

    Ok(())
}

pub async fn get_latest(force: bool) -> Result<()> {
    let archive = archive_path()?;

    if !archive.is_file() || force {
        download_latest(&archive).await?;
    }

    Ok(())
}

pub fn base_dir() -> Result<PathBuf> {
    let dir = home_dir();
    if !dir.is_some() {
        bail!("couldn't get home dir");
    }
    let mut dir = dir.unwrap();
    dir.push("df-crash-miner");
    Ok(dir)
}

pub fn archive_path() -> Result<PathBuf> {
    let mut path = base_dir()?;

    if cfg!(target_os = "linux") {
        path.push("current.tar.bz2")
    } else if cfg!(target_os = "macos") {
        path.push("current.tar.bz2")
    } else if cfg!(target_os = "windows") {
        path.push("current.zip")
    } else {
        panic!()
    }

    Ok(path)
}

async fn download_latest(archive: &Path) -> Result<()> {
    let body = reqwest::get("http://www.bay12games.com/dwarves/older_versions.html")
        .await?
        .text()
        .await?;

    let reg = if cfg!(target_os = "linux") {
        Regex::new(r"df_([\d_]+)_linux.tar.bz2").unwrap()
    } else if cfg!(target_os = "macos") {
        Regex::new(r"df_([\d_]+)_osx.tar.bz2").unwrap()
    } else if cfg!(target_os = "windows") {
        Regex::new(r"df_([\d_]+)_win.zip").unwrap()
    } else {
        panic!();
    };

    let cap = reg.captures_iter(&body).next();
    if !cap.is_some() {
        bail!("no matchy");
    }
    let cap = cap.unwrap();

    let url = ["http://www.bay12games.com/dwarves/", &cap[0]].concat();

    let response = reqwest::get(&url).await?;

    let mut dest = fs::File::create(archive)?;

    let bytes = response.bytes().await?;

    dest.write(&bytes)?;

    Ok(())
}
