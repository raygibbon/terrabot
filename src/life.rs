//! Bounded habitat state. The application supplies clock input; simulation stays deterministic.
use crate::pet::Activity;
use serde::{Deserialize, Serialize};
use terragame_core::random::Random;

pub const COUNT: usize = 17;
pub const REST: f32 = 0.12;
pub const PLANT: f32 = 0.72;
pub const CHAT: f32 = 0.94;
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Life {
    pub comfort: f32,
    pub restfulness: f32,
    /// Stable boldness, liveliness and social inclination.
    pub character: [f32; 3],
    pub preferences: [f32; COUNT],
    pub favourite_rest: f32,
    pub favourite_explore: f32,
    pub familiarity: f32,
    pub object_familiarity: f32,
    pub interactions: [u64; 5],
    pub routine: Activity,
    pub position: f32,
}
impl Life {
    pub fn new(seed: u64) -> Self {
        let mut rng = Random::new(seed);
        let mut sample = || 0.25 + rng.below(501).unwrap_or(250) as f32 / 1000.0;
        Self {
            comfort: 0.7,
            restfulness: 0.75,
            character: [sample(), sample(), sample()],
            preferences: [0.5; COUNT],
            favourite_rest: REST,
            favourite_explore: sample(),
            familiarity: 0.0,
            object_familiarity: 0.0,
            interactions: [0; 5],
            routine: Activity::Sit,
            position: 0.5,
        }
    }
    pub fn validate(&self) -> bool {
        self.character
            .iter()
            .chain(self.preferences.iter())
            .copied()
            .chain([
                self.comfort,
                self.restfulness,
                self.favourite_rest,
                self.favourite_explore,
                self.familiarity,
                self.object_familiarity,
                self.position,
            ])
            .all(|v| v.is_finite() && (0.0..=1.0).contains(&v))
    }
}
#[derive(Clone, Copy, Debug)]
pub enum Action {
    Attention,
    Orb,
    Play,
    Rest,
    Wake,
}
#[derive(Clone, Copy, Debug)]
pub struct Orb {
    pub x: f32,
    pub height: f32,
    pub velocity: f32,
    pub vertical: f32,
}
impl Default for Orb {
    fn default() -> Self {
        Self {
            x: 0.4,
            height: 3.0,
            velocity: 0.0,
            vertical: 0.0,
        }
    }
}
impl Orb {
    pub fn step(&mut self) {
        self.x += self.velocity * 0.05;
        if self.x < 0.08 || self.x > 0.92 {
            self.velocity *= -0.8;
            self.x = self.x.clamp(0.08, 0.92);
        }
        self.velocity *= 0.994;
        self.vertical -= 32.0 * 0.05;
        self.height += self.vertical * 0.05;
        if self.height < 3.0 {
            self.height = 3.0;
            self.vertical = -self.vertical * 0.65;
        }
    }
}
/// Called outside the fixed-step engine, at most once a minute. No shell or network access.
#[cfg(unix)]
pub fn local_hour() -> Option<u8> {
    let seconds: libc::time_t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs()
        .try_into()
        .ok()?;
    let mut value = std::mem::MaybeUninit::<libc::tm>::uninit();
    // SAFETY: both pointers reference live, correctly aligned storage; localtime_r
    // initializes the output on success and does not retain either pointer.
    if unsafe { libc::localtime_r(&seconds, value.as_mut_ptr()) }.is_null() {
        return None;
    }
    let hour = unsafe { value.assume_init() }.tm_hour;
    u8::try_from(hour).ok().filter(|h| *h < 24)
}
#[cfg(not(unix))]
pub fn local_hour() -> Option<u8> {
    None
}
