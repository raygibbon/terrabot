//! Versioned metadata only. No chat content; no offline simulation.
use crate::pet::Snapshot;
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};
const LIMIT: u64 = 16 * 1024;
pub fn default_path() -> Option<PathBuf> {
    std::env::var_os("XDG_STATE_HOME")
        .filter(|p| Path::new(p).is_absolute())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/state")))
        .map(|p| p.join("terrabot/terri.json"))
}
pub fn load(path: &Path, seed: u64) -> Result<Snapshot, String> {
    let file = match fs::File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Snapshot::new(seed)),
        Err(e) => return Err(e.to_string()),
    };
    let mut bytes = Vec::new();
    file.take(LIMIT + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > LIMIT {
        return Err("Terri state exceeds 16 KiB".into());
    }
    let state: Snapshot = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    state.validate()?;
    Ok(state)
}
pub fn save(path: &Path, state: &Snapshot) -> Result<(), String> {
    state.validate()?;
    let parent = path.parent().ok_or("State path has no parent")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let temporary = parent.join(format!(".terri-{}.tmp", std::process::id()));
    let bytes = serde_json::to_vec_pretty(state).map_err(|e| e.to_string())?;
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(&temporary)
        .map_err(|e| format!("Cannot create state temporary file: {e}"))?;
    let result = (|| {
        file.write_all(&bytes)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temporary, path)?;
        #[cfg(unix)]
        fs::File::open(parent)?.sync_all()?;
        Ok::<_, std::io::Error>(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result.map_err(|e| e.to_string())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn roundtrip_atomic_and_invalid_preserved() {
        let path = std::env::temp_dir()
            .join(format!("terrabot-state-{}", std::process::id()))
            .join("terri.json");
        let _ = fs::remove_dir_all(path.parent().unwrap());
        let mut s = load(&path, 42).unwrap();
        s.conversations = 7;
        save(&path, &s).unwrap();
        assert_eq!(load(&path, 99).unwrap(), s);
        let original = fs::read(&path).unwrap();
        let mut invalid = s;
        invalid.energy = f32::NAN;
        assert!(save(&path, &invalid).is_err());
        assert_eq!(fs::read(&path).unwrap(), original);
        fs::write(&path, b"{\"version\":99}").unwrap();
        assert!(load(&path, 0).is_err());
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
    #[test]
    fn oversized_rejected() {
        let path = std::env::temp_dir().join(format!("terrabot-oversize-{}", std::process::id()));
        fs::write(&path, vec![b' '; LIMIT as usize + 1]).unwrap();
        assert!(load(&path, 0).is_err());
        fs::remove_file(path).unwrap();
    }
}
