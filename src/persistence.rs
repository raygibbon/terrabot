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
        .map(|p| p.join("terrabot/trixie.json"))
}
pub fn load(path: &Path, seed: u64) -> Result<Snapshot, String> {
    let file = match fs::File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // Read the old name only when the new file is absent. Clean shutdown
            // writes trixie.json atomically; keep the legacy file as a backup.
            if path.file_name().is_some_and(|n| n == "trixie.json") {
                return load(&path.with_file_name("terri.json"), seed);
            }
            return Ok(Snapshot::new(seed));
        }
        Err(e) => return Err(e.to_string()),
    };
    let mut bytes = Vec::new();
    file.take(LIMIT + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > LIMIT {
        return Err("Trixie state exceeds 16 KiB".into());
    }
    let mut value: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    match value["version"].as_u64() {
        Some(1) => {
            let object = value.as_object_mut().ok_or("Invalid state object")?;
            if object.contains_key("life") {
                return Err("Unexpected life field in legacy state".into());
            }
            object.insert("version".into(), 2.into());
            object.insert(
                "life".into(),
                serde_json::to_value(crate::life::Life::new(seed)).map_err(|e| e.to_string())?,
            );
        }
        Some(2) => {}
        _ => return Err("Unsupported Trixie state version".into()),
    }
    let state: Snapshot = serde_json::from_value(value).map_err(|e| e.to_string())?;
    state.validate()?;
    Ok(state)
}
pub fn save(path: &Path, state: &Snapshot) -> Result<(), String> {
    state.validate()?;
    let parent = path.parent().ok_or("State path has no parent")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let temporary = parent.join(format!(".trixie-{}.tmp", std::process::id()));
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
    fn character_rename_preserves_legacy_and_prefers_new_state() {
        let dir = std::env::temp_dir().join(format!("trixie-rename-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let old = dir.join("terri.json");
        let new = dir.join("trixie.json");
        let mut state = Snapshot::new(42);
        state.conversations = 73;
        state.life.preferences[0] = 0.77;
        save(&old, &state).unwrap();
        let original = fs::read(&old).unwrap();
        assert_eq!(load(&new, 99).unwrap(), state);
        save(&new, &state).unwrap();
        assert_eq!(fs::read(&old).unwrap(), original);
        fs::write(&old, b"invalid legacy data").unwrap();
        assert_eq!(load(&new, 0).unwrap(), state);
        fs::remove_file(&new).unwrap();
        assert!(load(&new, 0).is_err());
        assert!(!new.exists());
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn migrates_legacy_without_losing_metadata() {
        let path = std::env::temp_dir().join(format!("terrabot-migration-{}", std::process::id()));
        let mut legacy = serde_json::to_value(Snapshot::new(19)).unwrap();
        legacy.as_object_mut().unwrap().remove("life");
        legacy["version"] = 1.into();
        legacy["conversations"] = 42.into();
        let original = serde_json::to_vec(&legacy).unwrap();
        fs::write(&path, &original).unwrap();
        let migrated = load(&path, 19).unwrap();
        assert_eq!(migrated.version, 2);
        assert_eq!(migrated.conversations, 42);
        assert_eq!(migrated.life, crate::life::Life::new(19));
        assert_eq!(fs::read(&path).unwrap(), original);
        save(&path, &migrated).unwrap();
        assert_eq!(load(&path, 0).unwrap(), migrated);
        fs::remove_file(path).unwrap();
    }
    #[test]
    fn roundtrip_atomic_and_invalid_preserved() {
        let path = std::env::temp_dir()
            .join(format!("terrabot-state-{}", std::process::id()))
            .join("trixie.json");
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
