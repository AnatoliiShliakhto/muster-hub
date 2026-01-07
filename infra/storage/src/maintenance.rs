use std::path::Path;
use std::time::{Duration, SystemTime};
use tracing::{error, info};
use walkdir::{DirEntry, WalkDir};

pub(crate) async fn purge_tmp(root: &Path) {
    let root = root.to_path_buf();
    let now = SystemTime::now();
    let threshold = Duration::from_secs(300);

    match tokio::task::spawn_blocking(move || remove_stale(&root, now, threshold)).await {
        Ok((removed, failed)) if removed > 0 || failed > 0 => {
            info!(removed, failed, "Cleaned up temporary files");
        },
        Err(e) => {
            error!(error = %e, "Temp file cleanup task panicked");
        },
        _ => {},
    }
}

fn remove_stale(root: &Path, now: SystemTime, threshold: Duration) -> (usize, usize) {
    let mut removed = 0;
    let mut failed = 0;

    WalkDir::new(root)
        .contents_first(true)
        .into_iter()
        .flatten()
        .filter(|e| e.path() != root)
        .for_each(|entry| {
            let path = entry.path();

            if entry.file_type().is_file() {
                if is_tmp(&entry) && is_stale(&entry, now, threshold) {
                    match std::fs::remove_file(path) {
                        Ok(()) => removed += 1,
                        Err(e) => {
                            tracing::warn!(p = %path.display(), err = %e, "IO fail");
                            failed += 1;
                        },
                    }
                }
            } else if entry.file_type().is_dir() {
                let _ = std::fs::remove_dir(path);
            }
        });

    (removed, failed)
}

fn is_tmp(entry: &DirEntry) -> bool {
    if !entry.file_type().is_file() {
        return false;
    }
    entry
        .path()
        .file_name()
        .and_then(|name| name.to_str())
        .map_or(false, |name| name.contains(".mhubtmp."))
}

fn is_stale(entry: &DirEntry, now: SystemTime, threshold: Duration) -> bool {
    std::fs::metadata(entry.path())
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|modified| now.duration_since(modified).ok())
        .map_or(true, |age| age > threshold)
}
