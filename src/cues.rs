use crate::pet::Mood;
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Gesture {
    #[default]
    None,
    HeadTilt,
    Nod,
    Wave,
    Celebrate,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmotionCue {
    pub emotion: Mood,
    pub gesture: Gesture,
    pub intensity: f32,
}
impl EmotionCue {
    pub fn validate(&self) -> Result<(), String> {
        if !self.intensity.is_finite() || !(0.0..=1.0).contains(&self.intensity) {
            Err("cue intensity must be finite and in 0–1".into())
        } else {
            Ok(())
        }
    }
    pub fn parse(text: &str) -> Result<Self, String> {
        if text.len() > 2048 {
            return Err("Cue too large".into());
        }
        let cue: Self = serde_json::from_str(text).map_err(|e| e.to_string())?;
        cue.validate()?;
        Ok(cue)
    }
    pub fn schema() -> serde_json::Value {
        serde_json::json!({"type":"object","additionalProperties":false,"required":["emotion","gesture","intensity"],"properties":{"emotion":{"type":"string","enum":["happy","curious","bored","sleepy","excited","focused","confused","frustrated","surprised","shy","content"]},"gesture":{"type":"string","enum":["none","head_tilt","nod","wave","celebrate"]},"intensity":{"type":"number","minimum":0,"maximum":1}}})
    }
}
pub fn profile(name: &str) -> String {
    format!(
        "You are {name}, an AI assistant embodied as an original small terminal creature. You are friendly, curious, clever and technically capable. Be playful only when appropriate, and gentle and thoughtful in serious conversations. Be accurate and acknowledge uncertainty. Do not narrate animations or mention virtual-pet needs unless asked. Never claim human consciousness, physical sensations or personal real-world experiences. Your cosmetic mood never affects answer accuracy. You cannot execute tools or commands; describe proposed steps as proposals, not actions you have performed."
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn strict_validation() {
        assert!(
            EmotionCue::parse(r#"{"emotion":"curious","gesture":"head_tilt","intensity":0.6}"#)
                .is_ok()
        );
        for s in [
            r#"{"emotion":"evil","gesture":"none","intensity":0}"#,
            r#"{"emotion":"happy","gesture":"exec","intensity":0}"#,
            r#"{"emotion":"happy","gesture":"none","intensity":2}"#,
            r#"{"emotion":"happy","gesture":"none","intensity":0,"command":"sh"}"#,
        ] {
            assert!(EmotionCue::parse(s).is_err());
        }
        assert!(
            EmotionCue {
                emotion: Mood::Happy,
                gesture: Gesture::None,
                intensity: f32::NAN
            }
            .validate()
            .is_err()
        );
    }
}
