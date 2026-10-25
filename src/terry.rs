//! Procedural terminal-cell character. Each cell holds two independently shaded pixels.
use std::time::{Duration, Instant};
use terrabox::{Canvas, Color, Style};
use terratui::{Theme, Widget};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Idle,
    Listening,
    Thinking,
    Responding,
    Happy,
    Confused,
    Error,
    // Reserved until independently enforced policy events exist.
    #[allow(dead_code)]
    SecurityBlocked,
}
impl State {
    pub fn label(self) -> &'static str {
        match self {
            Self::Idle => "Ready when you are",
            Self::Listening => "Listening",
            Self::Thinking => "Thinking",
            Self::Responding => "Responding",
            Self::Happy => "Happy to help",
            Self::Confused => "Let's try again",
            Self::Error => "Needs a little help",
            Self::SecurityBlocked => "Security blocked",
        }
    }
}
pub struct Terry {
    pub state: State,
    since: Instant,
    pub elapsed: Duration,
}
impl Terry {
    pub fn new(now: Instant) -> Self {
        Self {
            state: State::Idle,
            since: now,
            elapsed: Duration::ZERO,
        }
    }
    pub fn set(&mut self, state: State, now: Instant) {
        if self.state != state {
            self.state = state;
            self.since = now;
        }
    }
    pub fn settle(&mut self, now: Instant, typing: bool) {
        if self.state == State::Happy
            && now.saturating_duration_since(self.since) >= Duration::from_secs(2)
        {
            self.set(
                if typing {
                    State::Listening
                } else {
                    State::Idle
                },
                now,
            );
        }
    }
    fn pixel(&self, x: f64, y: f64, bg: Color) -> Color {
        let t = self.elapsed.as_secs_f64();
        let active = matches!(self.state, State::Thinking | State::Responding);
        let accent = if matches!(self.state, State::Error | State::SecurityBlocked) {
            Color::Rgb(255, 107, 123)
        } else {
            Color::Rgb(93, 238, 216)
        };
        let bounce = if self.state == State::Happy {
            (t * 9.0).sin().abs() * 1.6
        } else {
            (t * 2.1).sin() * 0.35
        };
        let y = y + bounce;
        // Floating thought orb and independent orbital particles.
        let orb_x = 27.5;
        let orb_y = 4.0 + (t * 2.0).sin() * 0.6;
        let orb = ((x - orb_x) / 2.6).powi(2) + ((y - orb_y) / 2.6).powi(2);
        if orb < 1.0 {
            return if x < orb_x && y < orb_y {
                Color::Rgb(220, 255, 245)
            } else if orb < 0.55 {
                accent
            } else {
                Color::Rgb(42, 125, 141)
            };
        }
        if active {
            for n in 0..3 {
                let a = t * 3.0 + n as f64 * 2.094;
                if (x - orb_x - a.cos() * 4.0).abs() < 0.65
                    && (y - orb_y - a.sin() * 3.1).abs() < 0.65
                {
                    return accent;
                }
            }
        }
        // Ground shadow, feet and articulated hands.
        if ((x - 17.5) / 12.0).powi(2) + ((y - 34.5) / 1.2).powi(2) < 1.0 {
            return Color::Rgb(24, 37, 52);
        }
        if (28.0..33.0).contains(&y) && ((9.0..15.0).contains(&x) || (21.0..27.0).contains(&x)) {
            return if y < 30.0 {
                Color::Rgb(70, 104, 135)
            } else {
                Color::Rgb(154, 196, 209)
            };
        }
        let hand_y = if self.state == State::Happy {
            20.0 - (t * 9.0).sin() * 1.5
        } else {
            24.0
        };
        if ((x - 4.5) / 2.4).powi(2) + ((y - hand_y) / 4.0).powi(2) < 1.0
            || ((x - 30.5) / 2.4).powi(2) + ((y - hand_y) / 4.0).powi(2) < 1.0
        {
            return if !(5.0..=31.0).contains(&x) {
                Color::Rgb(87, 137, 161)
            } else {
                Color::Rgb(172, 220, 223)
            };
        }
        // Rounded chassis with directional lighting and a darker lower lip.
        let tilt = if self.state == State::Confused {
            (x - 18.0) * 0.08
        } else {
            0.0
        };
        let y = y + tilt;
        if rounded(x, y, 6.0, 9.0, 29.0, 29.0, 6.0) {
            let shade = ((x - 6.0) * 1.4 + (y - 9.0) * 2.0) as u8;
            let mut body = Color::Rgb(
                218u8.saturating_sub(shade),
                245u8.saturating_sub(shade),
                241u8.saturating_sub(shade / 2),
            );
            if y > 26.0 {
                body = Color::Rgb(82, 137, 158);
            }
            // Visor rim and glass.
            if rounded(x, y, 8.0, 12.0, 27.0, 23.0, 3.5) {
                body = Color::Rgb(60, 110, 133);
            }
            if rounded(x, y, 9.0, 13.0, 26.0, 22.0, 3.0) {
                body = if y < 15.0 {
                    Color::Rgb(27, 53, 74)
                } else {
                    Color::Rgb(13, 28, 45)
                };
                let blink = t % 5.7 > 5.48 && self.state == State::Idle;
                let gaze = if self.state == State::Listening {
                    0.8
                } else {
                    0.0
                };
                for (i, cx) in [13.0 + gaze, 22.0 + gaze].into_iter().enumerate() {
                    let eye_y = if self.state == State::Confused && i == 1 {
                        16.0
                    } else {
                        17.0
                    };
                    let dx = (x - cx).abs();
                    let dy = (y - eye_y).abs();
                    let lit = if blink {
                        dx < 2.0 && dy < 0.6
                    } else if self.state == State::Happy {
                        dx < 2.2 && (y - eye_y + 1.0 - dx * 0.6).abs() < 0.7
                    } else if self.state == State::SecurityBlocked {
                        dx < 2.0 && (dy < 0.6 || dx < 0.6)
                    } else {
                        dx < 1.8 && dy < if self.state == State::Error { 1.2 } else { 2.5 }
                    };
                    if lit {
                        body = if x < cx && y < eye_y {
                            Color::Rgb(226, 255, 247)
                        } else {
                            accent
                        };
                    }
                }
                if (16.0..19.5).contains(&x) && (y - 20.0).abs() < 0.55 {
                    body = accent;
                }
                if self.state == State::Responding
                    && (t * 8.0).sin() > 0.0
                    && (17.0..19.0).contains(&x)
                    && (20.0..22.0).contains(&y)
                {
                    body = accent;
                }
            }
            // Chest power core, tiny vents and specular highlights.
            if ((x - 17.5) / 1.8).powi(2) + ((y - 25.0) / 1.3).powi(2) < 1.0 {
                body = accent;
            }
            if (23.0..26.0).contains(&x) && (y - 25.0).abs() < 0.5 {
                body = Color::Rgb(68, 115, 141);
            }
            if (10.0..16.0).contains(&x) && (y - 10.5).abs() < 0.5 {
                body = Color::Rgb(242, 255, 252);
            }
            if self.state == State::SecurityBlocked {
                let dx = (x - 17.5).abs();
                if (23.0..28.5).contains(&y) && dx < (29.0 - y) * 0.65 {
                    body = if dx < 0.6 && y < 26.5 {
                        Color::Rgb(45, 24, 40)
                    } else {
                        accent
                    };
                }
            }
            return body;
        }
        // Antenna connects Terry to the floating activity orb.
        if (x - 18.0).abs() < 0.65 && (6.0..10.0).contains(&y) {
            return Color::Rgb(108, 169, 184);
        }
        if (x - 18.0).abs() < 1.2 && (y - 6.0).abs() < 1.0 {
            return accent;
        }
        bg
    }
}
fn rounded(x: f64, y: f64, left: f64, top: f64, right: f64, bottom: f64, r: f64) -> bool {
    let cx = x.clamp(left + r, right - r);
    let cy = y.clamp(top + r, bottom - r);
    x >= left
        && x <= right
        && y >= top
        && y <= bottom
        && (x - cx).powi(2) + (y - cy).powi(2) <= r * r
}
impl Widget for Terry {
    fn draw(&mut self, canvas: &mut Canvas<'_>, _: bool, theme: &Theme) {
        let size = canvas.size();
        let small = size.width < 36 || size.height < 20;
        let width = if small { 18 } else { 36 };
        let height = if small { 9 } else { 18 };
        let x0 = size.width.saturating_sub(width) / 2;
        let y0 = size.height.saturating_sub(height + 2) / 2;
        for row in 0..height.min(size.height) {
            for col in 0..width.min(size.width) {
                let scale = if small { 2.0 } else { 1.0 };
                let top = self.pixel(
                    f64::from(col) * scale,
                    f64::from(row * 2) * scale,
                    theme.surface.background,
                );
                let bottom = self.pixel(
                    f64::from(col) * scale,
                    f64::from(row * 2 + 1) * scale,
                    theme.surface.background,
                );
                canvas.put_char(x0 + col, y0 + row, '▀', Style::new(top, bottom));
            }
        }
        let label = self.state.label();
        let x = size
            .width
            .saturating_sub(terrabox::text_width(label) as u16)
            / 2;
        canvas.put_str(x, y0 + height + 1, label, theme.focused);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn eyes_and_orb_animate_independently() {
        let now = Instant::now();
        let mut terry = Terry::new(now);
        let bg = Color::Rgb(13, 20, 32);
        let open_eye = terry.pixel(13.0, 16.0, bg);
        terry.elapsed = Duration::from_millis(5500);
        assert_ne!(open_eye, terry.pixel(13.0, 16.0, bg));
        terry.set(State::Thinking, now);
        terry.elapsed = Duration::ZERO;
        let particle = terry.pixel(31.5, 4.0, bg);
        terry.elapsed = Duration::from_secs(1);
        assert_ne!(particle, terry.pixel(31.5, 4.0, bg));
    }
    #[test]
    fn happy_settles_and_active_states_persist() {
        let now = Instant::now();
        let mut t = Terry::new(now);
        t.set(State::Happy, now);
        t.settle(now + Duration::from_secs(1), false);
        assert_eq!(t.state, State::Happy);
        t.settle(now + Duration::from_secs(2), true);
        assert_eq!(t.state, State::Listening);
        for state in [
            State::Thinking,
            State::Responding,
            State::Error,
            State::Confused,
            State::SecurityBlocked,
        ] {
            t.set(state, now);
            t.settle(now + Duration::from_secs(20), false);
            assert_eq!(t.state, state);
        }
    }
}
