//! Pure cosmetic simulation. No I/O, model requests or execution capabilities.
use crate::{
    config::PetConfig,
    cues::{EmotionCue, Gesture},
};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use terragame_core::{FixedStep, random::Random};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mood {
    Happy,
    Curious,
    Bored,
    Sleepy,
    Excited,
    Focused,
    Confused,
    Frustrated,
    Surprised,
    Shy,
    #[default]
    Content,
}
impl Mood {
    pub const ALL: [Self; 11] = [
        Self::Happy,
        Self::Curious,
        Self::Bored,
        Self::Sleepy,
        Self::Excited,
        Self::Focused,
        Self::Confused,
        Self::Frustrated,
        Self::Surprised,
        Self::Shy,
        Self::Content,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Happy => "Happy",
            Self::Curious => "Curious",
            Self::Bored => "Bored",
            Self::Sleepy => "Sleepy",
            Self::Excited => "Excited",
            Self::Focused => "Focused",
            Self::Confused => "Confused",
            Self::Frustrated => "Frustrated",
            Self::Surprised => "Surprised",
            Self::Shy => "Shy",
            Self::Content => "Content",
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Activity {
    Wander,
    #[default]
    Sit,
    Play,
    Investigate,
    Sleep,
    Wake,
    Dance,
    Stumble,
    Watch,
    Celebrate,
}
impl Activity {
    pub const ALL: [Self; 10] = [
        Self::Wander,
        Self::Sit,
        Self::Play,
        Self::Investigate,
        Self::Sleep,
        Self::Wake,
        Self::Dance,
        Self::Stumble,
        Self::Watch,
        Self::Celebrate,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Wander => "Exploring",
            Self::Sit => "Taking a moment",
            Self::Play => "Playing with light",
            Self::Investigate => "What's over there?",
            Self::Sleep => "Resting",
            Self::Wake => "A little stretch",
            Self::Dance => "A little dance",
            Self::Stumble => "Found my feet",
            Self::Watch => "Here with you",
            Self::Celebrate => "Nicely done!",
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Interaction {
    #[default]
    None,
    Typing,
    Question,
    Answer,
    Error,
    Denied,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub version: u32,
    pub energy: f32,
    pub curiosity: f32,
    pub boredom: f32,
    pub playfulness: f32,
    pub sociability: f32,
    pub mood: Mood,
    pub preferred: [Activity; 3],
    pub interactions: u64,
    pub conversations: u64,
    pub sessions: u64,
    pub active_ticks: u64,
    pub rng_state: u64,
    pub last_interaction: Interaction,
}
impl Snapshot {
    pub fn new(seed: u64) -> Self {
        Self {
            version: 1,
            energy: 0.85,
            curiosity: 0.75,
            boredom: 0.1,
            playfulness: 0.65,
            sociability: 0.8,
            mood: Mood::Content,
            preferred: [Activity::Play, Activity::Investigate, Activity::Wander],
            interactions: 0,
            conversations: 0,
            sessions: 0,
            active_ticks: 0,
            rng_state: seed,
            last_interaction: Interaction::None,
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 {
            return Err(format!("Unsupported Terri state version {}", self.version));
        }
        if [
            self.energy,
            self.curiosity,
            self.boredom,
            self.playfulness,
            self.sociability,
        ]
        .iter()
        .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
        {
            return Err("Terri traits must be finite numbers in 0–1".into());
        }
        Ok(())
    }
}
pub enum PetEvent<'a> {
    Idle,
    Typing,
    User(&'a str),
    Thinking,
    Speaking,
    Done(&'a str),
    Error,
    Denied,
    Cue(EmotionCue),
}
/// One authoritative fixed-step timeline; drawing never advances RNG or simulation.
pub struct Pet {
    pub traits: Snapshot,
    pub activity: Activity,
    pub x: f32,
    pub facing: f32,
    pub speaking: bool,
    pub thinking: bool,
    pub gentle: bool,
    pub denied: bool,
    pub gesture: Gesture,
    pub cue_intensity: f32,
    pub age: f32,
    pub phase: f32,
    pub blink: bool,
    fixed: FixedStep,
    rng: Random,
    config: PetConfig,
    next_choice: u64,
    cooldowns: [u64; 10],
    priority: u8,
    hold_until: u64,
    next_blink: u64,
    blink_until: u64,
    target: f32,
    success_context: bool,
}
impl Pet {
    pub fn new(config: PetConfig, mut traits: Snapshot) -> Self {
        if traits.validate().is_err() {
            traits = Snapshot::new(config.seed);
        }
        let rng = Random::new(traits.rng_state);
        Self {
            traits,
            activity: Activity::Sit,
            x: 0.5,
            facing: 1.0,
            speaking: false,
            thinking: false,
            gentle: false,
            denied: false,
            gesture: Gesture::None,
            cue_intensity: 0.5,
            age: 0.0,
            phase: 0.0,
            blink: false,
            fixed: FixedStep::new(Duration::from_millis(50), 20)
                .expect("constant valid simulation step"),
            rng,
            config,
            next_choice: 80,
            cooldowns: [0; 10],
            priority: 0,
            hold_until: 0,
            next_blink: 60,
            blink_until: 0,
            target: 0.5,
            success_context: false,
        }
    }
    pub fn snapshot(&self) -> Snapshot {
        let mut state = self.traits.clone();
        state.rng_state = self.rng.state();
        state
    }
    pub fn advance(&mut self, elapsed: Duration) {
        let steps = self.fixed.advance(elapsed);
        for _ in 0..steps {
            self.step();
        }
    }
    pub fn remaining(&self) -> Duration {
        self.fixed.remaining()
    }
    fn step(&mut self) {
        self.traits.active_ticks = self.traits.active_ticks.saturating_add(1);
        let tick = self.traits.active_ticks;
        self.phase = (self.phase + 0.05) % 3600.0;
        self.age = (self.age + 0.05).min(3600.0);
        if tick >= self.next_blink {
            self.blink_until = tick.saturating_add(3);
            self.next_blink = tick.saturating_add(60 + self.rng.below(100).unwrap_or(0));
        }
        self.blink = tick < self.blink_until;
        if self.activity == Activity::Wander {
            let distance = self.target - self.x;
            self.facing = if distance < 0.0 { -1.0 } else { 1.0 };
            self.x = (self.x + distance.clamp(-0.003, 0.003)).clamp(0.0, 1.0);
        }
        if tick.is_multiple_of(20) {
            let rest = self.activity == Activity::Sleep;
            self.traits.energy =
                (self.traits.energy + if rest { 0.008 } else { -0.0006 }).clamp(0.0, 1.0);
            self.traits.boredom = (self.traits.boredom
                + if matches!(
                    self.activity,
                    Activity::Play | Activity::Dance | Activity::Investigate
                ) {
                    -0.012
                } else if self.thinking || self.speaking {
                    -0.015
                } else {
                    0.003
                })
            .clamp(0.0, 1.0);
        }
        if self.thinking || self.speaking {
            return;
        }
        if tick >= self.hold_until {
            self.priority = 0;
            self.denied = false;
            self.gesture = Gesture::None;
        }
        if self.priority == 0 && tick >= self.next_choice {
            if self.activity == Activity::Sleep && self.traits.energy < 0.85 {
                self.next_choice = tick.saturating_add(100);
                return;
            }
            if self.activity == Activity::Sleep {
                self.choose(Activity::Wake, 0, 60);
                return;
            }
            let activity = self.select();
            let duration = 120 + self.rng.below(160).unwrap_or(0);
            self.choose(activity, 0, duration);
        }
    }
    fn select(&mut self) -> Activity {
        if self.traits.energy < 0.18 {
            self.traits.mood = Mood::Sleepy;
            return Activity::Sleep;
        }
        let tick = self.traits.active_ticks;
        let mut weights = self.config.behaviour_weights.map(u64::from);
        for (i, a) in Activity::ALL.iter().enumerate() {
            if self.cooldowns[i] > tick || *a == self.activity {
                weights[i] = 0;
                continue;
            }
            if self.traits.preferred.contains(a) {
                weights[i] *= 2;
            }
            let trait_weight = match a {
                Activity::Play | Activity::Dance => self.traits.playfulness,
                Activity::Wander | Activity::Investigate => self.traits.curiosity,
                Activity::Watch => self.traits.sociability,
                _ => 0.5,
            };
            weights[i] *= 1 + (trait_weight * 4.0) as u64;
            if self.gentle
                && matches!(
                    a,
                    Activity::Dance | Activity::Stumble | Activity::Celebrate | Activity::Play
                )
            {
                weights[i] = 0;
            }
            if self.traits.boredom > 0.5
                && matches!(a, Activity::Play | Activity::Wander | Activity::Investigate)
            {
                weights[i] *= 3;
            }
        }
        let mut value = self.rng.below(weights.iter().sum()).unwrap_or(0);
        for (i, weight) in weights.iter().enumerate() {
            if value < *weight {
                return Activity::ALL[i];
            }
            value = value.saturating_sub(*weight);
        }
        Activity::Sit
    }
    pub fn choose(&mut self, activity: Activity, priority: u8, duration: u64) {
        if priority < self.priority {
            return;
        }
        if self.activity != activity {
            self.cooldowns[self.activity as usize] = self.traits.active_ticks.saturating_add(180);
            self.age = 0.0;
        }
        self.activity = activity;
        self.priority = priority;
        self.hold_until = self.traits.active_ticks.saturating_add(duration);
        self.next_choice = self.hold_until;
        if activity == Activity::Wander {
            self.target = self.rng.below(1000).unwrap_or(500) as f32 / 1000.0;
        }
        if priority == 0 {
            self.traits.mood = match activity {
                Activity::Sleep => Mood::Sleepy,
                Activity::Investigate | Activity::Wander => Mood::Curious,
                Activity::Play | Activity::Dance => Mood::Happy,
                Activity::Stumble => Mood::Surprised,
                _ => {
                    if self.traits.boredom > 0.7 {
                        Mood::Bored
                    } else {
                        Mood::Content
                    }
                }
            };
        }
    }
    pub fn demo_activity(&mut self, a: Activity) {
        self.priority = 0;
        self.thinking = false;
        self.speaking = false;
        self.denied = false;
        self.gentle = false;
        self.choose(a, 20, 160);
    }
    pub fn event(&mut self, event: PetEvent<'_>) {
        match event {
            PetEvent::Idle => {
                self.thinking = false;
                self.speaking = false;
                self.priority = 0;
                self.choose(Activity::Sit, 0, 80);
            }
            PetEvent::Typing => {
                if self.thinking || self.speaking {
                    return;
                }
                let was_sleep = self.activity == Activity::Sleep;
                self.traits.last_interaction = Interaction::Typing;
                self.traits.mood = Mood::Focused;
                self.choose(
                    if was_sleep {
                        Activity::Wake
                    } else {
                        Activity::Watch
                    },
                    60,
                    60,
                );
            }
            PetEvent::User(text) => {
                self.priority = 0;
                self.gentle = serious(text);
                self.gesture = Gesture::None;
                self.success_context = success(text);
                if !self.gentle
                    && ["hello", "hi terri"]
                        .iter()
                        .any(|s| text.to_lowercase().contains(s))
                {
                    self.gesture = Gesture::Wave;
                }
                self.traits.interactions = self.traits.interactions.saturating_add(1);
                self.traits.last_interaction = Interaction::Question;
                self.traits.boredom = (self.traits.boredom - 0.2).max(0.0);
                self.traits.mood = if self.gentle {
                    Mood::Focused
                } else if self.success_context || success(text) {
                    Mood::Happy
                } else {
                    Mood::Curious
                };
                self.choose(
                    if !self.gentle && (self.success_context || success(text)) {
                        Activity::Celebrate
                    } else {
                        Activity::Watch
                    },
                    60,
                    100,
                );
            }
            PetEvent::Thinking => {
                self.thinking = true;
                self.speaking = false;
                self.traits.mood = Mood::Focused;
                self.choose(Activity::Watch, 80, 100);
            }
            PetEvent::Speaking => {
                self.thinking = false;
                self.speaking = true;
                self.choose(Activity::Watch, 90, 80);
            }
            PetEvent::Done(text) => {
                self.thinking = false;
                self.speaking = false;
                self.priority = 0;
                self.gentle |= serious(text);
                self.traits.conversations = self.traits.conversations.saturating_add(1);
                self.traits.last_interaction = Interaction::Answer;
                self.traits.mood = if self.gentle {
                    Mood::Content
                } else if self.success_context || success(text) {
                    Mood::Happy
                } else {
                    Mood::Content
                };
                self.choose(
                    if !self.gentle && (self.success_context || success(text)) {
                        Activity::Celebrate
                    } else {
                        Activity::Watch
                    },
                    40,
                    160,
                );
            }
            PetEvent::Error => {
                self.thinking = false;
                self.speaking = false;
                self.priority = 0;
                self.traits.mood = Mood::Confused;
                self.traits.last_interaction = Interaction::Error;
                self.choose(Activity::Watch, 70, 100);
            }
            PetEvent::Denied => {
                self.thinking = false;
                self.speaking = false;
                self.denied = true;
                self.traits.last_interaction = Interaction::Denied;
                self.traits.mood = Mood::Focused;
                self.choose(Activity::Watch, 100, 80);
            }
            PetEvent::Cue(cue) => {
                if cue.validate().is_err()
                    || self.priority > 40
                    || self.thinking
                    || self.speaking
                    || self.gentle
                {
                    return;
                }
                self.traits.mood = cue.emotion;
                self.gesture = cue.gesture;
                self.cue_intensity = cue.intensity;
                self.choose(
                    match cue.gesture {
                        Gesture::Celebrate => Activity::Celebrate,
                        Gesture::Wave => Activity::Watch,
                        _ => Activity::Sit,
                    },
                    40,
                    80,
                );
            }
        }
    }
}
fn serious(text: &str) -> bool {
    let text = text.to_lowercase();
    [
        "terrible day",
        "sad",
        "grief",
        "died",
        "depress",
        "anxious",
        "sorry",
        "struggling",
        "hurt",
        "lost my",
        "afraid",
        "lonely",
        "suicid",
        "illness",
        "crisis",
    ]
    .iter()
    .any(|s| text.contains(s))
}
fn success(text: &str) -> bool {
    let text = text.to_lowercase();
    [
        "that worked",
        "it worked",
        "well done",
        "successfully",
        "great job",
    ]
    .iter()
    .any(|s| text.contains(s))
}
#[cfg(test)]
mod tests {
    use super::*;
    fn pet() -> Pet {
        Pet::new(PetConfig::default(), Snapshot::new(7))
    }
    #[test]
    fn deterministic_bounded_and_cooldowns() {
        let mut a = pet();
        let mut b = pet();
        for _ in 0..10000 {
            a.advance(Duration::from_millis(50));
            b.advance(Duration::from_millis(50));
        }
        assert_eq!(a.snapshot(), b.snapshot());
        assert_eq!(a.activity, b.activity);
        a.snapshot().validate().unwrap();
        assert!((0.0..=1.0).contains(&a.x));
    }
    #[test]
    fn priorities_and_sensitive_conversation() {
        let mut p = pet();
        p.demo_activity(Activity::Sleep);
        p.event(PetEvent::Typing);
        assert_eq!(p.activity, Activity::Wake);
        p.event(PetEvent::User("I'm having a terrible day"));
        p.event(PetEvent::Thinking);
        p.demo_activity(Activity::Dance);
        p.event(PetEvent::Thinking);
        p.choose(Activity::Play, 0, 1);
        assert_eq!(p.activity, Activity::Watch);
        p.event(PetEvent::User("I'm sad"));
        p.event(PetEvent::Done("That worked!"));
        assert_ne!(p.activity, Activity::Celebrate);
        p.event(PetEvent::Cue(EmotionCue {
            emotion: Mood::Excited,
            gesture: Gesture::Celebrate,
            intensity: 1.0,
        }));
        assert_ne!(p.activity, Activity::Celebrate);
    }
    #[test]
    fn rest_recovers_without_penalty() {
        let mut p = pet();
        p.traits.energy = 0.05;
        p.next_choice = 0;
        p.advance(Duration::from_secs(1));
        assert_eq!(p.activity, Activity::Sleep);
        let e = p.traits.energy;
        for _ in 0..20 {
            p.advance(Duration::from_secs(1));
        }
        assert!(p.traits.energy > e);
    }
    #[test]
    fn simulation_is_independent_of_render_partition() {
        let mut a = pet();
        let mut b = pet();
        for _ in 0..1000 {
            a.advance(Duration::from_millis(10));
        }
        for _ in 0..250 {
            b.advance(Duration::from_millis(40));
        }
        assert_eq!(a.snapshot(), b.snapshot());
        assert_eq!(a.activity, b.activity);
        assert_eq!(a.x, b.x);
    }
    #[test]
    fn mood_events_and_denial_priority() {
        let mut p = pet();
        p.event(PetEvent::User("That worked!"));
        p.event(PetEvent::Thinking);
        p.event(PetEvent::Speaking);
        p.event(PetEvent::Done("You're welcome."));
        assert_eq!(p.activity, Activity::Celebrate);
        assert_eq!(p.traits.mood, Mood::Happy);
        p.event(PetEvent::Denied);
        p.event(PetEvent::Cue(EmotionCue {
            emotion: Mood::Happy,
            gesture: Gesture::Celebrate,
            intensity: 0.5,
        }));
        assert!(p.denied);
        assert_eq!(p.activity, Activity::Watch);
        for _ in 0..5 {
            p.advance(Duration::from_secs(1));
        }
        assert!(!p.denied);
    }
    #[test]
    fn selection_respects_configured_weights() {
        let mut p = pet();
        p.config.behaviour_weights = [0, 0, 100, 0, 0, 0, 0, 0, 0, 0];
        assert_eq!(p.select(), Activity::Play);
        p.cooldowns[Activity::Play as usize] = 999;
        assert_eq!(p.select(), Activity::Sit);
    }
}
