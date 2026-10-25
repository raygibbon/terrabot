use serde::Deserialize;
use std::{fs, path::PathBuf};

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub agent: Agent,
    pub ollama: Ollama,
    pub interface: Interface,
    pub security: Security,
    pub pet: PetConfig,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Agent {
    pub name: String,
}
impl Default for Agent {
    fn default() -> Self {
        Self {
            name: "Trixie".into(),
        }
    }
}
#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Ollama {
    pub endpoint: String,
    pub model: String,
    pub num_ctx: u32,
    pub num_predict: u32,
    pub timeout_seconds: u64,
    pub emotion_cues: bool,
}
impl Default for Ollama {
    fn default() -> Self {
        Self {
            endpoint: "http://127.0.0.1:11434".into(),
            model: "auto".into(),
            num_ctx: 8192,
            num_predict: 2048,
            timeout_seconds: 300,
            emotion_cues: false,
        }
    }
}
#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Interface {
    pub theme: String,
    #[serde(alias = "show_terry", alias = "show_terri")]
    pub show_trixie: bool,
    pub animation_intensity: f32,
    pub palette: String,
    pub animation: bool,
    pub animation_fps: u32,
}
impl Default for Interface {
    fn default() -> Self {
        Self {
            theme: "dark".into(),
            show_trixie: true,
            animation_intensity: 1.0,
            palette: "auto".into(),
            animation: true,
            animation_fps: 20,
        }
    }
}
#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PetConfig {
    pub persist: bool,
    pub autonomous: bool,
    pub daily_routine: bool,
    pub develop_preferences: bool,
    pub reset_personality: bool,
    pub diagnostics: bool,
    pub seed: u64,
    /// Wander, sit, play, investigate, sleep, wake, dance, stumble, watch, celebrate.
    pub behaviour_weights: [u16; 10],
}
impl Default for PetConfig {
    fn default() -> Self {
        Self {
            persist: true,
            autonomous: true,
            daily_routine: true,
            develop_preferences: true,
            reset_personality: false,
            diagnostics: false,
            seed: 0x5445525249,
            behaviour_weights: [24, 14, 22, 16, 3, 0, 8, 1, 12, 0],
        }
    }
}
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Security {
    pub tools_enabled: bool,
    pub mcp_enabled: bool,
    pub shell_enabled: bool,
}
impl Config {
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut c: Self = toml::from_str(text).map_err(|e| e.to_string())?;
        if matches!(c.agent.name.as_str(), "Terri" | "Terry") {
            c.agent.name = "Trixie".into();
        }
        if c.ollama.endpoint.len() > 2048 {
            return Err("endpoint exceeds 2048 bytes".into());
        }
        let url = reqwest::Url::parse(&c.ollama.endpoint)
            .map_err(|e| format!("Invalid endpoint: {e}"))?;
        if !matches!(url.scheme(), "http" | "https")
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
            || url.path() != "/"
        {
            return Err(
                "endpoint must be an HTTP(S) origin without credentials, path or query".into(),
            );
        }
        for (label, value, max) in [
            ("agent.name", &c.agent.name, 40),
            ("ollama.model", &c.ollama.model, 200),
        ] {
            if value.trim().is_empty() || value.len() > max || value.chars().any(char::is_control) {
                return Err(format!("Invalid {label}"));
            }
        }
        if !matches!(c.interface.theme.as_str(), "dark" | "light") {
            return Err("theme must be dark or light".into());
        }
        if !(1..=60).contains(&c.interface.animation_fps)
            || !(512..=131072).contains(&c.ollama.num_ctx)
            || !(1..=32768).contains(&c.ollama.num_predict)
            || !(5..=3600).contains(&c.ollama.timeout_seconds)
        {
            return Err(
                "Limits: fps 1–60, num_ctx 512–131072, num_predict 1–32768, timeout_seconds 5–3600"
                    .into(),
            );
        }
        if !c.interface.animation_intensity.is_finite()
            || !(0.0..=1.0).contains(&c.interface.animation_intensity)
            || !matches!(
                c.interface.palette.as_str(),
                "auto" | "truecolor" | "indexed" | "ascii"
            )
        {
            return Err(
                "animation_intensity must be 0–1; palette auto, truecolor, indexed or ascii".into(),
            );
        }
        if c.pet.behaviour_weights.iter().any(|w| *w > 1000)
            || c.pet.behaviour_weights.iter().all(|w| *w == 0)
        {
            return Err(
                "behaviour_weights require ten weights in 0–1000, at least one nonzero".into(),
            );
        }
        if c.security.tools_enabled || c.security.mcp_enabled || c.security.shell_enabled {
            return Err(
                "v0.4.4 supports chat only; all security enable flags must remain false".into(),
            );
        }
        Ok(c)
    }
    pub fn load(path: Option<PathBuf>) -> Result<Self, String> {
        let explicit = path.is_some();
        let path = match path {
            Some(p) => p,
            None => {
                let home = std::env::var_os("HOME").ok_or("HOME is unset; use --config PATH")?;
                PathBuf::from(home).join(".config/terrabot/config.toml")
            }
        };
        match fs::read_to_string(&path) {
            Ok(s) => Self::parse(&s),
            Err(e) if !explicit && e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(format!("{}: {e}", path.display())),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn defaults_and_partial() {
        for key in ["show_terri", "show_terry", "show_trixie"] {
            let c =
                Config::parse(&format!("[agent]\nname='Terri'\n[interface]\n{key}=false")).unwrap();
            assert_eq!(c.agent.name, "Trixie");
            assert!(!c.interface.show_trixie);
        }
        assert_eq!(
            Config::parse("[agent]\nname='Custom'").unwrap().agent.name,
            "Custom"
        );
        assert_eq!(Config::parse("").unwrap().ollama.model, "auto");
        assert_eq!(
            Config::parse("[interface]\nanimation=false")
                .unwrap()
                .interface
                .animation_fps,
            20
        );
    }
    #[test]
    fn reject_invalid() {
        for s in [
            "[interface]\nanimation_fps=0",
            "[security]\nshell_enabled=true",
            "[ollama]\nendpoint='file:///tmp/x'",
            "[agent]\nname=''",
            "[ollama]\nnum_predict=0",
            "[interface]\ntheme='other'",
            "typo=true",
            "[interface]\nanimation_intensity=2.0",
            "[interface]\npalette='invalid'",
            "[pet]\nbehaviour_weights=[0,0,0,0,0,0,0,0,0,0]",
        ] {
            assert!(Config::parse(s).is_err(), "{s}");
        }
    }
}
