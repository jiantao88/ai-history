use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::model::Session;
use super::SessionDigest;

const CACHE_VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
struct CacheEntry {
    version: u32,
    source_mtime: u64,
    source_size: u64,
    digest: SessionDigest,
}

fn cache_dir() -> Option<PathBuf> {
    dirs::cache_dir().map(|d| d.join("ai-history").join("digests"))
}

fn cache_path(session: &Session) -> Option<PathBuf> {
    cache_dir().map(|d| d.join(format!("{}.digest.json", session.id)))
}

pub fn load_cached_digest(session: &Session) -> Option<SessionDigest> {
    let path = cache_path(session)?;
    let content = fs::read_to_string(&path).ok()?;
    let cached: CacheEntry = serde_json::from_str(&content).ok()?;

    if cached.version != CACHE_VERSION {
        return None;
    }

    let source_meta = fs::metadata(&session.file_path).ok()?;
    let source_mtime = source_meta
        .modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();
    let source_size = source_meta.len();

    if cached.source_mtime != source_mtime || cached.source_size != source_size {
        return None;
    }

    Some(cached.digest)
}

pub fn save_digest(session: &Session, digest: &SessionDigest) -> Result<()> {
    let dir = cache_dir().ok_or_else(|| anyhow::anyhow!("Cannot determine cache directory"))?;
    fs::create_dir_all(&dir)?;

    let path = dir.join(format!("{}.digest.json", session.id));

    let source_meta = fs::metadata(&session.file_path)?;
    let source_mtime = source_meta
        .modified()?
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();
    let source_size = source_meta.len();

    let entry = CacheEntry {
        version: CACHE_VERSION,
        source_mtime,
        source_size,
        digest: digest.clone(),
    };

    let json = serde_json::to_string_pretty(&entry)?;

    // Atomic write: temp file + rename
    let tmp_path = dir.join(format!("{}.digest.tmp", session.id));
    fs::write(&tmp_path, &json)?;
    fs::rename(&tmp_path, &path)?;

    Ok(())
}
