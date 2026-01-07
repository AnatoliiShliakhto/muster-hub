use crate::error::StorageError;
use std::path::{Component, Path, PathBuf};

/// Collapse `.` / `..` lexically while ensuring the path never escapes the sandbox root.
///
/// Allows `..` as long as it doesn't go "above" the provided root (i.e. above the
/// empty relative base).
fn normalize_relative(path: &Path) -> Result<PathBuf, StorageError> {
    let mut out = PathBuf::new();

    for c in path.components() {
        match c {
            Component::CurDir => {},
            Component::Normal(seg) => out.push(seg),
            Component::ParentDir => {
                if !out.pop() {
                    return Err(StorageError::PathTraversalAttempt {
                        message: path.display().to_string().into(),
                        context: Some("Path attempted to escape sandbox via '..'".into()),
                    });
                }
            },
            Component::RootDir | Component::Prefix(_) => {
                return Err(StorageError::PathTraversalAttempt {
                    message: path.display().to_string().into(),
                    context: Some("Absolute paths are not allowed in sandbox".into()),
                });
            },
        }
    }

    Ok(out)
}

/// Safely joins a path to the root and ensures it doesn't escape the sandbox.
pub(crate) fn resolve_path(root: &Path, path: impl AsRef<Path>) -> Result<PathBuf, StorageError> {
    let path = path.as_ref();

    if path.is_absolute() {
        return Err(StorageError::PathTraversalAttempt {
            message: format!("Absolute paths are not allowed in sandbox {}", path.display()).into(),
            context: None,
        });
    }

    let safe_rel = normalize_relative(path)?;
    let joined = root.join(safe_rel);

    match joined.canonicalize() {
        Ok(canonical) => validate_canonical(root, canonical),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => validate_path(root, &joined),
        Err(e) => Err(StorageError::Io { source: e, context: None }),
    }
}

/// Resolves a path with namespace and sharding applied.
///
/// Subdirectories are preserved, and sharding is applied to the final filename.
pub(crate) fn resolve_sharding(
    root: &Path,
    ns: Option<&str>,
    path: impl AsRef<Path>,
) -> Result<PathBuf, StorageError> {
    let path = path.as_ref();
    let parent = path.parent().filter(|p| !p.as_os_str().is_empty());
    let filename =
        path.file_name().and_then(|s| s.to_str()).ok_or_else(|| StorageError::FileNotFound {
            message: path.display().to_string().into(),
            context: Some("Target must be a file".into()),
        })?;

    let mut shard = PathBuf::new();
    if let Some(n) = ns {
        shard.push(n);
    }
    if let Some(p) = parent {
        shard.push(p);
    }

    let chars: Vec<char> = filename.chars().collect();
    if chars.len() >= 4 {
        let shard1: String = chars[0..2].iter().collect();
        let shard2: String = chars[2..4].iter().collect();
        shard.push(shard1);
        shard.push(shard2);
    }
    shard.push(filename);

    resolve_path(root, shard)
}

fn validate_canonical(root: &Path, canonical: PathBuf) -> Result<PathBuf, StorageError> {
    if canonical.starts_with(root) {
        Ok(canonical)
    } else {
        Err(StorageError::PathTraversalAttempt {
            message: canonical.display().to_string().into(),
            context: Some("Path attempted to escape sandbox via .. sequences".into()),
        })
    }
}

/// Validates a path that doesn't exist yet by finding and verifying its first existing ancestor.
///
/// This function walks up the directory tree from the target path until it finds a parent
/// that exists on disk, then verifies that parent is within the sandbox. This allows safe
/// validation of deeply nested paths without requiring all intermediate directories to exist.
///
/// # Security
/// - Prevents symlink attacks by canonicalizing the first existing ancestor
/// - Ensures the entire path chain originates from within the sandbox
/// - Detects attempts to escape via relative path segments (e.g., `../../`)
fn validate_path(root: &Path, joined: &Path) -> Result<PathBuf, StorageError> {
    if !joined.starts_with(root) {
        return Err(StorageError::PathTraversalAttempt {
            message: joined.display().to_string().into(),
            context: Some("Path is outside sandbox boundaries".into()),
        });
    }

    let mut current = Some(joined);

    while let Some(path) = current {
        if path == root {
            return Ok(joined.to_path_buf());
        }

        if path.exists() {
            return match path.canonicalize() {
                Ok(canonical) if canonical.starts_with(root) => Ok(joined.to_path_buf()),
                Ok(canonical) => Err(StorageError::PathTraversalAttempt {
                    message: canonical.display().to_string().into(),
                    context: Some("Existing parent directory is a symlink outside sandbox".into()),
                }),
                Err(e) => Err(StorageError::Io {
                    source: e,
                    context: Some("Failed to verify parent directory".into()),
                }),
            };
        }

        current = path.parent();
    }

    Err(StorageError::PathTraversalAttempt {
        message: joined.display().to_string().into(),
        context: Some("No valid parent directory found within sandbox".into()),
    })
}
