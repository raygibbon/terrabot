//! Handcrafted pixel layers composed into a bounded logical grid, then TerraBox cells.
use crate::{
    config::PetConfig,
    cues::Gesture,
    motion::{ClipId, Expression, Face, MotionEvent, Shape, Timeline},
    pet::{Activity, Pet, PetEvent, Snapshot},
};
use std::{io, time::Duration};
use terrabox::{Canvas, Color, Style};
use terratui::{Theme, Widget};

// Original moon-moth creature: asymmetric mint feelers, lilac coat, peach cheeks.
// Palette letters are pixels, not terminal glyphs. Spaces are transparent.
const BODY: &[&str] = &[
    "       oo              oo",
    "      occo            occo",
    "      occco          occco",
    "      obcccoooooooooccccbo",
    "     oabccccddddddcccccbbo",
    "    oabcccddddddddddcccbbao",
    "   oabcccddddddddddddccbbao",
    "   obcccfffffffffffffccbbbo",
    "  oabccfffffffffffffffcbbbao",
    "  obccffffffffffffffffcbbbo",
    "  obccffffffffffffffffcbbbo",
    "  obccffffffffffffffffcbbbo",
    "  obccffffffffffffffffcbbbo",
    "  obccffffffffffffffffcbbbo",
    "  obccffffffffffffffffcbbbo",
    "  obccffffffffffffffffcbbbo",
    "  obccfffppffffffffppfcbbbo",
    "  oabccffffffffffffffcbbbao",
    "   obccfffffffffffffccbbbo",
    "   oabccccffffffffccccbao",
    "    oabcccccccccccccbbao",
    "     oabbbccceeecccbbbao",
    "      oabbbceeeecbbbbao",
    "       oabbbceecbbbbao",
    "        oabbbbbbbbao",
    "         ooooooooo",
];
const HANDS: [&[&str]; 3] = [
    &[" oo ", "occo", "obbo", " oo "],
    &[" oo ", "occo", "occo", "obbo", " oo "],
    &["  oo", " occo", "obbo", " oo "],
];
const FEET: [&[&str]; 4] = [
    &[" oooo ", "occcbo", "oooooo"],
    &[" ooo  ", "occbo ", " oooo "],
    &["  ooo ", " occbo", " oooo "],
    &[" oooo ", "occcbo", "oooooo"],
];
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
fn indexed(color: Color) -> Color {
    if let Color::Rgb(r, g, b) = color {
        let level = |n: u8| ((u16::from(n) * 5 + 127) / 255) as u8;
        Color::Indexed(16 + 36 * level(r) + 6 * level(g) + level(b))
    } else {
        color
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Pose {
    pub squash: f32,
    pub lean: f32,
    pub lift: f32,
    pub arm: f32,
}
impl Default for Pose {
    fn default() -> Self {
        Self {
            squash: 1.0,
            lean: 0.0,
            lift: 0.0,
            arm: 0.0,
        }
    }
}
impl Pose {
    fn target(frame: crate::motion::Frame, time: f32, intensity: f32) -> Self {
        let amount = intensity.clamp(0.0, 1.0);
        Self {
            squash: 1.0 + (frame.squash - 1.0) * amount + (time * 2.0).sin() * 0.008 * amount,
            lean: frame.lean * amount,
            lift: frame.lift * amount,
            arm: 0.0,
        }
    }
    fn approach(&mut self, target: Self, seconds: f32) {
        let a = 1.0 - (-seconds.min(1.0) * 9.0).exp();
        self.squash += (target.squash - self.squash) * a;
        self.lean += (target.lean - self.lean) * a;
        self.lift += (target.lift - self.lift) * a;
        self.arm += (target.arm - self.arm) * a;
    }
}
pub struct Terri {
    pub pet: Pet,
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
    orb_x: f32,
    orb_height: f32,
    orb_velocity: f32,
    orb_held: bool,
    transition: f32,
    previous_frame: crate::motion::Frame,
    previous_index: usize,
    trail: [(f32, f32); 6],
    trail_clock: f32,
    pub land_flash: f32,
}
impl Terri {
    pub fn new(config: PetConfig, state: Snapshot) -> Self {
        let diagnostics = config.diagnostics;
        let mut pet = Pet::new(config, state);
        pet.set_hour(crate::life::local_hour());
        Self {
            visual_x: pet.x,
            orb_x: pet.orb.x,
            orb_height: pet.orb.height,
            orb_velocity: 0.0,
            orb_held: false,
            timeline: Timeline::default(),
            face: Face::default(),
            preview: None,
            paused: false,
            speed: 1.0,
            guides: false,
            facing_override: None,
            transition: 1.0,
            previous_frame: crate::motion::clip(ClipId::Idle).frames[0],
            previous_index: 0,
            trail: [(pet.orb.x, pet.orb.height); 6],
            trail_clock: 0.0,
            land_flash: 0.0,
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
            if self.preview.is_none() {
                self.pet.advance(elapsed);
            }
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
        self.previous_frame = self.timeline.frame();
        self.previous_index = self.timeline.index;
        self.timeline = Timeline::default();
        self.timeline.request(id, true);
        self.transition = 0.0;
        self.orb_held = false;
    }
    pub fn step_frame(&mut self) {
        self.previous_frame = self.timeline.frame();
        self.previous_index = self.timeline.index;
        self.timeline.step_frame();
        self.transition = 0.0;
        self.update_pose(0.05);
    }
    fn advance_presentation(&mut self, elapsed: Duration) {
        let old_id = self.timeline.id;
        let old_index = self.timeline.index;
        let old_frame = self.timeline.frame();
        let wanted = self.preview.unwrap_or_else(|| ClipId::for_pet(&self.pet));
        if self.timeline.request(wanted, false) {
            self.transition = 0.0;
            self.orb_held = false;
        }
        if self.animate {
            self.timeline.advance(elapsed);
        }
        if old_id != self.timeline.id || old_index != self.timeline.index {
            self.previous_frame = old_frame;
            self.previous_index = old_index;
            self.transition = 0.0;
        }
        self.update_pose(elapsed.as_secs_f32().min(1.0));
    }
    fn update_pose(&mut self, dt: f32) {
        let amount = if self.animate { self.intensity } else { 0.0 };
        for event in self.timeline.drain_events() {
            match event {
                MotionEvent::Catch => self.orb_held = true,
                MotionEvent::Release => {
                    self.orb_held = false;
                    self.orb_velocity = 29.0;
                }
                MotionEvent::Land | MotionEvent::Step => self.land_flash = 1.0,
                _ => {}
            }
        }
        if self.timeline.id == ClipId::Play && self.timeline.index == 7 {
            self.orb_held = false;
        }
        self.transition = (self.transition + dt * 12.5).min(1.0);
        let mut target = Pose::target(
            self.timeline.frame(),
            self.timeline.total_ms as f32 / 1000.0,
            amount,
        );
        if !self.pet.thinking && !self.pet.speaking && !self.pet.gentle {
            match self.pet.gesture {
                Gesture::HeadTilt => target.lean += 2.0 * amount,
                Gesture::Nod => target.squash += (self.face.clock * 5.0).sin() * 0.025 * amount,
                _ => {}
            }
        }
        self.pose.approach(target, dt);
        self.face.advance(
            &self.pet,
            self.timeline.id,
            if self.animate { dt } else { 0.0 },
        );
        self.visual_x += (self.pet.x - self.visual_x) * (dt * 20.0).min(1.0);
        self.land_flash = (self.land_flash - dt * 4.0).max(0.0);
        let frame = self.timeline.frame();
        let hand_y = (frame.arms[0].1 + frame.arms[1].1) as f32 / 2.0;
        let (x, height) = if self.orb_held {
            (
                self.visual_x,
                (30.0 - hand_y) * self.pose.squash + self.pose.lift + 3.0,
            )
        } else if self.timeline.id == ClipId::Play {
            (self.visual_x, 3.0)
        } else if self.timeline.id == ClipId::Think {
            (
                self.visual_x + 0.16,
                32.0 + (self.face.clock * 2.0).sin() * 1.5 * amount,
            )
        } else {
            (self.pet.orb.x, self.pet.orb.height)
        };
        let a = 1.0 - (-dt * 12.0).exp();
        self.orb_x += (x.clamp(0.0, 1.0) - self.orb_x) * a;
        if self.timeline.id == ClipId::Play
            && (self.timeline.index == 4 || self.timeline.index == 5)
            && !self.orb_held
        {
            self.orb_velocity -= 40.0 * dt;
            self.orb_height = (self.orb_height + self.orb_velocity * dt).max(3.0);
        } else {
            self.orb_height += (height - self.orb_height) * a;
        }
        self.orb_height = self.orb_height.clamp(3.0, 40.0);
        if !self.pet.thinking && !self.pet.speaking {
            let target = ((self.orb_x - self.visual_x) * 4.0).clamp(-1.0, 1.0);
            self.face.gaze += (target - self.face.gaze) * (dt * 8.0).min(1.0);
        }
        self.trail_clock += dt;
        if self.trail_clock >= 0.05 {
            self.trail_clock %= 0.05;
            self.trail.rotate_right(1);
            self.trail[0] = (self.orb_x, self.orb_height);
        }
    }
    pub fn label(&self) -> String {
        if let Some(id) = self.preview {
            return format!(
                "{id:?} · frame {} · {}ms · {:.1}x{}",
                self.timeline.index + 1,
                self.timeline.elapsed_ms,
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
    fn body(&self, frame: crate::motion::Frame, foot_index: usize) -> Result<Sprite, &'static str> {
        let glow = Color::Rgb(105, 223, 206);
        let mut body = Sprite::new(32, 32).ok_or("Invalid sprite bounds")?;
        let (art, art_y, face_y) = match frame.shape {
            Shape::Stand => (BODY, 1, 12),
            Shape::Seat => (crate::art::SEATED, 7, 16),
            Shape::Curl => (crate::art::CURLED, 12, 18),
            Shape::Stretch => (crate::art::STRETCHED, 0, 10),
        };
        body.art(art, 1, art_y);
        for (i, (x, y)) in frame.feet.iter().enumerate() {
            body.art(FEET[(foot_index + i * 2) % FEET.len()], *x, *y);
        }
        let mut arms = frame.arms;
        if self.pet.gesture == Gesture::Wave
            && !self.pet.thinking
            && !self.pet.speaking
            && !self.pet.gentle
        {
            arms[1] = (
                28,
                8 + if self.animate {
                    (self.face.clock * 5.0).sin().round() as i32
                } else {
                    0
                },
            );
        }
        for (i, (x, y)) in arms.iter().enumerate() {
            body.art(HANDS[if *y < 15 { 1 + i % 2 } else { 0 }], *x, *y);
        }
        let eye = Color::Rgb(46, 55, 85);
        let light = Color::Rgb(251, 248, 255);
        let sleep = self.timeline.id == ClipId::Sleep && self.timeline.index >= 2;
        let expression = self.face.expression;
        let facing = self.facing_override.unwrap_or(self.pet.facing);
        for (i, cx) in [10, 20].into_iter().enumerate() {
            let cy = face_y
                + if expression == Expression::Curious && i == 1 {
                    1
                } else {
                    0
                };
            let closed = sleep
                || (self.animate && self.face.blink())
                || (expression == Expression::Wink && i == 1);
            if closed {
                body.line(cx, cy + 2, 4, eye);
                body.put(cx + 4, cy + 1, eye);
            } else if expression == Expression::Happy {
                body.line(cx + 1, cy, 2, eye);
                body.put(cx, cy + 1, eye);
                body.put(cx + 3, cy + 1, eye);
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
                body.put(cx + 2, cy + tall - 1, glow);
                if expression == Expression::Sad {
                    body.put(cx + if i == 0 { 3 } else { 0 }, cy - 2, eye);
                }
                if matches!(expression, Expression::Curious | Expression::Squint) {
                    body.line(cx, cy - 2, 2, eye);
                }
            }
        }
        let mouth_y = face_y + 7;
        if (expression == Expression::Talk && (self.face.clock * 9.0).sin() > -0.2)
            || (expression == Expression::Yawn && (1.0..1.7).contains(&(self.face.clock % 3.2)))
            || expression == Expression::Surprised
        {
            let height = if expression == Expression::Yawn { 4 } else { 2 };
            for dy in 0..height {
                body.line(15, mouth_y + dy, 3, eye);
            }
            body.line(15, mouth_y + height - 1, 3, Color::Rgb(218, 130, 168));
        } else if expression == Expression::Sad {
            body.line(15, mouth_y, 3, eye);
            body.put(14, mouth_y + 1, eye);
            body.put(18, mouth_y + 1, eye);
        } else {
            body.put(14, mouth_y, eye);
            body.line(15, mouth_y + 1, 3, eye);
            body.put(18, mouth_y, eye);
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
        let low = Color::Rgb(52, 65, 87);
        world.line(1, ground, width as i32 - 2, low);
        // Resting cushion and a small sprout stay secondary to the character.
        let resting =
            (16.0 + self.pet.traits.life.favourite_rest * (width as f32 - 34.0).max(0.0)) as i32;
        world.art(
            &[
                "  oooooooooooooo  ",
                " obccccddddccccbo ",
                "obbbbbbbbbbbbbbbbo",
                " oooooooooooooooo ",
            ],
            resting - 9,
            ground - 4,
        );
        world.line(resting - 5, ground - 3, 10, Color::Rgb(151, 120, 172));
        let plant = (16.0 + crate::life::PLANT * (width as f32 - 34.0).max(0.0)) as i32;
        world.art(
            &[" e e ", " eee ", "  e  ", " bbb ", " bbb "],
            plant - 2,
            ground - 5,
        );
        let body = self.body(self.timeline.frame(), self.timeline.index)?;
        let facing = self.facing_override.unwrap_or(self.pet.facing);
        let t = if self.animate { self.face.clock } else { 0.0 };
        let light = Color::Rgb(251, 248, 255);
        let sleep = self.timeline.id == ClipId::Sleep && self.timeline.index >= 2;
        // Smooth pose transforms around the grounded feet. World position is normalized.
        let x0 = (self.visual_x * (width as f32 - 34.0).max(0.0)) as i32;
        let extent = |shape| match shape {
            Shape::Stand | Shape::Stretch => 26.0,
            Shape::Seat => 20.0,
            Shape::Curl => 15.0,
        };
        let ratio = extent(self.previous_frame.shape) / extent(self.timeline.frame().shape);
        let blend = self.transition * self.transition * (3.0 - 2.0 * self.transition);
        let height_scale = self.pose.squash * (ratio + (1.0 - ratio) * blend);
        let width_scale = (1.0 / height_scale).sqrt().clamp(0.88, 1.15);
        world.line(x0 + 7, ground - 1, 20, Color::Rgb(28, 33, 51));
        // Inverse sampling fills stretched rows instead of leaving forward-mapping gaps.
        for dy in -37..=2 {
            let sy = (dy as f32 / height_scale + 30.0).round() as i32;
            if !(0..32).contains(&sy) {
                continue;
            }
            for dx in -5..38 {
                let sx = ((dx as f32 - 16.0) / width_scale + 16.0
                    - self.pose.lean * facing * (1.0 - sy as f32 / 32.0))
                    .round() as i32;
                let sx = if facing < 0.0 { 31 - sx } else { sx };
                if sx >= 0
                    && let Some(color) = body.get(sx as usize, sy as usize)
                {
                    world.put(
                        x0 + dx,
                        ground - 1 + dy - self.pose.lift.round() as i32,
                        color,
                    );
                }
            }
        }
        let play = !self.pet.moving
            && matches!(
                self.pet.activity,
                Activity::Play | Activity::Chase | Activity::Balance
            );
        let orb_x = 16.0 + self.orb_x * (width as f32 - 34.0).max(0.0);
        let orb_y = ground as f32 - self.orb_height;
        let pulse = if self.animate {
            (t * 3.0).sin() * 12.0 * self.intensity
        } else {
            0.0
        };
        let glow = Color::Rgb(
            105,
            (215.0 + pulse) as u8,
            if self.pet.thinking { 240 } else { 206 },
        );
        for dy in -2i32..=2 {
            for dx in -2i32..=2 {
                if dx * dx + dy * dy <= 5 {
                    world.put(
                        orb_x as i32 + dx,
                        orb_y as i32 + dy,
                        if (dx as f32 - (t * 2.0).cos()).abs() < 1.0 && dy < 0 {
                            light
                        } else if dx + dy > 2 {
                            Color::Rgb(45, 135, 139)
                        } else {
                            glow
                        },
                    );
                }
            }
        }
        // Six retained trail samples; no per-frame particle allocation.
        if self.animate
            && self.intensity > 0.0
            && (play || self.preview == Some(ClipId::Play) || self.pet.thinking)
        {
            for (i, (x, h)) in self.trail.iter().enumerate().skip(1) {
                let c = Color::Rgb(45, 145 - i as u8 * 12, 140 - i as u8 * 10);
                world.put(
                    (16.0 + x * (width as f32 - 34.0).max(0.0)) as i32,
                    (ground as f32 - h) as i32,
                    c,
                );
            }
        }
        if self.animate && self.intensity > 0.0 {
            for n in 0..3 {
                let phase = t * 0.7 + n as f32 * 2.1;
                world.put(
                    (width as f32 * (0.2 + n as f32 * 0.3)) as i32,
                    ground - 5 - (phase.sin() * 2.0) as i32,
                    Color::Rgb(49, 76, 85),
                );
            }
            if self.land_flash > 0.0 {
                for dx in [-3, 0, 29, 33] {
                    world.put(x0 + dx, ground - 2, low);
                }
            }
        }
        if self.guides {
            for x in 0..32 {
                world.put(x0 + x, ground - 31, Color::Rgb(90, 100, 120));
            }
            world.line(x0 + 14, ground - 1, 5, Color::Rgb(255, 190, 80));
            for y in 0..32 {
                world.put(x0 + 16, ground - y, Color::Rgb(80, 100, 110));
            }
        }
        if sleep {
            world.art(
                &["eeeee", "   e ", "  e  ", " e   ", "eeeee"],
                x0 + 26,
                ground - 24,
            );
        }
        Ok(world)
    }
}
impl Widget for Terri {
    fn draw(&mut self, c: &mut Canvas<'_>, _: bool, theme: &Theme) {
        if let Some(error) = &self.fault {
            c.put_str(1, 1, &format!("Terri paused: {error}"), theme.focused);
            return;
        }
        let size = c.size();
        if size.width < 4 || size.height < 3 {
            return;
        }
        let scale = if size.width < 68 || size.height < 21 {
            2
        } else {
            1
        };
        let width = (usize::from(size.width) * scale).min(100);
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
                c.put_str(1, 1, "Terri paused · chat remains available", theme.focused);
            }
        }
    }
}
pub fn default_terri() -> Terri {
    let config = PetConfig::default();
    let state = Snapshot::new(config.seed);
    Terri::new(config, state)
}
pub fn validate_fps(fps: u32) -> io::Result<Duration> {
    if !(1..=60).contains(&fps) {
        return Err(io::Error::other("FPS must be 1–60"));
    }
    Ok(Duration::from_secs_f64(1.0 / f64::from(fps)))
}
#[cfg(test)]
mod tests {
    use super::*;
    use terrabox::{Rect, Size, Surface};
    #[test]
    fn faces_layers_and_rendering_are_repeatable() {
        let mut t = default_terri();
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
    fn reduced_motion_pause_and_orb_contacts() {
        let mut t = default_terri();
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
    fn authored_silhouettes_and_walk_contacts_differ() {
        let mut t = default_terri();
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
        assert_ne!(c.frames[0].feet, c.frames[2].feet);
        assert_ne!(c.frames[0].arms, c.frames[2].arms);
    }
    #[test]
    fn frame_clips_loop_or_hold_last_frame() {
        let clip = AnimationClip::new(&FEET, Duration::from_millis(100), true).unwrap();
        assert_eq!(clip.frame(Duration::from_millis(100)), FEET[1]);
        assert_eq!(clip.frame(Duration::from_millis(400)), FEET[0]);
        let clip = AnimationClip::new(&FEET, Duration::from_millis(100), false).unwrap();
        assert_eq!(clip.frame(Duration::from_secs(10)), FEET[3]);
        assert!(AnimationClip::new(&FEET, Duration::ZERO, true).is_none());
    }
    #[test]
    fn artwork_bounds_and_no_simulation_in_render() {
        assert!(BODY.iter().all(|r| r.len() <= 32));
        let mut t = default_terri();
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
        let mut t = default_terri();
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
        let mut t = default_terri();
        t.pet.x = t.pet.traits.life.favourite_rest;
        t.pet.demo_activity(Activity::Sleep);
        t.advance(Duration::from_millis(50));
        assert_eq!(t.timeline.id, ClipId::Sleep);
        assert!(t.pose.squash.is_finite());
        t.intensity = f32::NAN;
        assert!(t.compose(40, 40).is_err());
    }
}
