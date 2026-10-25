//! Discovery, deterministic selection and endpoint-scoped user preference.
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Deserialize)]
pub struct Model {
    pub name: String,
    pub size: Option<u64>,
    pub capabilities: Option<Vec<String>>,
}
impl Model {
    pub fn suitable(&self) -> bool {
        if let Some(caps) = &self.capabilities {
            return caps.iter().any(|c| c == "completion");
        }
        // Older Ollama tags omit capabilities. Exclude obvious non-chat models.
        let name = self.name.to_lowercase();
        !["embed", "rerank", "bert"].iter().any(|s| name.contains(s))
    }
    pub fn label(&self) -> String {
        let size = self
            .size
            .map(|n| format!("{:.1} GiB", n as f64 / 1_073_741_824.0))
            .unwrap_or_else(|| "size unknown".into());
        format!(
            "{}  ({size}){}",
            self.name,
            if self.suitable() { "" } else { " · not chat" }
        )
    }
}
pub fn parse(bytes: &[u8]) -> Result<Vec<Model>, String> {
    #[derive(Deserialize)]
    struct Tags {
        models: Vec<Model>,
    }
    let mut models = serde_json::from_slice::<Tags>(bytes)
        .map_err(|e| format!("Invalid model list: {e}"))?
        .models;
    if models
        .iter()
        .any(|m| m.name.is_empty() || m.name.len() > 200 || m.name.chars().any(char::is_control))
    {
        return Err("Invalid model name in discovery response".into());
    }
    models.sort_by(|a, b| a.name.cmp(&b.name));
    models.dedup_by(|a, b| a.name == b.name);
    Ok(models)
}
pub fn configured<'a>(models: &'a [Model], name: &str) -> Option<&'a Model> {
    models
        .iter()
        .find(|m| m.name == name || (!name.contains(':') && m.name == format!("{name}:latest")))
}
pub fn select<'a>(
    models: &'a [Model],
    explicit: &str,
    previous: Option<&str>,
) -> Option<&'a Model> {
    if explicit != "auto"
        && let Some(m) = configured(models, explicit)
    {
        return Some(m);
    }
    if let Some(m) = previous.and_then(|n| models.iter().find(|m| m.name == n)) {
        return Some(m);
    }
    models.iter().filter(|m| m.suitable()).min_by_key(|m| {
        let name = m.name.to_lowercase();
        let specialized = ["coder", "code", "agent"].iter().any(|s| name.contains(s));
        (specialized, m.name.as_str())
    })
}
#[derive(Default, Serialize, Deserialize)]
struct Preferences {
    models: BTreeMap<String, String>,
}
pub fn path() -> Option<PathBuf> {
    crate::persistence::default_path().map(|p| p.with_file_name("models.json"))
}
fn read(path: &Path) -> Result<Preferences, String> {
    let file = match fs::File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Preferences::default()),
        Err(e) => return Err(e.to_string()),
    };
    let mut bytes = Vec::new();
    file.take(65537)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 65536 {
        return Err("Model preferences exceed 64 KiB".into());
    }
    serde_json::from_slice(&bytes).map_err(|e| e.to_string())
}
pub fn load(path: &Path, endpoint: &str) -> Result<Option<String>, String> {
    Ok(read(path)?.models.remove(endpoint.trim_end_matches('/')))
}
pub fn save(path: &Path, endpoint: &str, name: &str) -> Result<(), String> {
    let mut prefs = read(path)?;
    prefs
        .models
        .insert(endpoint.trim_end_matches('/').into(), name.into());
    let bytes = serde_json::to_vec_pretty(&prefs).map_err(|e| e.to_string())?;
    if bytes.len() > 65536 {
        return Err("Model preferences exceed 64 KiB".into());
    }
    fs::create_dir_all(path.parent().ok_or("No preferences directory")?)
        .map_err(|e| e.to_string())?;
    let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let result = (|| -> std::io::Result<()> {
        let mut file = options.open(&temporary)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        fs::rename(&temporary, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(temporary);
    }
    result.map_err(|e| e.to_string())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn discovery_and_selection() {
        let m = parse(br#"{"models":[{"name":"z-chat:2","size":123,"capabilities":["completion"]},{"name":"a-coder:1"},{"name":"embed:1","capabilities":["embedding"]}]}"#).unwrap();
        assert_eq!(m[2].size, Some(123));
        assert_eq!(select(&m, "auto", None).unwrap().name, "z-chat:2");
        assert_eq!(
            select(&m, "missing", Some("a-coder:1")).unwrap().name,
            "a-coder:1"
        );
        assert!(configured(&m, "missing").is_none());
        assert_eq!(
            select(&m, "z-chat:2", Some("a-coder:1")).unwrap().name,
            "z-chat:2"
        );
        assert!(select(&parse(br#"{"models":[]}"#).unwrap(), "auto", None).is_none());
        assert!(select(&m[1..2], "auto", None).is_none());
        assert!(parse(br#"{}"#).is_err());
    }
    #[test]
    fn preferences_are_endpoint_scoped() {
        let dir = std::env::temp_dir().join(format!("terrabot-models-{}", std::process::id()));
        let path = dir.join("models.json");
        save(&path, "http://one/", "chosen:exact").unwrap();
        assert_eq!(
            load(&path, "http://one").unwrap().as_deref(),
            Some("chosen:exact")
        );
        assert_eq!(load(&path, "http://two").unwrap(), None);
        fs::remove_dir_all(dir).unwrap();
    }
}
