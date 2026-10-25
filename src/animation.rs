//! Handcrafted pixel layers composed into a bounded logical grid, then TerraBox cells.
use crate::{
    art::{self, Point},
    config::PetConfig,
    motion::{ClipId, Expression, Face, MotionEvent, Timeline},
    pet::{Activity, Pet, PetEvent, Snapshot},
};
use std::{io, time::Duration};
use terrabox::{Canvas, Color, Style};
use terratui::{Theme, Widget};

/// Borrowed frame sequence, reusable for full sprites or independent layers.
pub struct AnimationClip<'a> {
    frames: &'a [&'a [&'a str]],
    frame_time: Duration,
    looping: bool,
}
impl<'a> AnimationClip<'a> {
    pub fn new(frames: &'a [&'a [&'a str]], frame_time: Duration, looping: bool) -> Option<Self> {
        if frames.is_empty() || frame_time.is_zero() {
            return None;
        }
        Some(Self {
            frames,
            frame_time,
            looping,
        })
    }
    pub fn frame(&self, elapsed: Duration) -> &'a [&'a str] {
        let n = elapsed.as_nanos() / self.frame_time.as_nanos();
        let index = if self.looping {
            n % self.frames.len() as u128
        } else {
            n.min(self.frames.len() as u128 - 1)
        };
        self.frames[index as usize]
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Palette {
    TrueColor,
    Indexed,
    Ascii,
}
fn ink(c: char) -> Option<Color> {
    Some(match c {
        'o' => Color::Rgb(51, 42, 79),
        'a' => Color::Rgb(100, 80, 144),
        'b' => Color::Rgb(146, 112, 183),
        'c' => Color::Rgb(190, 157, 219),
        'd' => Color::Rgb(232, 211, 247),
        'f' => Color::Rgb(225, 209, 239),
        'p' => Color::Rgb(246, 157, 171),
        'e' => Color::Rgb(111, 226, 208),
        _ => return None,
    })
}
#[derive(Clone)]
pub struct Sprite {
    pub width: usize,
    pub height: usize,
    pixels: Vec<Option<Color>>,
}
impl Sprite {
    pub fn new(width: usize, height: usize) -> Option<Self> {
        if width > 160 || height > 80 {
            return None;
        }
        Some(Self {
            width,
            height,
            pixels: vec![None; width.checked_mul(height)?],
        })
    }
    pub fn put(&mut self, x: i32, y: i32, color: Color) {
        if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
            self.pixels[y as usize * self.width + x as usize] = Some(color);
        }
    }
    pub fn get(&self, x: usize, y: usize) -> Option<Color> {
        if x < self.width && y < self.height {
            self.pixels[y * self.width + x]
        } else {
            None
        }
    }
    fn art(&mut self, rows: &[&str], x: i32, y: i32) {
        for (dy, row) in rows.iter().enumerate() {
            for (dx, ch) in row.chars().enumerate() {
                if let Some(c) = ink(ch) {
                    self.put(x + dx as i32, y + dy as i32, c);
                }
            }
        }
    }
    fn line(&mut self, x: i32, y: i32, width: i32, color: Color) {
        for dx in 0..width {
            self.put(x + dx, y, color);
        }
    }
    /// One inside boundary on the completed opacity mask, never one per layer.
    fn outline(&mut self) {
        let mut mask = [false; art::WIDTH * art::HEIGHT];
        assert!(self.pixels.len() <= mask.len());
        for (dst, pixel) in mask.iter_mut().zip(&self.pixels) {
            *dst = pixel.is_some();
        }
        for y in 0..self.height {
            for x in 0..self.width {
                let i = y * self.width + x;
                if !mask[i] {
                    continue;
                }
                let edge = [(-1, 0), (1, 0), (0, -1), (0, 1)].iter().any(|(dx, dy)| {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    nx < 0
                        || ny < 0
                        || nx >= self.width as i32
                        || ny >= self.height as i32
                        || !mask[ny as usize * self.width + nx as usize]
                });
                if edge {
                    self.pixels[i] = ink('o');
                } else if self.pixels[i] == ink('o') {
                    self.pixels[i] = ink('a');
                }
            }
        }
    }
    pub fn draw(
        &self,
        c: &mut Canvas<'_>,
        x0: u16,
        y0: u16,
        scale: usize,
        palette: Palette,
        bg: Color,
    ) {
        let scale = scale.clamp(1, 2);
        for row in 0..self.height.div_ceil(2 * scale) {
            for col in 0..self.width.div_ceil(scale) {
                let top = self.get(col * scale, row * 2 * scale).unwrap_or(bg);
                let bottom = self.get(col * scale, (row * 2 + 1) * scale).unwrap_or(bg);
                let (glyph, fg, back) = if palette == Palette::Ascii {
                    (' ', bg, if top != bg { top } else { bottom })
                } else {
                    ('▀', top, bottom)
                };
                let convert = |color| {
                    if palette == Palette::TrueColor {
                        color
                    } else {
                        indexed(color)
                    }
                };
                c.put_char(
                    x0.saturating_add(col as u16),
                    y0.saturating_add(row as u16),
                    glyph,
                    Style::new(convert(fg), convert(back)),
                );
            }
        }
    }
}
fn parse_frame(text: &str) -> Sprite {
    let mut s = Sprite::new(art::WIDTH, art::HEIGHT).unwrap();
    let rows: Vec<_> = text.lines().collect();
    assert_eq!(rows.len(), art::HEIGHT);
    for (y, row) in rows.iter().enumerate() {
        assert_eq!(row.len(), art::WIDTH);
        for (x, c) in row.chars().enumerate() {
            assert!(c == '.' || ink(c).is_some(), "invalid sprite symbol {c}");
            if let Some(color) = ink(c) {
                s.put(x as i32, y as i32, color);
            }
        }
    }
    s
}
fn cached_frame(shape: art::Shape) -> &'static Sprite {
    static FRAMES: std::sync::OnceLock<Vec<Sprite>> = std::sync::OnceLock::new();
    &FRAMES.get_or_init(|| {
        art::Shape::ALL
            .iter()
            .map(|shape| {
                let mut s = parse_frame(art::model(*shape).pixels);
                s.outline();
                s
            })
            .collect()
    })[shape as usize]
}
fn cached_foreground() -> &'static Sprite {
    static FRONT: std::sync::OnceLock<Sprite> = std::sync::OnceLock::new();
    FRONT.get_or_init(|| parse_frame(art::model(art::Shape::Hold).foreground.unwrap()))
}
fn indexed(color: Color) -> Color {
    if let Color::Rgb(r, g, b) = color {
        let level = |n: u8| ((u16::from(n) * 5 + 127) / 255) as u8;
        Color::Indexed(16 + 36 * level(r) + 6 * level(g) + level(b))
    } else {
        color
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub struct Pose {
    pub lift: f32,
}
impl Pose {
    fn target(frame: crate::motion::Frame, _time: f32, intensity: f32) -> Self {
        Self {
            lift: frame.lift * intensity,
        }
    }
    fn approach(&mut self, target: Self, seconds: f32) {
        self.lift += (target.lift - self.lift) * (1.0 - (-seconds.min(1.0) * 9.0).exp());
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrbDepth {
    Behind,
    Side,
    Front,
    Held,
    Free,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrbState {
    Resting,
    Floating,
    Picking,
    Held,
    Thrown,
    Bouncing,
    Returning,
}
pub struct Trixie {
    pub pet: Pet,
    pub orb_depth: OrbDepth,
    pub last_event: Option<MotionEvent>,
    pub intensity: f32,
    pub animate: bool,
    pub palette: Palette,
    pub fault: Option<String>,
    pose: Pose,
    pub diagnostics: bool,
    next_clock: std::time::Instant,
    pub timeline: Timeline,
    pub face: Face,
    pub preview: Option<ClipId>,
    pub paused: bool,
    pub speed: f32,
    pub guides: bool,
    pub facing_override: Option<f32>,
    pub visual_x: f32,
    pub velocity: f32,
    travel_delta: f32,
    orb_x: f32,
    orb_height: f32,
    orb_velocity: f32,
    orb_held: bool,
    pub orb_state: OrbState,
    orb_offset: f32,
    pick_time: f32,
    transition: f32,
    previous_frame: crate::motion::Frame,
    trail: [(f32, f32, f32); 6],
    trail_clock: f32,
}
impl Trixie {
    pub fn new(config: PetConfig, state: Snapshot) -> Self {
        let diagnostics = config.diagnostics;
        let mut pet = Pet::new(config, state);
        pet.set_hour(crate::life::local_hour());
        Self {
            orb_depth: OrbDepth::Side,
            last_event: None,
            visual_x: pet.x,
            velocity: 0.0,
            travel_delta: 0.0,
            orb_x: pet.orb.x,
            orb_height: 19.0,
            orb_velocity: 0.0,
            orb_held: false,
            orb_state: OrbState::Floating,
            orb_offset: 22.0,
            pick_time: 0.0,
            timeline: Timeline::default(),
            face: Face::default(),
            preview: None,
            paused: false,
            speed: 1.0,
            guides: false,
            facing_override: None,
            transition: 1.0,
            previous_frame: crate::motion::clip(ClipId::Idle).frames[0],
            trail: [(pet.x, 19.0, 22.0); 6],
            trail_clock: 0.0,
            pet,
            diagnostics,
            next_clock: std::time::Instant::now() + Duration::from_secs(60),
            intensity: 1.0,
            animate: true,
            palette: Palette::TrueColor,
            fault: None,
            pose: Pose::default(),
        }
    }
    pub fn advance(&mut self, elapsed: Duration) {
        if std::time::Instant::now() >= self.next_clock {
            self.pet.set_hour(crate::life::local_hour());
            self.next_clock = std::time::Instant::now() + Duration::from_secs(60);
        }
        if !self.paused {
            let old_x = self.pet.x;
            if self.preview.is_none() {
                self.pet.advance(elapsed);
            }
            self.travel_delta = self.pet.x - old_x;
            self.velocity = self.travel_delta / elapsed.as_secs_f32().max(0.001);
            self.advance_presentation(elapsed.mul_f32(self.speed.clamp(0.1, 3.0)));
        }
    }
    pub fn event(&mut self, event: PetEvent<'_>) {
        self.pet.event(event);
        if !self.animate {
            self.timeline.request(ClipId::for_pet(&self.pet), true);
            self.pose = Pose::target(self.timeline.frame(), 0.0, 0.0);
        }
    }
    pub fn preview_clip(&mut self, id: ClipId) {
        self.preview = Some(id);
        self.visual_x = self.pet.x;
        self.previous_frame = self.timeline.frame();
        self.timeline = Timeline::default();
        self.timeline.request(id, true);
        self.transition = 0.0;
        self.orb_held = false;
        self.orb_state = OrbState::Floating;
        if id == ClipId::Play {
            self.orb_x = self.visual_x;
            self.orb_height = 3.0;
            self.orb_offset = 22.0;
            self.orb_state = OrbState::Resting;
        }
    }
    pub fn step_frame(&mut self) {
        self.previous_frame = self.timeline.frame();
        self.timeline.step_frame();
        self.transition = 0.0;
        self.update_pose(0.05);
        self.pose = Pose::target(
            self.timeline.frame(),
            0.0,
            if self.animate { self.intensity } else { 0.0 },
        );
    }
    fn advance_presentation(&mut self, elapsed: Duration) {
        let old_id = self.timeline.id;
        let old_index = self.timeline.index;
        let old_frame = self.timeline.frame();
        let wanted = self.preview.unwrap_or_else(|| ClipId::for_pet(&self.pet));
        if self.timeline.request(wanted, false) {
            self.transition = 0.0;
            if self.orb_held {
                self.orb_state = OrbState::Returning;
                self.orb_velocity = 0.0;
            }
            self.orb_held = false;
        }
        if self.animate {
            self.timeline.advance(elapsed);
        }
        if old_id != self.timeline.id || old_index != self.timeline.index {
            self.previous_frame = old_frame;
            self.transition = 0.0;
        }
        self.update_pose(elapsed.as_secs_f32().min(1.0));
    }
    fn update_pose(&mut self, dt: f32) {
        let amount = if self.animate { self.intensity } else { 0.0 };
        for event in self.timeline.drain_events() {
            self.last_event = Some(event);
            match event {
                MotionEvent::Catch => {
                    self.orb_held = true;
                    self.orb_state = OrbState::Picking;
                    self.pick_time = 0.0;
                }
                MotionEvent::Release => {
                    self.orb_held = false;
                    self.orb_velocity = 22.0;
                    self.orb_state = OrbState::Thrown;
                }
                MotionEvent::Land => {}
                _ => {}
            }
        }
        if self.timeline.id == ClipId::Play && self.timeline.index == 7 && self.orb_held {
            self.orb_held = false;
            self.orb_state = OrbState::Returning;
            self.orb_velocity = 0.0;
        }
        self.transition = (self.transition + dt * 12.5).min(1.0);
        let target = Pose::target(
            self.timeline.frame(),
            self.timeline.total_ms as f32 / 1000.0,
            amount,
        );
        self.pose.approach(target, dt);
        self.face.advance(
            &self.pet,
            self.timeline.id,
            if self.animate { dt } else { 0.0 },
        );
        if self.preview.is_none() {
            self.visual_x = self.pet.x;
        }
        let anchor = self.orb_anchor();
        let offset = anchor.x as f32 - art::ORIGIN.x as f32;
        let height = art::ORIGIN.y as f32 - anchor.y as f32 + self.root_lift() as f32;
        let ease = 1.0 - (-dt * 12.0).exp();
        match self.orb_state {
            OrbState::Picking => {
                self.pick_time += dt;
                self.orb_x += (self.visual_x - self.orb_x) * ease;
                self.orb_offset += (offset - self.orb_offset) * ease;
                self.orb_height += (height - self.orb_height) * ease;
                if self.pick_time >= 0.18 {
                    self.orb_state = OrbState::Held;
                }
            }
            OrbState::Held => {
                self.orb_x = self.visual_x;
                self.orb_offset = offset;
                self.orb_height = height;
            }
            OrbState::Thrown | OrbState::Bouncing | OrbState::Returning => {
                self.orb_height += self.orb_velocity * dt - 24.0 * dt * dt;
                self.orb_velocity -= 48.0 * dt;
                if self.orb_height <= 3.0 {
                    self.orb_height = 3.0;
                    self.orb_velocity = -self.orb_velocity * 0.35;
                    self.orb_state = if self.orb_velocity < 3.0 {
                        OrbState::Resting
                    } else {
                        OrbState::Bouncing
                    };
                }
            }
            OrbState::Resting | OrbState::Floating => {
                let sleeping = self.timeline.id == ClipId::Sleep;
                let playful = matches!(self.timeline.id, ClipId::Curious | ClipId::Bored);
                let orbit = self.timeline.id == ClipId::Curious && amount > 0.0;
                let phase = self.timeline.total_ms as f32 / 1000.0 * 1.1;
                let h = if orbit {
                    14.0 + 6.0 * phase.sin()
                } else if sleeping || self.timeline.id == ClipId::Play {
                    3.0
                } else if playful {
                    8.0 + (self.face.clock * 2.0).sin().abs() * 5.0 * amount
                } else {
                    19.0 + (self.face.clock * 1.5).sin() * amount
                };
                let x = self.visual_x;
                let side = if orbit {
                    22.0 * phase.cos()
                } else if self.facing() < 0.0 {
                    -22.0
                } else {
                    22.0
                };
                self.orb_state = if h > 3.1 {
                    OrbState::Floating
                } else {
                    OrbState::Resting
                };
                self.orb_x += (x.clamp(0.0, 1.0) - self.orb_x) * ease;
                self.orb_offset += (side - self.orb_offset) * ease;
                self.orb_height += (h - self.orb_height).clamp(-28.0 * dt, 28.0 * dt);
            }
        }
        self.orb_depth = match self.orb_state {
            OrbState::Held => OrbDepth::Held,
            OrbState::Thrown | OrbState::Bouncing | OrbState::Returning => OrbDepth::Free,
            OrbState::Picking => OrbDepth::Front,
            _ if self.timeline.id == ClipId::Curious && amount > 0.0 => {
                if (self.timeline.total_ms as f32 / 1000.0 * 1.1).sin() >= 0.0 {
                    OrbDepth::Behind
                } else {
                    OrbDepth::Front
                }
            }
            // An interruption must not reveal a rear orb through the body.
            // Retain its current depth until the eased path clears the silhouette.
            _ if self.orb_offset.abs() < 19.0
                && matches!(self.orb_depth, OrbDepth::Behind | OrbDepth::Front) =>
            {
                self.orb_depth
            }
            _ => OrbDepth::Side,
        };
        self.orb_height = self.orb_height.clamp(3.0, 40.0);
        if !self.pet.thinking && !self.pet.speaking {
            let target = (self.orb_offset / 22.0).clamp(-1.0, 1.0);
            self.face.gaze += (target - self.face.gaze) * (dt * 8.0).min(1.0);
        }
        self.trail_clock += dt;
        if self.trail_clock >= 0.05 {
            self.trail_clock %= 0.05;
            self.trail.rotate_right(1);
            self.trail[0] = (self.orb_x, self.orb_height, self.orb_offset);
        }
    }
    pub fn label(&self) -> String {
        if let Some(id) = self.preview {
            return format!(
                "{id:?} · frame {} · {}/{}ms · {:.1}x{}",
                self.timeline.index + 1,
                self.timeline.elapsed_ms,
                self.timeline.frame().ms,
                self.speed,
                if self.paused { " · paused" } else { "" }
            );
        }
        if self.pet.denied {
            "Policy denied · no action taken".into()
        } else if self.pet.thinking {
            "Thinking with you".into()
        } else if self.pet.speaking {
            "Speaking".into()
        } else {
            format!(
                "{} · {}",
                self.pet.traits.mood.label(),
                self.pet.activity.label()
            )
        }
    }
    pub fn diagnostic_lines(&self) -> Vec<String> {
        let p = &self.pet;
        let s = &p.traits;
        let mut scores: Vec<_> = Activity::ALL.iter().zip(p.utilities.iter()).collect();
        scores.sort_by(|a, b| b.1.total_cmp(a.1));
        vec![
            format!(
                "{:?} {} x={:.2}",
                p.activity,
                if p.moving { "travel" } else { "interact" },
                p.x
            ),
            format!("Energy {:.2} Bored {:.2}", s.energy, s.boredom),
            format!("Curious {:.2} Play {:.2}", s.curiosity, s.playfulness),
            format!("Comfort {:.2} Social {:.2}", s.life.comfort, s.sociability),
            format!("Rest {:.2} night={}", s.life.restfulness, p.is_night()),
            format!("tick {} auto={}", s.active_ticks, p.autonomous),
            format!("Utility {:?} {:.1}", scores[0].0, scores[0].1),
            format!("        {:?} {:.1}", scores[1].0, scores[1].1),
        ]
    }
    fn facing(&self) -> f32 {
        self.facing_override.unwrap_or(
            if self.preview.is_some() && matches!(self.timeline.id, ClipId::Walk | ClipId::Run) {
                if self.velocity < 0.0 { -1.0 } else { 1.0 }
            } else {
                self.pet.facing
            },
        )
    }
    fn model(&self) -> art::Model {
        art::model(self.timeline.frame().shape)
    }
    fn root_lift(&self) -> i32 {
        if !self.animate || self.intensity == 0.0 {
            return 2;
        }
        let amount = if self.animate { self.intensity } else { 0.0 };
        let rate = if self.timeline.id == ClipId::Sleep {
            0.45
        } else {
            0.9
        };
        // Stable base levitation; reduced motion keeps the same resting altitude.
        (2.0 + self.pose.lift + (self.face.clock * rate).sin() * amount).round() as i32
    }
    fn orb_anchor(&self) -> Point {
        let a = self.model().orb;
        Point::new(
            if self.facing() < 0.0 {
                art::WIDTH as i32 - 1 - a.x
            } else {
                a.x
            },
            a.y,
        )
    }
    fn orb_position(&self, width: usize, ground: i32) -> Point {
        if self.orb_state == OrbState::Held {
            let a = self.orb_anchor();
            Point::new(
                self.root_x(width) + a.x,
                ground - art::ORIGIN.y - self.root_lift() as f32 as i32 + a.y,
            )
        } else {
            Point::new(
                (2.0 + art::ORIGIN.x as f32
                    + self.orb_x * (width as f32 - art::WIDTH as f32 - 4.0).max(0.0)
                    + self.orb_offset)
                    .round()
                    .clamp(3.0, width.saturating_sub(4).max(3) as f32) as i32,
                ground - self.orb_height.round() as i32,
            )
        }
    }
    fn root_x(&self, width: usize) -> i32 {
        (2.0 + self.visual_x * (width as f32 - art::WIDTH as f32 - 4.0).max(0.0)).round() as i32
    }
    fn body(
        &self,
        frame: crate::motion::Frame,
        _frame_index: usize,
    ) -> Result<Sprite, &'static str> {
        let mut body = cached_frame(frame.shape).clone();
        let model = art::model(frame.shape);
        let eye = Color::Rgb(46, 55, 85);
        let light = Color::Rgb(251, 248, 255);
        let sleep = (self.timeline.id == ClipId::Sleep && self.timeline.index >= 1)
            || (self.timeline.id == ClipId::Wake && self.timeline.index == 0);
        let expression = if self.timeline.id == ClipId::Wake && self.timeline.index == 1 {
            Expression::Sleepy
        } else {
            self.face.expression
        };
        let facing = self.facing();
        for (i, anchor) in model.eyes.iter().enumerate() {
            let cx = anchor.x;
            let cy = anchor.y;
            let closed = sleep
                || (self.animate && self.face.blink())
                || (expression == Expression::Wink && i == 1);
            if closed || expression == Expression::Happy {
                body.line(cx, cy + 2, 4, eye);
            } else {
                let tall = match expression {
                    Expression::Surprised => 6,
                    Expression::Squint | Expression::Sad => 3,
                    Expression::Sleepy => 2,
                    _ => 5,
                };
                for y in 0..tall {
                    body.line(cx, cy + y, 4, eye);
                }
                let gaze = (self.face.gaze * facing).round() as i32;
                body.put(cx + 1 + gaze, cy, light);
                body.put(cx + 1 + gaze, cy + 1, light);
                body.put(cx + 2, cy + tall - 1, light);
                if expression == Expression::Sad {
                    body.put(cx + if i == 0 { 3 } else { 0 }, cy - 2, eye);
                }
                if matches!(expression, Expression::Curious | Expression::Squint) {
                    body.line(cx, cy - 2, 2, eye);
                }
            }
        }
        let mouth_y = model.mouth.y;
        let mx = model.mouth.x;
        if (expression == Expression::Talk && (self.face.clock * 9.0).sin() > -0.2)
            || (expression == Expression::Yawn && (1.0..1.7).contains(&(self.face.clock % 3.2)))
            || expression == Expression::Surprised
        {
            let height = if expression == Expression::Yawn { 4 } else { 2 };
            for dy in 0..height {
                body.line(mx - 1, mouth_y + dy, 3, eye);
            }
            body.line(mx - 1, mouth_y + height - 1, 3, Color::Rgb(218, 130, 168));
        } else if expression == Expression::Sad {
            body.line(mx - 1, mouth_y, 3, eye);
            body.put(mx - 2, mouth_y + 1, eye);
            body.put(mx + 2, mouth_y + 1, eye);
        } else {
            body.put(mx - 2, mouth_y, eye);
            body.line(mx - 1, mouth_y + 1, 3, eye);
            body.put(mx + 2, mouth_y, eye);
        }
        if self.pet.denied {
            for y in 0..7 {
                body.line(12 + y / 2, 20 + y, 8 - y, Color::Rgb(250, 105, 130));
            }
            body.line(15, 21, 2, eye);
            body.put(15, 23, eye);
        }
        Ok(body)
    }
    pub fn compose(&self, width: usize, height: usize) -> Result<Sprite, &'static str> {
        if !self.intensity.is_finite() || !(0.0..=1.0).contains(&self.intensity) {
            return Err("Invalid animation intensity");
        }
        let mut world =
            Sprite::new(width, height).ok_or("Character viewport exceeds safe bounds")?;
        if width < 4 || height < 4 {
            return Ok(world);
        }
        let ground = height as i32 - 3;
        let body = self.body(self.timeline.frame(), self.timeline.index)?;
        let facing = self.facing();
        let t = if self.animate { self.face.clock } else { 0.0 };
        let light = Color::Rgb(251, 248, 255);
        let sleep = self.timeline.id == ClipId::Sleep && self.timeline.index >= 2;
        // Integer-only placement: no resampling, shearing, colour blending or scale morph.
        let x0 = self.root_x(width);
        let lift = self.root_lift();
        let root_y = ground - self.model().origin.y - lift;
        let shadow_half = (7 - lift / 3).max(4);
        world.line(
            x0 + art::ORIGIN.x - shadow_half,
            ground - 1,
            shadow_half * 2,
            Color::Rgb(24, 29, 43),
        );
        let draw_character = |world: &mut Sprite| {
            for y in 0..art::HEIGHT {
                for x in 0..art::WIDTH {
                    let source_x = if facing < 0.0 { art::WIDTH - 1 - x } else { x };
                    if let Some(color) = body.get(source_x, y) {
                        world.put(x0 + x as i32, root_y + y as i32, color);
                    }
                }
            }
        };
        if self.orb_depth != OrbDepth::Behind {
            draw_character(&mut world);
        }
        let play = !self.pet.moving
            && matches!(
                self.pet.activity,
                Activity::Play | Activity::Chase | Activity::Balance
            );
        let orb = self.orb_position(width, ground);
        let orb_x = orb.x as f32;
        let orb_y = orb.y as f32;
        let pulse = if self.animate {
            (t * 3.0).sin() * 12.0 * self.intensity
        } else {
            0.0
        };
        let dim = sleep;
        let flash = matches!(self.timeline.id, ClipId::Happy | ClipId::Celebrate)
            && self.timeline.elapsed_ms < 180
            && self.animate
            && self.intensity > 0.0;
        let glow = if dim {
            Color::Rgb(65, 162, 158)
        } else {
            Color::Rgb(
                if flash { 145 } else { 45 },
                (230.0 + pulse).clamp(0.0, 255.0) as u8,
                218,
            )
        };
        // One orb draw: behind/side/front/held all use the same authoritative position.
        for dy in -3i32..=3 {
            for dx in -3i32..=3 {
                let radius = dx * dx + dy * dy;
                if radius > 10 {
                    continue;
                }
                let px = orb_x as i32 + dx;
                let py = orb_y as i32 + dy;
                let color = if radius >= 8 {
                    Color::Rgb(24, 103, 115)
                } else if dx == 0 && dy == 0 {
                    if dim { glow } else { light }
                } else if dx <= 0 && dy <= 0 {
                    Color::Rgb(145, 255, 249)
                } else {
                    glow
                };
                world.put(px, py, color);
            }
        }
        if self.orb_depth == OrbDepth::Behind {
            draw_character(&mut world);
        }
        if self.orb_depth == OrbDepth::Held && self.model().foreground.is_some() {
            let foreground = cached_foreground();
            for y in 0..art::HEIGHT {
                for x in 0..art::WIDTH {
                    let sx = if facing < 0.0 { art::WIDTH - 1 - x } else { x };
                    if let Some(color) = foreground.get(sx, y) {
                        world.put(x0 + x as i32, root_y + y as i32, color);
                    }
                }
            }
        }
        // Six retained trail samples; no per-frame particle allocation.
        if self.animate
            && self.intensity > 0.0
            && (play || self.preview == Some(ClipId::Play) || self.pet.thinking)
        {
            for (i, (x, h, offset)) in self.trail.iter().enumerate().skip(1) {
                let c = Color::Rgb(45, 145 - i as u8 * 12, 140 - i as u8 * 10);
                let px = (2.0
                    + art::ORIGIN.x as f32
                    + x * (width as f32 - art::WIDTH as f32 - 4.0).max(0.0)
                    + offset)
                    .round() as i32;
                let py = (ground as f32 - h).round() as i32;
                if world.get(px as usize, py as usize).is_none() {
                    world.put(px, py, c);
                }
            }
        }
        if self.animate && self.intensity > 0.0 && (play || self.preview == Some(ClipId::Play)) {
            for n in 0..3 {
                let phase = t * 0.7 + n as f32 * 2.1;
                let x = orb.x + n * 4 - 4;
                let y = ground - 5 - (phase.sin() * 2.0) as i32;
                if world.get(x as usize, y as usize).is_none() {
                    world.put(x, y, Color::Rgb(49, 76, 85));
                }
            }
        }

        if self.guides {
            let guide = Color::Rgb(90, 100, 120);
            for x in 0..art::WIDTH as i32 {
                world.put(x0 + x, root_y, guide);
                world.put(x0 + x, root_y + art::HEIGHT as i32 - 1, guide);
            }
            for y in 0..art::HEIGHT as i32 {
                world.put(x0, root_y + y, guide);
                world.put(x0 + art::WIDTH as i32 - 1, root_y + y, guide);
            }
            world.line(0, ground, width as i32, guide);
            let model = art::model(self.timeline.frame().shape);
            let hands = model.arms;
            for (p, color) in [
                (art::ORIGIN, Color::Rgb(255, 190, 80)),
                (model.eyes[0], Color::Rgb(80, 200, 255)),
                (model.eyes[1], Color::Rgb(80, 200, 255)),
                (model.mouth, Color::Rgb(80, 200, 255)),
                (hands[0], Color::Rgb(111, 226, 208)),
                (hands[1], Color::Rgb(111, 226, 208)),
            ] {
                let x = x0
                    + if facing < 0.0 {
                        art::WIDTH as i32 - 1 - p.x
                    } else {
                        p.x
                    };
                let y = root_y + p.y;
                world.line(x - 1, y, 3, color);
                world.put(x, y - 1, color);
                world.put(x, y + 1, color);
            }
            world.line(orb.x - 2, orb.y, 5, light);
        }
        if sleep {
            world.art(
                &["eeeee", "   e ", "  e  ", " e   ", "eeeee"],
                x0 + 33,
                ground - 29,
            );
        }
        Ok(world)
    }
}
impl Widget for Trixie {
    fn draw(&mut self, c: &mut Canvas<'_>, _: bool, theme: &Theme) {
        if let Some(error) = &self.fault {
            c.put_str(1, 1, &format!("Trixie paused: {error}"), theme.focused);
            return;
        }
        let size = c.size();
        if size.width < 4 || size.height < 3 {
            return;
        }
        let scale = if size.width < 48 || size.height < 21 {
            2
        } else {
            1
        };
        let width = (usize::from(size.width) * scale).min(64);
        let height = (usize::from(size.height.saturating_sub(2)) * 2 * scale).clamp(4, 44);
        match self.compose(width, height) {
            Ok(sprite) => {
                let x0 = size.width.saturating_sub((width / scale) as u16) / 2;
                let y0 = size
                    .height
                    .saturating_sub((height / (2 * scale)) as u16 + 2)
                    / 2;
                sprite.draw(c, x0, y0, scale, self.palette, theme.surface.background);
                c.put_str(1, size.height - 1, &self.label(), theme.focused);
                if self.guides {
                    c.put_str(
                        0,
                        0,
                        &format!(
                            "{:?} #{} clip={}ms v={:.3}/s home={:+.2} scale=1:{scale}",
                            self.timeline.id,
                            self.timeline.index + 1,
                            crate::motion::clip(self.timeline.id)
                                .frames
                                .iter()
                                .map(|f| f.ms)
                                .sum::<u32>(),
                            self.velocity,
                            self.visual_x - 0.5
                        ),
                        theme.bar,
                    );
                    c.put_str(
                        0,
                        1,
                        &format!(
                            "{:?}->{:?} {} · {:?}/{:?} · {:?}",
                            self.previous_frame.shape,
                            self.timeline.frame().shape,
                            if self.transition < 1.0 {
                                "settling"
                            } else {
                                "held"
                            },
                            self.orb_state,
                            self.orb_depth,
                            self.last_event
                        ),
                        theme.bar,
                    );
                }
                if self.diagnostics {
                    for (row, line) in self
                        .diagnostic_lines()
                        .iter()
                        .enumerate()
                        .take(usize::from(size.height.saturating_sub(1)))
                    {
                        c.put_str(0, row as u16, line, theme.bar);
                    }
                }
            }
            Err(error) => {
                self.fault = Some(error.into());
                c.put_str(
                    1,
                    1,
                    "Trixie paused · chat remains available",
                    theme.focused,
                );
            }
        }
    }
}
pub fn default_trixie() -> Trixie {
    let config = PetConfig::default();
    let state = Snapshot::new(config.seed);
    Trixie::new(config, state)
}
pub fn validate_fps(fps: u32) -> io::Result<Duration> {
    if !(1..=60).contains(&fps) {
        return Err(io::Error::other("FPS must be 1–60"));
    }
    Ok(Duration::from_secs_f64(1.0 / f64::from(fps)))
}
/// Compatibility names for pre-0.4.4 callers.
pub type Terri = Trixie;
pub use default_trixie as default_terri;

#[cfg(test)]
mod tests {
    use super::*;
    use terrabox::{Rect, Size, Surface};
    #[test]
    fn authored_frames_have_stable_anchors_and_no_lower_appendages() {
        for shape in art::Shape::ALL {
            let model = art::model(shape);
            let body = cached_frame(shape);
            assert_eq!(model.origin, art::ORIGIN);
            for a in model
                .eyes
                .into_iter()
                .chain([model.mouth])
                .chain(model.arms)
            {
                assert!(
                    body.get(a.x as usize, a.y as usize).is_some(),
                    "anchor outside {shape:?}: {a:?}"
                );
            }
            for x in [10, 11, 25, 26] {
                assert_eq!(body.get(x, 19), ink('p'), "cheek alignment {shape:?}");
            }
            // The lower body tapers into one continuous tip, never split feet.
            for y in 24..art::HEIGHT {
                let xs: Vec<_> = (0..art::WIDTH)
                    .filter(|x| body.get(*x, y).is_some())
                    .collect();
                if let (Some(first), Some(last)) = (xs.first(), xs.last()) {
                    assert_eq!(
                        xs.len(),
                        last - first + 1,
                        "split lower silhouette {shape:?}"
                    );
                }
                if y >= 28 {
                    assert!(xs.is_empty(), "leg/foot pixels below floating body");
                }
            }
        }
    }
    #[test]
    fn orb_layers_occlude_in_both_directions_and_holding_arms_cover_rim() {
        let mut t = default_trixie();
        t.animate = false;
        t.orb_x = t.visual_x;
        t.orb_offset = 0.0;
        t.orb_height = 15.0;
        t.orb_state = OrbState::Floating;
        let p = t.orb_position(64, 41);
        t.orb_depth = OrbDepth::Behind;
        let behind = t.compose(64, 44).unwrap();
        let body = t.body(t.timeline.frame(), 0).unwrap();
        assert_eq!(behind.get(p.x as usize, p.y as usize), body.get(18, 18));
        t.orb_depth = OrbDepth::Front;
        let front = t.compose(64, 44).unwrap();
        assert_eq!(
            front.get(p.x as usize, p.y as usize),
            Some(Color::Rgb(251, 248, 255))
        );
        assert_ne!(
            behind.get(p.x as usize, p.y as usize),
            front.get(p.x as usize, p.y as usize)
        );
        t.preview_clip(ClipId::Play);
        t.timeline.index = 2;
        t.orb_state = OrbState::Held;
        t.orb_depth = OrbDepth::Held;
        for facing in [-1.0, 1.0] {
            t.facing_override = Some(facing);
            let held = t.compose(64, 44).unwrap();
            let orb = t.orb_position(64, 41);
            assert_eq!(
                held.get(orb.x as usize, orb.y as usize),
                Some(Color::Rgb(251, 248, 255))
            );
            // The lower rim lies behind an explicitly authored foreground hand.
            let px = if facing < 0.0 { 35 - 27 } else { 27 };
            assert_eq!(
                held.get(
                    (t.root_x(64) + px) as usize,
                    (41 - art::ORIGIN.y - t.root_lift() + 22) as usize
                ),
                cached_foreground().get(27, 22)
            );
            assert!(cached_foreground().get(27, 22).is_some());
        }
    }
    #[test]
    fn orbit_depth_crossings_are_local_and_reduced_motion_is_still() {
        let mut t = default_trixie();
        t.preview_clip(ClipId::Curious);
        let mut depths = Vec::new();
        let mut last = t.orb_position(64, 41);
        for _ in 0..650 {
            t.advance(Duration::from_millis(10));
            let p = t.orb_position(64, 41);
            assert!((p.x - last.x).abs() <= 1 && (p.y - last.y).abs() <= 1);
            depths.push(t.orb_depth);
            last = p;
        }
        assert!(depths.contains(&OrbDepth::Behind) && depths.contains(&OrbDepth::Front));
        t.intensity = 0.0;
        t.preview_clip(ClipId::Idle);
        for _ in 0..100 {
            t.advance(Duration::from_millis(20));
            assert_eq!(t.root_lift(), 2);
        }
    }
    #[test]
    fn interrupted_orbit_keeps_rear_depth_until_it_reaches_the_side() {
        let mut t = default_trixie();
        t.preview_clip(ClipId::Curious);
        for _ in 0..140 {
            t.advance(Duration::from_millis(10));
        }
        assert_eq!(t.orb_depth, OrbDepth::Behind);
        assert!(t.orb_offset.abs() < 10.0);
        t.preview_clip(ClipId::Think);
        let mut cleared = false;
        for _ in 0..100 {
            t.advance(Duration::from_millis(10));
            if t.orb_offset.abs() < 19.0 {
                assert_eq!(t.orb_depth, OrbDepth::Behind);
            } else {
                assert_eq!(t.orb_depth, OrbDepth::Side);
                cleared = true;
            }
        }
        assert!(cleared);
    }
    #[test]
    fn body_palette_and_original_open_eyes_are_preserved() {
        let mut t = default_trixie();
        t.animate = false;
        for id in ClipId::ALL {
            t.preview_clip(id);
            for (index, frame) in crate::motion::clip(id).frames.iter().enumerate() {
                t.timeline.index = index;
                let body = t.body(*frame, index).unwrap();
                for pixel in body.pixels.iter().flatten() {
                    if let Color::Rgb(r, g, b) = *pixel {
                        assert!(
                            !(u16::from(g) > u16::from(r) + 20 && u16::from(b) > u16::from(r) + 20),
                            "turquoise body patch in {id:?}/{index}"
                        );
                    }
                }
            }
        }
        t.preview_clip(ClipId::Idle);
        t.face.expression = Expression::Neutral;
        t.face.gaze = 0.0;
        let frame = t.timeline.frame();
        let body = t.body(frame, 0).unwrap();
        for anchor in art::model(frame.shape).eyes {
            for y in 0..5 {
                for x in 0..4 {
                    let highlight = (x == 1 && y < 2) || (x == 2 && y == 4);
                    assert_eq!(
                        body.get((anchor.x + x) as usize, (anchor.y + y) as usize),
                        Some(if highlight {
                            Color::Rgb(251, 248, 255)
                        } else {
                            Color::Rgb(46, 55, 85)
                        })
                    );
                }
            }
        }
        t.face.expression = Expression::Happy;
        let body = t.body(frame, 0).unwrap();
        for anchor in art::model(frame.shape).eyes {
            for x in 0..4 {
                assert_eq!(
                    body.get((anchor.x + x) as usize, (anchor.y + 2) as usize),
                    Some(Color::Rgb(46, 55, 85))
                );
            }
        }
    }
    #[test]
    fn eye_palette_and_composition_do_not_leak_turquoise() {
        let mut t = default_trixie();
        for id in ClipId::ALL {
            t.preview_clip(id);
            for index in 0..crate::motion::clip(id).frames.len() {
                t.timeline.index = index;
                let frame = t.timeline.frame();
                let model = art::model(frame.shape);
                for expression in Expression::ALL {
                    t.face.expression = expression;
                    let body = t.body(frame, index).unwrap();
                    for eye in model.eyes {
                        for y in eye.y..eye.y + 6 {
                            for x in eye.x..eye.x + 4 {
                                if let Some(Color::Rgb(r, g, b)) = body.get(x as usize, y as usize)
                                {
                                    assert!(
                                        !(u16::from(g) > u16::from(r) + 20
                                            && u16::from(b) > u16::from(r) + 20),
                                        "teal eye {id:?}/{index}/{expression:?}"
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
        t.preview_clip(ClipId::Idle);
        t.face.expression = Expression::Neutral;
        // Force a free orb behind the face: it must not overwrite any body pixel.
        t.orb_x = t.visual_x;
        t.orb_offset = 0.0;
        t.orb_height = 16.0;
        t.orb_depth = OrbDepth::Behind;
        let world = t.compose(64, 44).unwrap();
        let body = t.body(t.timeline.frame(), 0).unwrap();
        for y in 0..art::HEIGHT {
            for x in 0..art::WIDTH {
                if let Some(color) = body.get(x, y) {
                    assert_eq!(
                        world.get(t.root_x(64) as usize + x, (10 - t.root_lift()) as usize + y),
                        Some(color)
                    );
                }
            }
        }
        assert!(
            world.get(1, 41).is_none(),
            "no habitat floor or decorations"
        );
    }
    #[test]
    fn completed_character_is_connected_and_outlined() {
        let t = default_trixie();
        for id in ClipId::ALL {
            for (index, frame) in crate::motion::clip(id).frames.iter().enumerate() {
                let body = t.body(*frame, index).unwrap();
                let mut seen = vec![false; body.pixels.len()];
                let start = body.pixels.iter().position(Option::is_some).unwrap();
                let mut queue = vec![start];
                seen[start] = true;
                while let Some(i) = queue.pop() {
                    let (x, y) = (i % body.width, i / body.width);
                    for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                        let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                        if nx < 0 || ny < 0 || nx >= body.width as i32 || ny >= body.height as i32 {
                            continue;
                        }
                        let n = ny as usize * body.width + nx as usize;
                        if body.pixels[n].is_some() && !seen[n] {
                            seen[n] = true;
                            queue.push(n);
                        }
                    }
                }
                assert!(
                    body.pixels
                        .iter()
                        .enumerate()
                        .all(|(i, p)| p.is_none() || seen[i]),
                    "detached {id:?}/{index}"
                );
                for y in 0..body.height {
                    for x in 0..body.width {
                        if body.get(x, y).is_none() {
                            continue;
                        }
                        assert!(
                            x > 0 && x + 1 < body.width && y > 0 && y < art::ORIGIN.y as usize,
                            "clipped or below ground {id:?}/{index} at {x},{y}"
                        );
                        if [(-1, 0), (1, 0), (0, -1), (0, 1)].iter().any(|(dx, dy)| {
                            body.get((x as i32 + dx) as usize, (y as i32 + dy) as usize)
                                .is_none()
                        }) {
                            assert_eq!(body.get(x, y), ink('o'), "broken outline {id:?}/{index}");
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn faces_layers_and_rendering_are_repeatable() {
        let mut t = default_trixie();
        for id in ClipId::ALL {
            t.preview_clip(id);
            for expression in Expression::ALL {
                t.face.override_expression = Some(expression);
                t.advance(Duration::from_millis(50));
                let snapshot = t.pet.snapshot();
                let a = t.compose(80, 44).unwrap();
                let b = t.compose(80, 44).unwrap();
                assert_eq!(a.pixels, b.pixels);
                assert!(a.pixels.iter().any(Option::is_some));
                assert_eq!(snapshot, t.pet.snapshot());
            }
        }
        let mut s = Sprite::new(4, 4).unwrap();
        s.art(&[" o"], 0, 0);
        assert!(s.get(0, 0).is_none());
        assert!(s.get(1, 0).is_some());
    }
    #[test]
    fn orb_attachment_and_release_are_continuous_in_both_directions() {
        for facing in [-1.0, 1.0] {
            let mut t = default_trixie();
            t.preview_clip(ClipId::Play);
            t.facing_override = Some(facing);
            let mut previous = t.orb_position(110, 45);
            let mut held = false;
            let mut released = false;
            for _ in 0..300 {
                let old_state = t.orb_state;
                t.advance(Duration::from_millis(10));
                let p = t.orb_position(110, 45);
                if t.orb_state == OrbState::Held {
                    held = true;
                    let anchor = t.model().orb;
                    let x = if facing < 0.0 {
                        art::WIDTH as i32 - 1 - anchor.x
                    } else {
                        anchor.x
                    };
                    assert_eq!(p.x, t.root_x(110) + x);
                    assert_eq!(p.y, 45 - art::ORIGIN.y - t.root_lift() + anchor.y);
                }
                if old_state == OrbState::Held && t.orb_state == OrbState::Thrown {
                    released = true;
                    assert!((p.x - previous.x).abs() <= 1);
                    assert!((p.y - previous.y).abs() <= 1);
                }
                if old_state == OrbState::Picking && t.orb_state == OrbState::Held {
                    assert!((p.x - previous.x).abs() <= 2);
                    assert!((p.y - previous.y).abs() <= 2);
                }
                previous = p;
            }
            assert!(held && released);
        }
    }
    #[test]
    fn reduced_motion_pause_and_orb_contacts() {
        let mut t = default_trixie();
        t.intensity = 0.0;
        t.preview_clip(ClipId::Jump);
        for _ in 0..20 {
            t.advance(Duration::from_millis(50));
            assert_eq!(t.pose.lift, 0.0);
        }
        t.paused = true;
        let index = t.timeline.index;
        t.advance(Duration::from_secs(1));
        assert_eq!(t.timeline.index, index);
        t.paused = false;
        t.intensity = 1.0;
        t.preview_clip(ClipId::Play);
        let mut caught = false;
        let mut thrown = false;
        for _ in 0..80 {
            t.advance(Duration::from_millis(50));
            assert!((3.0..=40.0).contains(&t.orb_height));
            assert!((0.0..=1.0).contains(&t.orb_x));
            caught |= t.orb_held;
            thrown |= t.orb_velocity > 0.0;
            if t.orb_held && t.timeline.elapsed_ms > 200 {
                assert!((t.orb_x - t.visual_x).abs() < 0.03);
            }
        }
        assert!(caught && thrown);
    }
    #[test]
    fn authored_floating_silhouettes_differ() {
        let mut t = default_trixie();
        let mut images = Vec::new();
        for id in [ClipId::Idle, ClipId::Sit, ClipId::Sleep] {
            t.preview_clip(id);
            for _ in 0..30 {
                t.advance(Duration::from_millis(50));
            }
            images.push(t.body(t.timeline.frame(), t.timeline.index).unwrap().pixels);
        }
        assert_ne!(images[0], images[1]);
        assert_ne!(images[1], images[2]);
        let c = crate::motion::clip(ClipId::Walk);
        assert_ne!(c.frames[0].shape, c.frames[1].shape);
    }
    #[test]
    fn frame_clips_loop_or_hold_last_frame() {
        let frames: [&[&str]; 4] = [&["o"], &["b"], &["c"], &["d"]];
        let clip = AnimationClip::new(&frames, Duration::from_millis(100), true).unwrap();
        assert_eq!(clip.frame(Duration::from_millis(100)), frames[1]);
        assert_eq!(clip.frame(Duration::from_millis(400)), frames[0]);
        let clip = AnimationClip::new(&frames, Duration::from_millis(100), false).unwrap();
        assert_eq!(clip.frame(Duration::from_secs(10)), frames[3]);
        assert!(AnimationClip::new(&frames, Duration::ZERO, true).is_none());
    }
    #[test]
    fn artwork_bounds_and_no_simulation_in_render() {
        assert_eq!(art::WIDTH, 36);
        let mut t = default_trixie();
        for a in Activity::ALL {
            t.pet.demo_activity(a);
            let snapshot = t.pet.snapshot();
            for (w, h) in [(0, 0), (1, 1), (18, 12), (42, 24), (120, 60)] {
                let mut surface = Surface::new(Size {
                    width: w,
                    height: h,
                });
                t.draw(
                    &mut surface.canvas(Rect::new(0, 0, w, h)),
                    false,
                    &Theme::default(),
                );
            }
            assert_eq!(snapshot, t.pet.snapshot());
        }
    }
    #[test]
    fn clipped_canvas_never_touches_chat() {
        let mut t = default_trixie();
        let mut s = Surface::new(Size {
            width: 80,
            height: 30,
        });
        let marker = Style::new(Color::Indexed(1), Color::Indexed(2));
        s.clear(marker);
        t.draw(
            &mut s.canvas(Rect::new(0, 0, 20, 20)),
            false,
            &Theme::default(),
        );
        assert_eq!(s.cell(21, 5).unwrap().style, marker);
    }
    #[test]
    fn smooth_transition_and_fault_is_local() {
        let mut t = default_trixie();
        t.pet.x = t.pet.traits.life.favourite_rest;
        t.pet.demo_activity(Activity::Sleep);
        t.advance(Duration::from_millis(50));
        assert_eq!(t.timeline.id, ClipId::Sleep);
        assert!(t.pose.lift.is_finite());
        t.intensity = f32::NAN;
        assert!(t.compose(40, 40).is_err());
    }
}
