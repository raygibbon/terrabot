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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    Stand,
    Crouch,
    Run,
    Seat,
    Curl,
    Stretch,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MotionEvent {
    Step,
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
    pub lean: f32,
    pub lift: f32,
    pub squash: f32,
    pub arms: [(i32, i32); 2],
    pub feet: [(i32, i32); 2],
    pub event: Option<MotionEvent>,
}
#[allow(clippy::too_many_arguments)] // Authored frame table: time, silhouette, pose, contacts, event.
const fn f(
    ms: u32,
    shape: Shape,
    lean: f32,
    lift: f32,
    squash: f32,
    arms: [(i32, i32); 2],
    feet: [(i32, i32); 2],
    event: Option<MotionEvent>,
) -> Frame {
    Frame {
        ms,
        shape,
        lean,
        lift,
        squash,
        arms,
        feet,
        event,
    }
}
const ST: Shape = Shape::Stand;
const SE: Shape = Shape::Seat;
const CR: Shape = Shape::Crouch;
const RN: Shape = Shape::Run;
const CU: Shape = Shape::Curl;
const SX: Shape = Shape::Stretch;
const DOWN: [(i32, i32); 2] = [(0, 19), (28, 19)];
const UP: [(i32, i32); 2] = [(0, 8), (28, 7)];
const FEET: [(i32, i32); 2] = [(7, 27), (20, 27)];
const IDLE: &[Frame] = &[
    f(900, ST, 0., 0., 1., DOWN, FEET, None),
    f(500, ST, -0.4, 0., 1.02, DOWN, FEET, None),
    f(900, ST, 0.4, 0., 1., DOWN, FEET, None),
];
const WALK: &[Frame] = &[
    f(
        120,
        ST,
        0.,
        0.,
        1.,
        [(1, 17), (27, 20)],
        [(7, 27), (20, 25)],
        Some(MotionEvent::Step),
    ),
    f(
        100,
        ST,
        0.,
        0.,
        1.,
        [(1, 18), (27, 18)],
        [(5, 27), (17, 24)],
        None,
    ),
    f(
        120,
        ST,
        0.,
        0.,
        1.,
        [(1, 20), (27, 17)],
        [(6, 25), (19, 27)],
        Some(MotionEvent::Step),
    ),
    f(
        100,
        ST,
        0.,
        0.,
        1.,
        [(1, 18), (27, 18)],
        [(9, 24), (17, 27)],
        None,
    ),
];
const RUN: &[Frame] = &[
    f(
        75,
        RN,
        0.,
        0.,
        0.93,
        [(1, 13), (27, 22)],
        [(3, 27), (24, 23)],
        Some(MotionEvent::Step),
    ),
    f(
        65,
        RN,
        0.,
        2.,
        1.,
        [(0, 15), (27, 20)],
        [(10, 23), (17, 22)],
        None,
    ),
    f(
        75,
        RN,
        0.,
        0.,
        0.93,
        [(0, 22), (28, 13)],
        [(8, 23), (24, 27)],
        Some(MotionEvent::Step),
    ),
    f(
        65,
        RN,
        0.,
        2.,
        1.,
        [(1, 20), (27, 15)],
        [(9, 22), (18, 23)],
        None,
    ),
];
const JUMP: &[Frame] = &[
    f(220, CR, 0., 0., 1.0, [(1, 23), (27, 23)], FEET, None),
    f(100, ST, 0., 2., 1.06, UP, FEET, Some(MotionEvent::Takeoff)),
    f(180, ST, 0., 7., 1., UP, [(6, 24), (21, 24)], None),
    f(
        180,
        ST,
        0.,
        8.,
        1.,
        [(0, 13), (28, 13)],
        [(5, 24), (22, 24)],
        None,
    ),
    f(120, ST, 0., 3., 1., DOWN, FEET, None),
    f(
        130,
        CR,
        0.,
        0.,
        0.94,
        [(0, 23), (28, 23)],
        [(5, 27), (22, 27)],
        Some(MotionEvent::Land),
    ),
    f(220, ST, 0., 0., 1., DOWN, FEET, None),
];
const SIT: &[Frame] = &[
    f(180, CR, 0., 0., 1.0, DOWN, FEET, None),
    f(
        220,
        SE,
        0.,
        0.,
        1.,
        [(1, 22), (27, 22)],
        [(4, 27), (23, 27)],
        None,
    ),
    f(
        1100,
        SE,
        0.,
        0.,
        1.,
        [(4, 23), (24, 23)],
        [(4, 27), (23, 27)],
        None,
    ),
];
const SLEEP: &[Frame] = &[
    f(180, CR, 0., 0., 1., DOWN, FEET, None),
    f(260, SE, 0., 0., 1., [(4, 23), (24, 23)], FEET, None),
    f(400, CU, 0., 0., 1., [(7, 24), (23, 24)], FEET, None),
    f(
        1800,
        CU,
        0.,
        0.,
        1.,
        [(9, 24), (22, 24)],
        [(7, 27), (21, 27)],
        None,
    ),
    f(
        1800,
        CU,
        0.,
        0.,
        1.,
        [(9, 23), (22, 23)],
        [(7, 27), (21, 27)],
        None,
    ),
];
const WAKE: &[Frame] = &[
    f(280, CU, 0., 0., 1., [(9, 24), (22, 24)], FEET, None),
    f(280, SE, 0., 0., 1., [(4, 23), (24, 23)], FEET, None),
    f(220, CR, 0., 0., 1., [(2, 16), (27, 16)], FEET, None),
    f(450, SX, 0., 0., 1., UP, FEET, None),
    f(180, CR, 0., 0., 1., [(0, 13), (28, 13)], FEET, None),
    f(300, ST, 0., 0., 1., DOWN, FEET, None),
];
const THINK: &[Frame] = &[
    f(600, ST, -1., 0., 1., [(3, 14), (26, 19)], FEET, None),
    f(800, ST, 1.5, 0., 1., [(5, 13), (26, 19)], FEET, None),
    f(500, ST, 1., 0., 1., [(4, 14), (26, 19)], FEET, None),
];
const TALK: &[Frame] = &[
    f(220, ST, 0., 0., 1., [(0, 19), (27, 16)], FEET, None),
    f(180, ST, 0.4, 0., 1., [(0, 19), (28, 15)], FEET, None),
    f(260, ST, 0., 0., 1., DOWN, FEET, None),
];
const HAPPY: &[Frame] = &[
    f(160, SE, -1., 0., 1., DOWN, FEET, None),
    f(240, ST, -2., 2., 1., UP, FEET, None),
    f(160, ST, 0., 0., 0.98, DOWN, FEET, None),
    f(240, ST, 2., 2., 1., UP, FEET, None),
];
const SAD: &[Frame] = &[
    f(1100, SE, -1.5, 0., 1., [(2, 24), (26, 24)], FEET, None),
    f(900, SE, -2., 0., 0.98, [(3, 24), (25, 24)], FEET, None),
];
const CURIOUS: &[Frame] = &[
    f(550, ST, -2., 0., 1., DOWN, FEET, None),
    f(700, ST, 2.5, 0., 1., [(0, 19), (28, 13)], FEET, None),
    f(300, ST, 0., 1., 1.02, [(0, 17), (28, 17)], FEET, None),
];
const BORED: &[Frame] = &[
    f(900, SE, -2., 0., 1., DOWN, FEET, None),
    f(500, SE, -1., 0., 1., [(0, 20), (23, 15)], FEET, None),
    f(500, SE, 0., 0., 1., DOWN, [(7, 27), (21, 25)], None),
    f(400, SE, 0., 0., 1., DOWN, FEET, Some(MotionEvent::Step)),
];
const PLAY: &[Frame] = &[
    f(350, ST, 2., 0., 1., DOWN, FEET, None),
    f(320, SE, 3., 0., 1., [(10, 23), (20, 23)], FEET, None),
    f(
        360,
        ST,
        1.,
        0.,
        1.,
        [(10, 21), (27, 22)],
        FEET,
        Some(MotionEvent::Catch),
    ),
    f(220, SE, -1., 0., 1., [(10, 22), (27, 22)], FEET, None),
    f(
        160,
        SX,
        0.,
        0.,
        1.,
        [(10, 6), (20, 6)],
        FEET,
        Some(MotionEvent::Release),
    ),
    f(650, ST, 0., 0., 1., [(8, 10), (22, 10)], FEET, None),
    f(
        240,
        SE,
        1.,
        0.,
        1.,
        [(10, 21), (27, 22)],
        FEET,
        Some(MotionEvent::Catch),
    ),
    f(300, ST, 0., 0., 1., DOWN, FEET, None),
];
const CELEBRATE: &[Frame] = &[
    JUMP[0],
    JUMP[1],
    JUMP[2],
    f(260, ST, -3., 7., 1., UP, [(5, 24), (22, 24)], None),
    f(180, ST, 3., 3., 1., UP, FEET, None),
    JUMP[5],
    f(450, ST, 0., 0., 1., [(0, 9), (28, 14)], FEET, None),
];
const STUMBLE: &[Frame] = &[
    f(160, ST, 2., 0., 1., DOWN, [(9, 27), (23, 25)], None),
    f(210, ST, 7., 1., 0.95, UP, [(12, 27), (25, 24)], None),
    f(
        280,
        SE,
        5.,
        0.,
        1.,
        [(1, 23), (28, 22)],
        FEET,
        Some(MotionEvent::Land),
    ),
    f(250, ST, -3., 0., 1., UP, FEET, None),
    f(300, ST, 0., 0., 1., DOWN, FEET, None),
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
            Sleep => 3,
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
        assert_eq!(a.index, 4);
    }
    #[test]
    fn durations_loops_and_pingpong() {
        let mut t = Timeline::default();
        t.request(ClipId::Walk, true);
        t.advance(Duration::from_millis(119));
        assert_eq!(t.index, 0);
        t.advance(Duration::from_millis(1));
        assert_eq!(t.index, 1);
        t.advance(Duration::from_millis(320));
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
                assert!((0.8..=1.2).contains(&f.squash));
                for (x, y) in f.arms.iter().chain(f.feet.iter()) {
                    assert!((0..=28).contains(x));
                    assert!((0..=27).contains(y));
                }
            }
        }
    }
}
