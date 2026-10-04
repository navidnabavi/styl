//! Caches parsed `.stylrc` files.
//!
//! Resolving a config does two things: walk up the directory tree looking for
//! `.stylrc`, then read and parse it. Doing both on every publish and every
//! formatting request means work on every debounce batch.
//!
//! Only the second half is cached. Discovery is a handful of `exists()` calls
//! and has to run every time, because a `.stylrc` created after a document was
//! opened must still be found. Reading and parsing TOML is the part worth
//! keeping off that path, and it is skipped while the file's mtime is unchanged.
//!
//! Caching discovery results instead — the obvious first design — makes
//! correctness depend on the client supporting dynamic file watching, since
//! nothing else would ever invalidate a cached miss. The `.stylrc` watcher is
//! still registered, because [`ConfigCache::clear`] picks up an edit without
//! waiting on mtime granularity; it is an optimization, not a requirement.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::linter::config::{discover_config, load_config, Config};

#[derive(Default)]
pub(super) struct ConfigCache {
    /// Config file path -> (mtime when parsed, the parse result).
    ///
    /// A failed parse is cached too, so a malformed `.stylrc` is not re-read on
    /// every keystroke.
    parsed: HashMap<PathBuf, (Option<SystemTime>, Option<Config>)>,
}

impl ConfigCache {
    /// The config governing `directory`, parsing only when the file is new or
    /// has changed on disk.
    pub(super) fn get(&mut self, directory: &Path) -> Option<&Config> {
        let path = discover_config(directory)?;
        let mtime = std::fs::metadata(&path)
            .ok()
            .and_then(|m| m.modified().ok());

        let stale = match self.parsed.get(&path) {
            Some((parsed_at, _)) => *parsed_at != mtime,
            None => true,
        };
        if stale {
            self.parsed
                .insert(path.clone(), (mtime, load_config(&path).ok()));
        }

        self.parsed
            .get(&path)
            .and_then(|(_, config)| config.as_ref())
    }

    /// Forget everything, so the next lookup re-parses.
    ///
    /// Called when the `.stylrc` watcher reports a change, which catches an edit
    /// that mtime granularity might not distinguish.
    pub(super) fn clear(&mut self) {
        self.parsed.clear();
    }

    #[cfg(test)]
    pub(super) fn len(&self) -> usize {
        self.parsed.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_a_config_created_after_the_first_lookup() {
        // The regression this design exists for: caching a miss made a newly
        // written `.stylrc` invisible for the rest of the session unless the
        // client happened to support file watching.
        let directory = tempfile::tempdir().expect("tempdir");
        let mut cache = ConfigCache::default();

        assert!(cache.get(directory.path()).is_none());

        std::fs::write(directory.path().join(".stylrc"), "spec = \"mapbox\"\n").expect("write");
        let config = cache
            .get(directory.path())
            .expect("a config written after the miss must still be found");
        assert_eq!(config.spec.as_deref(), Some("mapbox"));
    }

    #[test]
    fn reparses_only_when_the_file_changes() {
        let directory = tempfile::tempdir().expect("tempdir");
        let path = directory.path().join(".stylrc");
        std::fs::write(&path, "spec = \"mapbox\"\n").expect("write");

        let mut cache = ConfigCache::default();
        assert_eq!(
            cache.get(directory.path()).unwrap().spec.as_deref(),
            Some("mapbox")
        );
        assert_eq!(cache.len(), 1);

        // Repeated lookups reuse the parse.
        for _ in 0..5 {
            assert_eq!(
                cache.get(directory.path()).unwrap().spec.as_deref(),
                Some("mapbox")
            );
        }
        assert_eq!(cache.len(), 1);

        // An edit is picked up. mtime resolution is fine here because the write
        // below is a separate syscall with distinct content.
        std::thread::sleep(std::time::Duration::from_millis(10));
        std::fs::write(&path, "spec = \"maplibre\"\n").expect("rewrite");
        assert_eq!(
            cache.get(directory.path()).unwrap().spec.as_deref(),
            Some("maplibre")
        );
    }

    #[test]
    fn clear_forces_a_reparse() {
        let directory = tempfile::tempdir().expect("tempdir");
        std::fs::write(directory.path().join(".stylrc"), "spec = \"mapbox\"\n").expect("write");

        let mut cache = ConfigCache::default();
        assert!(cache.get(directory.path()).is_some());
        assert_eq!(cache.len(), 1);

        cache.clear();
        assert_eq!(cache.len(), 0);
        assert!(cache.get(directory.path()).is_some());
    }

    #[test]
    fn a_malformed_config_is_cached_as_a_failure() {
        let directory = tempfile::tempdir().expect("tempdir");
        std::fs::write(directory.path().join(".stylrc"), "this is not = = toml\n").expect("write");

        let mut cache = ConfigCache::default();
        assert!(cache.get(directory.path()).is_none());
        assert_eq!(cache.len(), 1, "the failure should be remembered");
    }
}
