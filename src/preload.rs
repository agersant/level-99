use anyhow::*;
use directories_next::BaseDirs;
use lazy_static::lazy_static;
use parking_lot::RwLock;
use regex::Regex;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

#[derive(Clone)]
pub struct CacheEntry {
    pub path: PathBuf,
    pub start_at: Duration,
}

lazy_static! {
    static ref CACHE: RwLock<HashMap<String, CacheEntry>> = { RwLock::new(HashMap::new()) };
}

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

fn url_to_start_time(url: &str) -> Result<Duration> {
    let timestamp_regex = Regex::new("t=([0-9]+)").unwrap(); // TODO Don't compile regex this on every call
    for captures in timestamp_regex.captures_iter(&url) {
        if let Ok(seconds) = captures[1].to_owned().parse::<u64>() {
            return Ok(Duration::from_secs(seconds));
        }
    }
    return Ok(Duration::from_secs(0));
}

pub fn preload_songs(urls: &Vec<String>) -> Result<()> {
    for url in urls {
        let path = url_to_path(url);
        let start_time = url_to_start_time(url);
        if let (Ok(path), Ok(start_at)) = (path, start_time) {
            let cache_entry = CacheEntry { path, start_at };
            let mut cache = CACHE.write();
            cache.insert(url.clone(), cache_entry);
        }
    }

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

pub fn retrieve_song(url: &str) -> Option<CacheEntry> {
    let cache = CACHE.read();
    let cache_entry = cache.get(url);
    if let Some(cache_entry) = cache_entry {
        if cache_entry.path.exists() {
            return Some(cache_entry.clone());
        }
    }
    eprintln!("Preload song cache miss: {}", url);
    None
}
