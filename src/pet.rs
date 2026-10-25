//! Pure cosmetic simulation. No I/O, model requests or execution capabilities.
use crate::{
    config::PetConfig,
    cues::{EmotionCue, Gesture},
    life::{Action, COUNT, Life, Orb},
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
    Explore,
    Chase,
    Balance,
    Rest,
    Nap,
    Search,
    Attention,
}
impl Activity {
    pub const ALL: [Self; COUNT] = [
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
        Self::Explore,
        Self::Chase,
        Self::Balance,
        Self::Rest,
        Self::Nap,
        Self::Search,
        Self::Attention,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Wander => "Wandering",
            Self::Sit => "Taking a moment",
            Self::Play => "Playing with light",
            Self::Investigate => "What's over there?",
            Self::Sleep => "Resting",
            Self::Wake => "A little stretch",
            Self::Dance => "A little dance",
            Self::Stumble => "Found my feet",
            Self::Watch => "Here with you",
            Self::Celebrate => "Nicely done!",
            Self::Explore => "Exploring the habitat",
            Self::Chase => "Chasing the orb",
            Self::Balance => "Balancing the orb",
            Self::Rest => "Resting on the cushion",
            Self::Nap => "A little nap",
            Self::Search => "Looking for something to do",
            Self::Attention => "Hello, I'm listening",
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
    pub life: Life,
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
            version: 2,
            life: Life::new(seed),
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
        if self.version != 2 {
            return Err(format!("Unsupported Trixie state version {}", self.version));
        }
        if !self.life.validate() {
            return Err("Invalid life state".into());
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
            return Err("Trixie traits must be finite numbers in 0–1".into());
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
    pub moving: bool,
    pub orb: Orb,
    pub utilities: [f32; COUNT],
    pub autonomous: bool,
    night: f32,
    hour: Option<u8>,
    duration: u64,
    travel_budget: u16,
    learned: bool,
    fixed: FixedStep,
    rng: Random,
    config: PetConfig,
    next_choice: u64,
    cooldowns: [u64; COUNT],
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
        if config.reset_personality {
            traits.life = Life::new(config.seed);
        }
        let rng = Random::new(traits.rng_state);
        // Migrate old habitat coordinates into the compact home area.
        let x = traits.life.position.clamp(0.46, 0.54);
        traits.life.position = x;
        let tick = traits.active_ticks;
        let routine = traits.life.routine;
        let autonomous = config.autonomous;
        let mut pet = Self {
            traits,
            activity: Activity::Sit,
            x,
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
            moving: false,
            orb: Orb::default(),
            utilities: [0.0; COUNT],
            autonomous,
            night: 0.0,
            hour: None,
            duration: 80,
            travel_budget: 20,
            learned: false,
            fixed: FixedStep::new(Duration::from_millis(50), 20)
                .expect("constant valid simulation step"),
            rng,
            config,
            next_choice: tick.saturating_add(80),
            cooldowns: [0; COUNT],
            priority: 0,
            hold_until: 0,
            next_blink: tick.saturating_add(60),
            blink_until: 0,
            target: 0.5,
            success_context: false,
        };
        pet.choose(routine, 0, 120);
        pet
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
        self.orb.step();
        if self.activity == Activity::Chase && self.moving {
            self.target = self.orb.x.clamp(0.46, 0.54);
        }
        let distance = self.target - self.x;
        self.moving = distance.abs() > 0.008;
        if self.moving {
            self.facing = if distance < 0.0 { -1.0 } else { 1.0 };
            let speed = if self.activity == Activity::Chase || self.priority >= 60 {
                0.004
            } else {
                0.002
            };
            self.x = (self.x + distance.clamp(-speed, speed)).clamp(0.0, 1.0);
            if self.travel_budget > 0 {
                self.travel_budget -= 1;
                self.next_choice = tick.saturating_add(self.duration);
                if self.priority == 0 || self.priority == 60 {
                    self.hold_until = self.next_choice;
                }
            }
            self.age = 0.0;
        } else {
            self.facing =
                if self.activity == Activity::Watch || self.activity == Activity::Attention {
                    1.0
                } else {
                    self.facing
                };
            if self.activity == Activity::Balance {
                self.orb.x += (self.x - self.orb.x) * 0.3;
                self.orb.height += (34.0 - self.orb.height) * 0.3;
                self.orb.vertical = 0.0;
                self.orb.velocity = 0.0;
            } else if matches!(self.activity, Activity::Play | Activity::Chase)
                && self.orb.height <= 3.1
            {
                self.orb.vertical = 25.0;
                self.orb.velocity = if self.activity == Activity::Chase {
                    -self.facing * 0.14
                } else {
                    self.facing * 0.025
                };
                if self.activity == Activity::Chase && self.age > 2.0 {
                    self.target = self.orb.x.clamp(0.46, 0.54);
                }
            }
        }
        self.traits.life.position = self.x;
        if tick.is_multiple_of(20) {
            self.needs();
        }
        if self.thinking || self.speaking {
            return;
        }
        if tick >= self.hold_until {
            self.priority = 0;
            self.denied = false;
            self.gesture = Gesture::None;
        }
        if self.autonomous && self.priority == 0 && tick >= self.next_choice {
            if self.activity == Activity::Sleep && self.traits.energy < 0.82 {
                self.next_choice = tick.saturating_add(100);
                return;
            }
            if self.activity == Activity::Sleep {
                self.choose(Activity::Wake, 0, 60);
                return;
            }
            let activity = self.select();
            let duration = match activity {
                Activity::Sleep | Activity::Nap | Activity::Rest => 400,
                Activity::Wake => 80,
                _ => 180 + self.rng.below(180).unwrap_or(0),
            };
            self.choose(activity, 0, duration);
        }
    }
    pub fn set_hour(&mut self, hour: Option<u8>) {
        self.hour = hour.filter(|h| *h < 24);
    }
    pub fn is_night(&self) -> bool {
        self.config.daily_routine && self.hour.is_some_and(|h| !(7..22).contains(&h))
    }
    fn needs(&mut self) {
        let resting = !self.moving
            && matches!(
                self.activity,
                Activity::Sleep | Activity::Nap | Activity::Rest | Activity::Sit
            );
        let playing = !self.moving
            && matches!(
                self.activity,
                Activity::Play | Activity::Chase | Activity::Balance | Activity::Dance
            );
        let exploring = matches!(
            self.activity,
            Activity::Explore | Activity::Investigate | Activity::Wander
        );
        self.night += (if self.is_night() { 1.0 } else { 0.0 } - self.night) * 0.03;
        self.traits.energy = (self.traits.energy
            + if resting {
                0.006
            } else if self.moving || playing {
                -0.003
            } else {
                -0.001
            })
        .clamp(0.0, 1.0);
        self.traits.boredom = (self.traits.boredom
            + if playing || exploring || self.speaking {
                -0.012
            } else {
                0.004
            })
        .clamp(0.0, 1.0);
        self.traits.curiosity =
            (self.traits.curiosity + if exploring { -0.005 } else { 0.002 }).clamp(0.0, 1.0);
        self.traits.playfulness =
            (self.traits.playfulness + if playing { -0.005 } else { 0.0015 }).clamp(0.0, 1.0);
        self.traits.sociability = (self.traits.sociability
            + if self.speaking || self.activity == Activity::Attention {
                -0.005
            } else {
                0.001
            })
        .clamp(0.0, 1.0);
        let l = &mut self.traits.life;
        l.comfort = (l.comfort + if resting { 0.006 } else { -0.003 }).clamp(0.0, 1.0);
        l.restfulness = (l.restfulness + if resting { 0.006 } else { -0.002 }).clamp(0.0, 1.0);
        if self.activity == Activity::Investigate && !self.moving {
            l.object_familiarity = (l.object_familiarity + 0.005).min(1.0);
        }
        if !self.moving && self.age >= 3.0 && !self.learned && self.config.develop_preferences {
            self.learned = true;
            for p in &mut l.preferences {
                *p += (0.5 - *p) * 0.001;
            }
            let pref = &mut l.preferences[self.activity as usize];
            *pref = (*pref + 0.003 * (1.0 - *pref)).min(0.85);
            if exploring {
                l.favourite_explore += (self.x - l.favourite_explore) * 0.01;
            }
            if resting {
                l.favourite_rest += (self.x - l.favourite_rest) * 0.01;
            }
        }
    }
    fn select(&mut self) -> Activity {
        if self.traits.energy < 0.18 {
            self.traits.mood = Mood::Sleepy;
            return Activity::Sleep;
        }
        let s = &self.traits;
        let l = &s.life;
        let mut scores = [0.0; COUNT];
        for (i, a) in Activity::ALL.iter().enumerate() {
            let category = match a {
                Activity::Explore => 0,
                Activity::Chase | Activity::Balance => 2,
                Activity::Rest => 1,
                Activity::Nap => 4,
                Activity::Search => 3,
                Activity::Attention => 8,
                _ => i,
            };
            if (category == 2 && self.cooldowns[Activity::Play as usize] > s.active_ticks)
                || self.cooldowns[i] > s.active_ticks
                || *a == self.activity
                || self.config.behaviour_weights[category] == 0
            {
                continue;
            }
            if self.gentle
                && matches!(
                    a,
                    Activity::Play
                        | Activity::Chase
                        | Activity::Balance
                        | Activity::Dance
                        | Activity::Celebrate
                        | Activity::Stumble
                )
            {
                continue;
            }
            if *a == Activity::Celebrate && !self.success_context {
                continue;
            }
            if *a == Activity::Wake && !matches!(self.activity, Activity::Sleep | Activity::Nap) {
                continue;
            }
            let need = match a {
                Activity::Sleep | Activity::Nap => {
                    (1.0 - s.energy) * 70.0
                        + (1.0 - l.restfulness) * 25.0
                        + self.night * if *a == Activity::Sleep { 80.0 } else { 55.0 }
                }
                Activity::Rest | Activity::Sit => {
                    (1.0 - l.comfort) * 70.0 + (1.0 - s.energy) * 30.0
                }
                Activity::Play | Activity::Chase | Activity::Balance => {
                    s.boredom * 30.0 + s.playfulness * 25.0 + l.character[1] * 12.0
                }
                Activity::Explore | Activity::Wander => {
                    s.curiosity * 30.0 + s.boredom * 20.0 + l.character[0] * 12.0
                }
                Activity::Investigate => s.curiosity * 25.0 + (1.0 - l.object_familiarity) * 20.0,
                Activity::Watch | Activity::Attention => {
                    s.sociability * 30.0 + l.character[2] * 10.0
                }
                Activity::Dance => s.playfulness * 30.0 + s.energy * 10.0,
                Activity::Search => s.boredom * 25.0 + (1.0 - s.curiosity) * 25.0,
                Activity::Wake | Activity::Celebrate | Activity::Stumble => 0.0,
            };
            if matches!(a, Activity::Sleep | Activity::Nap) && s.energy > 0.75 && self.night < 0.5 {
                continue;
            }
            scores[i] =
                (need + l.preferences[i] * 15.0 + if s.preferred.contains(a) { 5.0 } else { 0.0 })
                    * (0.75 + (self.config.behaviour_weights[category] as f32 / 80.0).min(3.0));
        }
        self.utilities = scores;
        let mut best = Activity::Sit;
        let mut value = 0.0;
        for (i, score) in scores.iter().enumerate() {
            if *score <= 0.0 {
                continue;
            }
            let score = score + self.rng.below(120).unwrap_or(0) as f32 / 10.0;
            if score > value {
                value = score;
                best = Activity::ALL[i];
            }
        }
        best
    }
    pub fn interact(&mut self, action: Action) {
        let i = action as usize;
        self.traits.life.interactions[i] = self.traits.life.interactions[i].saturating_add(1);
        self.traits.life.familiarity = (self.traits.life.familiarity + 0.002).min(1.0);
        if self.thinking || self.speaking {
            return;
        }
        self.priority = 0;
        let activity = match action {
            Action::Attention => Activity::Attention,
            Action::Orb => Activity::Balance,
            Action::Play => Activity::Chase,
            Action::Rest => Activity::Rest,
            Action::Wake => Activity::Wake,
        };
        if self.gentle && matches!(action, Action::Play | Action::Orb) {
            self.choose(Activity::Attention, 60, 100);
        } else {
            self.choose(activity, 60, 200);
        }
    }
    pub fn reset_preferences(&mut self) {
        self.traits.life = Life::new(self.config.seed);
        self.traits.life.position = self.x;
    }
    pub fn choose(&mut self, activity: Activity, priority: u8, duration: u64) {
        if priority < self.priority {
            return;
        }
        if self.activity != activity {
            self.cooldowns[self.activity as usize] = self.traits.active_ticks.saturating_add(600);
            self.age = 0.0;
        }
        if self.activity != activity {
            self.learned = false;
        }
        self.activity = activity;
        self.traits.life.routine = activity;
        self.duration = duration;
        self.travel_budget = 20;
        self.priority = priority;
        self.hold_until = self.traits.active_ticks.saturating_add(duration);
        self.next_choice = self.hold_until;
        // Needs and preferences remain authoritative; only local shuffles travel.
        self.target = match activity {
            Activity::Wander => 0.5 + (self.rng.below(81).unwrap_or(40) as f32 - 40.0) / 1000.0,
            Activity::Chase => self.orb.x.clamp(0.46, 0.54),
            _ => self.x,
        };
        self.moving = (self.target - self.x).abs() > 0.008;
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
                let was_sleep = matches!(self.activity, Activity::Sleep | Activity::Nap);
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
                    && ["hello", "hi trixie"]
                        .iter()
                        .any(|s| text.to_lowercase().contains(s))
                {
                    self.gesture = Gesture::Wave;
                }
                self.traits.life.familiarity = (self.traits.life.familiarity + 0.002).min(1.0);
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
    fn autonomous_travel_stays_near_home_without_renderer_overrides() {
        let mut old = Snapshot::new(7);
        old.life.position = 0.95;
        let mut p = Pet::new(PetConfig::default(), old);
        for _ in 0..24000 {
            let before = p.x;
            p.advance(Duration::from_millis(50));
            assert!((0.46..=0.54).contains(&p.x));
            assert!((p.x - before).abs() <= 0.0041);
            assert_eq!(p.snapshot().life.position, p.x);
        }
        for activity in [
            Activity::Rest,
            Activity::Sleep,
            Activity::Explore,
            Activity::Attention,
        ] {
            p.demo_activity(activity);
            let before = p.x;
            p.advance(Duration::from_millis(50));
            assert_eq!(p.x, before);
            assert!(!p.moving);
        }
    }
    #[test]
    fn chase_cannot_extend_itself_forever() {
        let mut p = pet();
        p.interact(Action::Play);
        let mut left_chase = false;
        for _ in 0..400 {
            p.advance(Duration::from_millis(50));
            left_chase |= p.activity != Activity::Chase;
        }
        assert!(
            left_chase,
            "chase must finish within its bounded travel/interaction budget"
        );
        assert!(p.traits.energy > 0.1);
    }
    #[test]
    fn day_night_bias_and_disabled_learning() {
        let mut day = pet();
        let mut night = pet();
        day.traits.energy = 0.6;
        night.traits.energy = 0.6;
        night.night = 1.0;
        day.select();
        night.select();
        assert!(night.utilities[Activity::Nap as usize] > day.utilities[Activity::Nap as usize]);
        day.config.develop_preferences = false;
        let before = day.traits.life.preferences;
        day.interact(Action::Rest);
        for _ in 0..200 {
            day.advance(Duration::from_millis(50));
        }
        assert_eq!(day.traits.life.preferences, before);
    }
    #[test]
    fn rest_and_orb_interactions_stay_in_place() {
        let mut p = pet();
        p.interact(Action::Rest);
        let before = p.x;
        for _ in 0..80 {
            p.advance(Duration::from_millis(50));
        }
        assert!(!p.moving);
        assert_eq!(p.x, before);
        p.interact(Action::Orb);
        for _ in 0..120 {
            p.advance(Duration::from_millis(50));
        }
        assert_eq!(p.activity, Activity::Balance);
        assert!(!p.moving);
        assert!(p.orb.height > 30.0);
    }
    #[test]
    fn needs_routine_and_learning_are_bounded() {
        let mut p = pet();
        p.set_hour(Some(23));
        assert!(p.is_night());
        p.config.daily_routine = false;
        assert!(!p.is_night());
        p.config.daily_routine = true;
        p.x = p.traits.life.favourite_rest;
        p.traits.energy = 0.3;
        p.traits.life.comfort = 0.1;
        p.interact(Action::Rest);
        let preferences = p.traits.life.preferences;
        for _ in 0..100 {
            p.advance(Duration::from_millis(50));
        }
        assert!(p.traits.energy > 0.3);
        assert!(p.traits.life.comfort > 0.1);
        assert!(
            p.traits.life.preferences[Activity::Rest as usize]
                > preferences[Activity::Rest as usize]
        );
        assert!(
            p.traits.life.preferences[Activity::Rest as usize]
                - preferences[Activity::Rest as usize]
                < 0.01
        );
        p.event(PetEvent::Typing);
        assert!(matches!(p.activity, Activity::Watch | Activity::Wake));
        p.traits.validate().unwrap();
        p.reset_preferences();
        assert_eq!(p.traits.life.preferences, [0.5; COUNT]);
    }
    #[test]
    fn conversation_preempts_life_and_pause_prevents_selection() {
        let mut p = pet();
        p.interact(Action::Play);
        p.event(PetEvent::User("I am grieving and sad"));
        p.event(PetEvent::Thinking);
        p.interact(Action::Orb);
        assert_eq!(p.activity, Activity::Watch);
        assert!(p.thinking);
        p.event(PetEvent::Done("I am here."));
        for _ in 0..600 {
            p.advance(Duration::from_millis(50));
        }
        assert!(!matches!(
            p.activity,
            Activity::Play | Activity::Chase | Activity::Dance | Activity::Celebrate
        ));
        p.autonomous = false;
        p.interact(Action::Rest);
        for _ in 0..1000 {
            p.advance(Duration::from_millis(50));
        }
        assert_eq!(p.activity, Activity::Rest);
    }
    #[test]
    fn utilities_respond_to_needs_and_cooldowns() {
        let mut p = pet();
        p.traits.energy = 0.1;
        assert_eq!(p.select(), Activity::Sleep);
        p.traits.energy = 0.9;
        p.traits.curiosity = 1.0;
        p.traits.boredom = 1.0;
        let first = p.select();
        p.choose(first, 0, 100);
        assert_ne!(p.select(), first);
        p.choose(Activity::Sit, 0, 100);
        assert_ne!(p.select(), first);
        assert!(p.utilities.iter().all(|v| v.is_finite() && *v >= 0.0));
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
