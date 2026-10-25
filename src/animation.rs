//! Handcrafted pixel layers composed into a bounded logical grid, then TerraBox cells.
use crate::{
    config::PetConfig,
    cues::Gesture,
    pet::{Activity, Mood, Pet, PetEvent, Snapshot},
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
    fn target(p: &Pet, intensity: f32) -> Self {
        let t = p.phase;
        let amount = intensity.clamp(0.0, 1.0);
        let mut pose = Self {
            squash: 1.0 + (t * 2.0).sin() * 0.015 * amount,
            ..Self::default()
        };
        match p.activity {
            Activity::Wander => {
                pose.lift = (t * 8.0).sin().abs() * 0.8 * amount;
                pose.lean = p.facing * 0.5 * amount;
            }
            Activity::Sit => pose.squash = 0.9,
            Activity::Sleep => {
                pose.squash = 0.65 + (t * 1.3).sin() * 0.02 * amount;
                pose.lean = 1.0;
            }
            Activity::Wake => {
                pose.squash = 1.08;
                pose.arm = 1.0;
            }
            Activity::Play => {
                pose.lean = (t * 3.0).sin() * 1.5 * amount;
                pose.arm = 1.0;
            }
            Activity::Investigate => pose.lean = 2.0 * amount,
            Activity::Dance | Activity::Celebrate => {
                pose.lift = (t * 6.0).sin().max(0.0) * 3.5 * amount;
                pose.squash = 1.0 - (t * 6.0).cos() * 0.06 * amount;
                pose.lean = (t * 3.0).sin() * 2.0 * amount;
                pose.arm = 1.0;
            }
            Activity::Stumble => {
                pose.lean = (p.age * 3.0).sin() * 5.0 * amount;
                pose.squash = 0.82;
            }
            Activity::Watch => {
                pose.lean = 0.5 * amount;
            }
        }
        if p.gesture == Gesture::HeadTilt {
            pose.lean += 2.0 * p.cue_intensity * amount;
        }
        if p.gesture == Gesture::Nod {
            pose.squash += (t * 5.0).sin() * 0.04 * p.cue_intensity * amount;
        }
        if p.gesture == Gesture::Wave {
            pose.arm = 1.0;
        }
        pose
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
}
impl Terri {
    pub fn new(config: PetConfig, state: Snapshot) -> Self {
        Self {
            pet: Pet::new(config, state),
            intensity: 1.0,
            animate: true,
            palette: Palette::TrueColor,
            fault: None,
            pose: Pose::default(),
        }
    }
    pub fn advance(&mut self, elapsed: Duration) {
        self.pet.advance(elapsed);
        let target = Pose::target(&self.pet, if self.animate { self.intensity } else { 0.0 });
        self.pose.approach(target, elapsed.as_secs_f32());
    }
    pub fn event(&mut self, event: PetEvent<'_>) {
        self.pet.event(event);
        if !self.animate {
            self.pose = Pose::target(&self.pet, 0.0);
        }
    }
    pub fn label(&self) -> String {
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
        let glow = Color::Rgb(105, 223, 206);
        world.line(1, ground, width as i32 - 2, low);
        // Resting cushion and a small sprout stay secondary to the character.
        for dy in 0..2 {
            world.line(3, ground - 1 - dy, 8, Color::Rgb(80, 71, 116));
        }
        world.put(width as i32 - 5, ground - 1, glow);
        world.put(width as i32 - 5, ground - 2, glow);
        world.put(width as i32 - 6, ground - 3, glow);
        let mut body = Sprite::new(32, 32).ok_or("Invalid sprite bounds")?;
        body.art(BODY, 1, 1);
        let t = if self.animate {
            self.pet.phase + (0.05 - self.pet.remaining().as_secs_f32())
        } else {
            0.0
        };
        let walking = self.pet.activity == Activity::Wander && self.animate;
        if walking {
            let clip = AnimationClip::new(&FEET, Duration::from_millis(125), true)
                .ok_or("Invalid walk clip")?;
            let elapsed = Duration::from_secs_f32(t);
            body.art(clip.frame(elapsed), 7, 27);
            body.art(clip.frame(elapsed + Duration::from_millis(250)), 20, 27);
        } else {
            body.art(FEET[0], 7, 27);
            body.art(FEET[0], 20, 27);
        }
        let hand = if self.pose.arm > 0.5 {
            1 + (t * 3.0) as usize % 2
        } else {
            0
        };
        body.art(HANDS[hand], 0, if hand > 0 { 12 } else { 19 });
        body.art(HANDS[hand], 28, if hand > 0 { 11 } else { 19 });
        let eye = Color::Rgb(46, 55, 85);
        let light = Color::Rgb(251, 248, 255);
        let sleep = self.pet.activity == Activity::Sleep;
        let mood = self.pet.traits.mood;
        for (i, cx) in [10, 20].into_iter().enumerate() {
            let cy = 12
                + if mood == Mood::Confused && i == 1 {
                    1
                } else {
                    0
                };
            if sleep
                || (self.animate && self.pet.blink)
                || matches!(mood, Mood::Bored | Mood::Sleepy)
            {
                body.line(cx, cy + 2, 4, eye);
            } else if matches!(mood, Mood::Happy | Mood::Excited) {
                body.line(cx + 1, cy, 2, eye);
                body.put(cx, cy + 1, eye);
                body.put(cx + 3, cy + 1, eye);
            } else {
                let tall = if mood == Mood::Surprised {
                    6
                } else if mood == Mood::Shy {
                    3
                } else {
                    5
                };
                for y in 0..tall {
                    body.line(cx, cy + y, 4, eye);
                }
                let gaze = if self.pet.activity == Activity::Watch {
                    1
                } else {
                    0
                };
                body.put(cx + 1 + gaze, cy, light);
                body.put(cx + 1 + gaze, cy + 1, light);
                body.put(cx + 2, cy + tall - 1, glow);
                if mood == Mood::Frustrated {
                    body.line(cx, cy - 1, 4, eye);
                }
                if matches!(mood, Mood::Curious | Mood::Focused) {
                    body.line(cx, cy - 2, 2, eye);
                }
            }
        }
        if self.pet.speaking && (t * 10.0).sin() > 0.0 {
            body.line(15, 18, 3, eye);
            body.line(15, 19, 3, eye);
        } else if self.pet.gentle || matches!(mood, Mood::Confused | Mood::Frustrated) {
            body.line(15, 19, 3, eye);
        } else {
            body.put(14, 18, eye);
            body.line(15, 19, 3, eye);
            body.put(18, 18, eye);
        }
        if self.pet.denied {
            for y in 0..7 {
                body.line(12 + y / 2, 20 + y, 8 - y, Color::Rgb(250, 105, 130));
            }
            body.line(15, 21, 2, eye);
            body.put(15, 23, eye);
        }
        // Smooth pose transforms around the grounded feet. World position is normalized.
        let x0 = (self.pet.x * (width as f32 - 34.0).max(0.0)) as i32;
        let height_scale = self.pose.squash.clamp(0.6, 1.15);
        // Inverse sampling fills stretched rows instead of leaving forward-mapping gaps.
        for dy in -37..=2 {
            let sy = (dy as f32 / height_scale + 30.0).round() as i32;
            if !(0..32).contains(&sy) {
                continue;
            }
            for dx in -5..38 {
                let sx = (dx as f32 - self.pose.lean * (1.0 - sy as f32 / 32.0)).round() as i32;
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
        let play = self.pet.activity == Activity::Play;
        let orb_x = if play {
            x0 as f32 + 16.0 + (t * 2.0).sin() * 12.0 * self.intensity
        } else {
            x0 as f32 + 27.0
        };
        let orb_y = if play {
            ground as f32 - 21.0 - (t * 2.0).cos().abs() * 10.0 * self.intensity
        } else {
            ground as f32 - 32.0 + (t * 2.0).sin() * self.intensity
        };
        for dy in -2i32..=2 {
            for dx in -2i32..=2 {
                if dx * dx + dy * dy <= 5 {
                    world.put(
                        orb_x as i32 + dx,
                        orb_y as i32 + dy,
                        if dx < 0 && dy < 0 { light } else { glow },
                    );
                }
            }
        }
        // Fixed upper bound: at most six independent particles, no growing collection.
        if self.animate
            && self.intensity > 0.0
            && (play
                || self.pet.thinking
                || matches!(self.pet.activity, Activity::Dance | Activity::Celebrate))
        {
            for n in 0..6 {
                let phase = t * 2.0 + n as f32 * 1.047;
                world.put(
                    (orb_x + phase.cos() * 5.0) as i32,
                    (orb_y + phase.sin() * 4.0) as i32,
                    if n % 2 == 0 { glow } else { light },
                );
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
        let scale = if size.width < 36 || size.height < 21 {
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
        t.pet.demo_activity(Activity::Sleep);
        t.advance(Duration::from_millis(50));
        assert!(t.pose.squash > 0.65 && t.pose.squash < 1.0);
        t.intensity = f32::NAN;
        assert!(t.compose(40, 40).is_err());
    }
}
