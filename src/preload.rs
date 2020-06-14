use anyhow::*;
use directories_next::BaseDirs;
use regex::Regex;
use std::path::PathBuf;
use std::process::Command;

fn get_cache_dir() -> Result<PathBuf> {
    let mut dir = BaseDirs::new()
        .context("could not locate system directories")?
        .cache_dir()
        .to_path_buf();
    dir.push("level-99");
    Ok(dir)
}

fn url_to_path(url: &str) -> Result<PathBuf> {
    let mut path = get_cache_dir()?;
    let video_id_regex = Regex::new("v=([^&]+)")?; // TODO Don't compile regex this on every call
    for captures in video_id_regex.captures_iter(&url) {
        let id = captures[1].to_owned();
        path.push(id);
        return Ok(path);
    }
    Err(anyhow!("No video ID in URL"))
}

pub fn preload_songs(urls: &Vec<String>) -> Result<()> {
    let mut output_template = get_cache_dir()?;
    output_template.push("%(id)s");
    let output_template = output_template.to_string_lossy();

    let mut ytdl_args = vec![
        "-f",
        "webm[abr>0]/bestaudio/best",
        "--no-playlist",
        "--ignore-config",
        "-o",
        output_template.as_ref(),
    ];
    let mut args = urls.iter().map(|s| s.as_str()).collect::<Vec<&str>>();
    ytdl_args.append(&mut args);

    if let Err(e) = Command::new("youtube-dl").args(&ytdl_args).spawn() {
        eprintln!("Could not spawn youtube-dl process for preloading: {}", e);
    }

    Ok(())
}

pub fn retrieve_song(url: &str) -> Option<PathBuf> {
    let path = url_to_path(url).ok();
    if let Some(path) = path {
        if path.exists() {
            return Some(path);
        }
    }
    eprintln!("Preload song cache miss: {}", url);
    None
}
