//! Authored presentation timelines. No simulation state or I/O is mutated here.
use crate::pet::{Activity, Mood, Pet};
use std::time::Duration;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClipId {
    Idle,
    Walk,
    Run,
    Jump,
    Sit,
    Sleep,
    Wake,
    Think,
    Talk,
    Happy,
    Sad,
    Curious,
    Bored,
    Play,
    Celebrate,
    Stumble,
}
impl ClipId {
    pub const ALL: [Self; 16] = [
        Self::Idle,
        Self::Walk,
        Self::Run,
        Self::Jump,
        Self::Sit,
        Self::Sleep,
        Self::Wake,
        Self::Think,
        Self::Talk,
        Self::Happy,
        Self::Sad,
        Self::Curious,
        Self::Bored,
        Self::Play,
        Self::Celebrate,
        Self::Stumble,
    ];
    pub fn for_pet(p: &Pet) -> Self {
        if p.speaking {
            return Self::Talk;
        }
        if p.thinking {
            return Self::Think;
        }
        if !p.gentle {
            match p.gesture {
                crate::cues::Gesture::Wave | crate::cues::Gesture::HeadTilt => {
                    return Self::Curious;
                }
                crate::cues::Gesture::Nod => return Self::Sit,
                crate::cues::Gesture::Celebrate => return Self::Celebrate,
                crate::cues::Gesture::None => {}
            }
        }
        if p.moving {
            return Self::Walk;
        }
        match p.activity {
            Activity::Sleep | Activity::Nap => Self::Sleep,
            Activity::Sit | Activity::Rest => Self::Sit,
            Activity::Wake => Self::Wake,
            Activity::Play | Activity::Balance | Activity::Chase => Self::Play,
            Activity::Dance => Self::Happy,
            Activity::Celebrate => Self::Celebrate,
            Activity::Stumble => Self::Stumble,
            Activity::Investigate | Activity::Explore | Activity::Search => Self::Curious,
            _ if p.gentle => Self::Sad,
            _ if p.traits.mood == Mood::Bored => Self::Bored,
            _ => Self::Idle,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Playback {
    Loop,
    Once,
    PingPong,
}
pub use crate::art::Shape;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MotionEvent {
    Takeoff,
    Land,
    Release,
    Catch,
    Complete,
}
#[derive(Clone, Copy, Debug)]
pub struct Frame {
    pub ms: u32,
    pub shape: Shape,
    pub lift: f32,
    pub event: Option<MotionEvent>,
}
const fn f(ms: u32, shape: Shape, lift: f32, event: Option<MotionEvent>) -> Frame {
    Frame {
        ms,
        shape,
        lift,
        event,
    }
}
use Shape::{
    Droop as D, Hold as H, Peek as P, Reach as E, Relax as R, Rise as U, Sleep as S, Stretch as X,
};
const IDLE: &[Frame] = &[
    f(1800, R, 0., None),
    f(800, D, 0., None),
    f(1800, R, 0., None),
];
// Legacy Walk/Run names now mean a local glide: no steps or foot contacts.
const WALK: &[Frame] = &[
    f(700, R, 0., None),
    f(600, E, 1., None),
    f(900, R, 0., None),
];
const RUN: &[Frame] = &[
    f(400, E, 1., None),
    f(500, U, 2., None),
    f(800, R, 0., None),
];
const JUMP: &[Frame] = &[
    f(300, D, 0., None),
    f(250, U, 2., Some(MotionEvent::Takeoff)),
    f(350, U, 3., None),
    f(300, R, 1., Some(MotionEvent::Land)),
    f(600, R, 0., None),
];
const SIT: &[Frame] = &[
    f(400, R, 0., None),
    f(700, D, 0., None),
    f(1800, S, 0., None),
];
const SLEEP: &[Frame] = &[
    f(500, D, 0., None),
    f(700, S, 0., None),
    f(2200, S, 0., None),
    f(2200, D, 0., None),
];
const WAKE: &[Frame] = &[
    f(500, S, 0., None),
    f(450, D, 0., None),
    f(500, E, 1., None),
    f(500, X, 2., None),
    f(400, U, 1., None),
    f(1000, R, 0., None),
];
const THINK: &[Frame] = &[
    f(900, P, 0., None),
    f(1200, E, 0., None),
    f(900, P, 0., None),
];
const TALK: &[Frame] = &[
    f(500, R, 0., None),
    f(450, E, 0., None),
    f(900, R, 0., None),
];
const HAPPY: &[Frame] = &[
    f(300, D, 0., None),
    f(500, U, 2., None),
    f(400, E, 1., None),
    f(1400, R, 0., None),
];
const SAD: &[Frame] = &[f(1600, D, 0., None), f(1800, S, 0., None)];
const CURIOUS: &[Frame] = &[
    f(900, R, 0., None),
    f(1600, P, 1., None),
    f(1000, E, 0., None),
    f(1300, R, 0., None),
];
const BORED: &[Frame] = &[
    f(1400, D, 0., None),
    f(900, S, 0., None),
    f(700, P, 0., None),
    f(1500, D, 0., None),
];
const PLAY: &[Frame] = &[
    f(900, P, 0., None),
    f(600, E, 0., None),
    f(900, H, 0., Some(MotionEvent::Catch)),
    f(350, H, 1., None),
    f(250, U, 2., Some(MotionEvent::Release)),
    f(1100, P, 0., None),
    f(650, H, 0., Some(MotionEvent::Catch)),
    f(1400, R, 0., None),
];
const CELEBRATE: &[Frame] = &[
    f(300, D, 0., None),
    f(350, E, 1., None),
    f(650, U, 3., None),
    f(500, X, 2., None),
    f(450, E, 1., None),
    f(1200, R, 0., None),
];
const STUMBLE: &[Frame] = &[
    f(300, E, 0., None),
    f(400, D, 0., None),
    f(600, R, 0., None),
];
#[derive(Clone, Copy)]
pub struct Clip {
    pub frames: &'static [Frame],
    pub playback: Playback,
    pub priority: u8,
    pub interruptible: bool,
    pub loop_from: usize,
}
pub fn clip(id: ClipId) -> Clip {
    use ClipId::*;
    let frames = match id {
        Idle => IDLE,
        Walk => WALK,
        Run => RUN,
        Jump => JUMP,
        Sit => SIT,
        Sleep => SLEEP,
        Wake => WAKE,
        Think => THINK,
        Talk => TALK,
        Happy => HAPPY,
        Sad => SAD,
        Curious => CURIOUS,
        Bored => BORED,
        Play => PLAY,
        Celebrate => CELEBRATE,
        Stumble => STUMBLE,
    };
    Clip {
        frames,
        playback: match id {
            Jump | Wake | Celebrate | Stumble => Playback::Once,
            Think | Curious | Sad => Playback::PingPong,
            _ => Playback::Loop,
        },
        priority: match id {
            Talk => 90,
            Think => 80,
            Jump | Stumble => 40,
            Wake | Celebrate => 30,
            _ => 10,
        },
        interruptible: !matches!(id, Jump | Stumble | Wake | Celebrate),
        loop_from: match id {
            Sit => 2,
            Sleep => 2,
            _ => 0,
        },
    }
}
pub struct Timeline {
    pub id: ClipId,
    pub index: usize,
    pub elapsed_ms: u32,
    pub complete: bool,
    reverse: bool,
    sub_ms: u32,
    pub total_ms: u64,
    events: [Option<MotionEvent>; 8],
    count: usize,
}
impl Default for Timeline {
    fn default() -> Self {
        Self {
            id: ClipId::Idle,
            index: 0,
            elapsed_ms: 0,
            complete: false,
            reverse: false,
            sub_ms: 0,
            total_ms: 0,
            events: [None; 8],
            count: 0,
        }
    }
}
impl Timeline {
    pub fn request(&mut self, id: ClipId, force: bool) -> bool {
        if self.id == id {
            return false;
        }
        let old = clip(self.id);
        if !force && !self.complete && !old.interruptible && clip(id).priority <= old.priority {
            return false;
        }
        let gait = |id| matches!(id, ClipId::Walk | ClipId::Run);
        let phase = if !force && gait(self.id) && gait(id) {
            let total: u32 = old.frames.iter().map(|f| f.ms).sum();
            let elapsed: u32 = old
                .frames
                .iter()
                .take(self.index)
                .map(|f| f.ms)
                .sum::<u32>()
                + self.elapsed_ms;
            Some(elapsed as f64 / total as f64)
        } else {
            None
        };
        *self = Self {
            id,
            ..Self::default()
        };
        if let Some(phase) = phase {
            let total: u32 = clip(id).frames.iter().map(|f| f.ms).sum();
            self.advance(Duration::from_secs_f64(phase * f64::from(total) / 1000.0));
            self.count = 0;
            self.events = [None; 8];
        }
        true
    }
    /// Gait timing follows normalized travel, not render-frame count.
    pub fn advance_distance(&mut self, distance: f32, stride: f32) {
        let cycle: u32 = clip(self.id).frames.iter().map(|f| f.ms).sum();
        self.advance(Duration::from_secs_f64(
            f64::from(distance.abs() / stride) * f64::from(cycle) / 1000.0,
        ));
    }
    pub fn frame(&self) -> Frame {
        clip(self.id).frames[self.index]
    }
    fn emit(&mut self, event: MotionEvent) {
        if self.count < 8 {
            self.events[self.count] = Some(event);
            self.count += 1;
        }
    }
    pub fn drain_events(&mut self) -> impl Iterator<Item = MotionEvent> {
        let events = self.events;
        self.events = [None; 8];
        self.count = 0;
        events.into_iter().flatten()
    }
    pub fn advance(&mut self, dt: Duration) {
        let nanos = dt.as_nanos().min(1_000_000_000) + u128::from(self.sub_ms);
        let mut remaining = (nanos / 1_000_000) as u32;
        self.sub_ms = (nanos % 1_000_000) as u32;
        self.total_ms = self.total_ms.saturating_add(u64::from(remaining));
        let c = clip(self.id);
        while remaining > 0 && !self.complete {
            let spent = remaining.min(self.frame().ms - self.elapsed_ms);
            remaining -= spent;
            self.elapsed_ms += spent;
            if self.elapsed_ms < self.frame().ms {
                break;
            }
            self.elapsed_ms = 0;
            match c.playback {
                Playback::Once if self.index + 1 == c.frames.len() => {
                    self.complete = true;
                    self.emit(MotionEvent::Complete);
                    break;
                }
                Playback::PingPong => {
                    if self.index + 1 == c.frames.len() {
                        self.reverse = true;
                    }
                    if self.index == 0 {
                        self.reverse = false;
                    }
                    self.index = if self.reverse {
                        self.index - 1
                    } else {
                        self.index + 1
                    };
                }
                _ => {
                    self.index = if self.index + 1 == c.frames.len() {
                        c.loop_from
                    } else {
                        self.index + 1
                    }
                }
            }
            if let Some(e) = self.frame().event {
                self.emit(e);
            }
        }
    }
    pub fn step_frame(&mut self) {
        let mut ms = self.frame().ms - self.elapsed_ms;
        while ms > 0 {
            let chunk = ms.min(1000);
            self.advance(Duration::from_millis(u64::from(chunk)));
            ms -= chunk;
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Expression {
    Neutral,
    Happy,
    Sad,
    Curious,
    Sleepy,
    Surprised,
    Squint,
    Wink,
    Talk,
    Yawn,
}
impl Expression {
    pub const ALL: [Self; 10] = [
        Self::Neutral,
        Self::Happy,
        Self::Sad,
        Self::Curious,
        Self::Sleepy,
        Self::Surprised,
        Self::Squint,
        Self::Wink,
        Self::Talk,
        Self::Yawn,
    ];
}
pub struct Face {
    pub expression: Expression,
    pub override_expression: Option<Expression>,
    pub clock: f32,
    pub gaze: f32,
}
impl Default for Face {
    fn default() -> Self {
        Self {
            expression: Expression::Neutral,
            override_expression: None,
            clock: 0.,
            gaze: 0.,
        }
    }
}
impl Face {
    pub fn advance(&mut self, p: &Pet, id: ClipId, dt: f32) {
        self.clock = (self.clock + dt) % 3600.;
        self.gaze += (if p.thinking || p.speaking {
            1.
        } else {
            (p.orb.x - p.x) * 4.
        }
        .clamp(-1., 1.)
            - self.gaze)
            * (dt * 8.).min(1.);
        self.expression = self.override_expression.unwrap_or(if p.gentle {
            Expression::Sad
        } else {
            match id {
                ClipId::Sleep => Expression::Sleepy,
                ClipId::Talk => Expression::Talk,
                ClipId::Think => Expression::Squint,
                ClipId::Happy | ClipId::Celebrate => Expression::Happy,
                ClipId::Sad => Expression::Sad,
                ClipId::Curious | ClipId::Play => Expression::Curious,
                ClipId::Bored => Expression::Yawn,
                _ => match p.traits.mood {
                    Mood::Happy | Mood::Excited => Expression::Happy,
                    Mood::Surprised => Expression::Surprised,
                    Mood::Confused | Mood::Curious => Expression::Curious,
                    Mood::Focused | Mood::Frustrated => Expression::Squint,
                    Mood::Sleepy => Expression::Sleepy,
                    Mood::Bored => Expression::Yawn,
                    _ => Expression::Neutral,
                },
            }
        });
    }
    pub fn blink(&self) -> bool {
        self.clock % 4.7 < 0.12 && self.clock > 0.5
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cues_select_authored_clips_but_conversation_keeps_priority() {
        let mut p = Pet::new(
            crate::config::PetConfig::default(),
            crate::pet::Snapshot::new(7),
        );
        p.gesture = crate::cues::Gesture::Wave;
        assert_eq!(ClipId::for_pet(&p), ClipId::Curious);
        p.thinking = true;
        assert_eq!(ClipId::for_pet(&p), ClipId::Think);
        p.speaking = true;
        assert_eq!(ClipId::for_pet(&p), ClipId::Talk);
    }
    #[test]
    fn play_events_match_authored_holding_frames() {
        let mut t = Timeline::default();
        t.request(ClipId::Play, true);
        let mut events = Vec::new();
        for _ in 0..600 {
            t.advance(Duration::from_millis(10));
            let shape = t.frame().shape;
            for event in t.drain_events() {
                if event == MotionEvent::Catch {
                    assert_eq!(shape, Shape::Hold);
                }
                events.push(event);
            }
        }
        assert_eq!(
            events,
            vec![MotionEvent::Catch, MotionEvent::Release, MotionEvent::Catch]
        );
    }
    #[test]
    fn gait_distance_is_independent_of_frame_partition() {
        let mut a = Timeline::default();
        let mut b = Timeline::default();
        a.request(ClipId::Walk, true);
        b.request(ClipId::Walk, true);
        for _ in 0..20 {
            a.advance_distance(0.003, 0.08);
        }
        for _ in 0..10 {
            b.advance_distance(0.006, 0.08);
        }
        assert_eq!((a.index, a.elapsed_ms), (b.index, b.elapsed_ms));
        let index = a.index;
        a.request(ClipId::Run, false);
        assert_eq!(a.index, index);
        a.advance_distance(0.0, 0.1);
        assert_eq!(a.index, index);
    }
    #[test]
    fn fractional_frames_and_long_frame_step() {
        let mut a = Timeline::default();
        let mut b = Timeline::default();
        for _ in 0..60 {
            a.advance(Duration::from_nanos(16_666_667));
        }
        b.advance(Duration::from_secs(1));
        assert_eq!((a.index, a.elapsed_ms), (b.index, b.elapsed_ms));
        a.request(ClipId::Sleep, true);
        a.step_frame();
        a.step_frame();
        assert_eq!(a.index, 2);
        a.step_frame();
        assert_eq!(a.index, 3);
        a.step_frame();
        assert_eq!(a.index, 2);
    }
    #[test]
    fn durations_loops_and_pingpong() {
        let mut t = Timeline::default();
        t.request(ClipId::Walk, true);
        t.advance(Duration::from_millis(699));
        assert_eq!(t.index, 0);
        t.advance(Duration::from_millis(1));
        assert_eq!(t.index, 1);
        t.advance(Duration::from_millis(600));
        t.advance(Duration::from_millis(900));
        assert_eq!(t.index, 0);
        t.request(ClipId::Think, true);
        for _ in 0..3 {
            t.step_frame();
        }
        assert_eq!(t.index, 1);
    }
    #[test]
    fn one_shot_priority_and_events() {
        let mut t = Timeline::default();
        t.request(ClipId::Jump, true);
        assert!(!t.request(ClipId::Idle, false));
        let mut events = Vec::new();
        for _ in 0..20 {
            t.advance(Duration::from_millis(100));
            events.extend(t.drain_events());
        }
        assert!(t.complete);
        assert!(events.contains(&MotionEvent::Land));
        assert_eq!(
            events
                .iter()
                .filter(|e| **e == MotionEvent::Complete)
                .count(),
            1
        );
        t.request(ClipId::Jump, true);
        assert!(t.request(ClipId::Talk, false));
    }
    #[test]
    fn all_frames_have_safe_authored_bounds() {
        for id in ClipId::ALL {
            let c = clip(id);
            assert!(c.frames.len() >= 2);
            for f in c.frames {
                assert!(f.ms > 0);
                let m = crate::art::model(f.shape);
                assert_eq!(m.origin, crate::art::ORIGIN);
                assert!((0.0..=3.0).contains(&f.lift));
            }
        }
    }
}
