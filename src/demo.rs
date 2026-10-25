//! Interactive character laboratory. No Ollama or personality-file access.
use crate::{
    animation::{Palette, Terri, validate_fps},
    config::PetConfig,
    motion::{ClipId, Expression},
    pet::{Activity, Mood, PetEvent, Snapshot},
};
use std::{
    io,
    time::{Duration, Instant},
};
use terrabox::{Color, Deadline, Event, FrameClock, Key, Modifiers, Style};
use terratui::{App, AppEvent, Application, Context, Desktop, Layout, Theme, TimerId, WindowId};
struct Demo {
    id: WindowId,
    last: Instant,
    clock: FrameClock,
    auto: bool,
    seed: u64,
    index: usize,
    mood: usize,
    clip_index: usize,
    expression: usize,
    showcase: bool,
    next_showcase: Instant,
    fake_end: Option<Instant>,
    stop: Option<Instant>,
    frames: u64,
    start: Instant,
    metrics: std::rc::Rc<std::cell::Cell<(u64, Duration)>>,
}
impl App for Demo {
    type Message = ();
    fn init(&mut self, c: &mut Context<'_, ()>) -> io::Result<()> {
        c.set_timer(TimerId(1), Deadline(Instant::now()));
        Ok(())
    }
    fn event(&mut self, event: AppEvent<()>, c: &mut Context<'_, ()>) -> io::Result<()> {
        let now = Instant::now();
        match event {
            AppEvent::Timer(_) => {
                if self.stop.is_some_and(|end| now >= end) {
                    c.quit();
                    return Ok(());
                }
                let elapsed = now - self.last;
                self.last = now;
                if let Some(t) = c.desktop.widget_mut::<Terri>(self.id) {
                    if self.showcase && now >= self.next_showcase {
                        t.preview_clip(ClipId::ALL[self.clip_index % ClipId::ALL.len()]);
                        self.clip_index += 1;
                        self.next_showcase = now + Duration::from_secs(4);
                    }
                    t.advance(elapsed);
                    if self.fake_end.is_some_and(|end| now >= end) {
                        t.event(PetEvent::Done("Here is a careful explanation."));
                        self.fake_end = None;
                    }
                }
                if self.clock.poll(now).is_some()
                    && c.desktop
                        .widget_mut::<Terri>(self.id)
                        .is_some_and(|t| t.animate && !t.paused && t.fault.is_none())
                {
                    self.frames += 1;
                    if let Some(t) = c.desktop.widget_mut::<Terri>(self.id)
                        && t.preview.is_some()
                    {
                        c.desktop.status.text = format!(
                            " TERRI preview · {:.1} FPS · [/] clip · Space pause · . step · ,/; speed · X face · V facing · G guides · Enter replay · Z life · Q quit",
                            self.frames as f64 / self.start.elapsed().as_secs_f64().max(0.001)
                        );
                    }
                    c.redraw();
                }
                let deadline = self
                    .clock
                    .deadline()
                    .unwrap_or(Deadline(now + Duration::from_millis(50)));
                c.set_timer(
                    TimerId(1),
                    Deadline(deadline.0.min(now + Duration::from_millis(50))),
                );
            }
            AppEvent::Terminal(Event::Key(k)) => {
                if k.key == Key::Escape
                    || matches!(k.key, Key::Char('q' | 'Q'))
                    || k.modifiers.contains(Modifiers::CTRL)
                        && matches!(k.key, Key::Char('C' | 'c'))
                {
                    c.quit();
                    return Ok(());
                }
                if let Some(t) = c.desktop.widget_mut::<Terri>(self.id) {
                    match k.key {
                        Key::Char('[' | ']') => {
                            self.showcase = false;
                            self.clip_index = if k.key == Key::Char(']') {
                                (self.clip_index + 1) % 16
                            } else {
                                (self.clip_index + 15) % 16
                            };
                            t.preview_clip(ClipId::ALL[self.clip_index]);
                        }
                        Key::Char(' ') => t.paused = !t.paused,
                        Key::Char('.') => {
                            t.paused = true;
                            t.step_frame();
                        }
                        Key::Char(',') => t.speed = (t.speed / 1.25).max(0.1),
                        Key::Char(';') => t.speed = (t.speed * 1.25).min(3.0),
                        Key::Char('x') => {
                            self.expression = (self.expression + 1) % Expression::ALL.len();
                            t.face.override_expression = Some(Expression::ALL[self.expression]);
                            t.face.expression = Expression::ALL[self.expression];
                        }
                        Key::Char('v') => {
                            t.facing_override = Some(-t.facing_override.unwrap_or(t.pet.facing))
                        }
                        Key::Char('g') => t.guides = !t.guides,
                        Key::Char('z') => {
                            t.preview = None;
                            self.showcase = false;
                            t.face.override_expression = None;
                            t.facing_override = None;
                            t.paused = false;
                        }
                        Key::Enter => {
                            if let Some(id) = t.preview {
                                t.preview_clip(id);
                            }
                        }
                        Key::Char('m') => {
                            self.mood = (self.mood + 1) % Mood::ALL.len();
                            t.pet.traits.mood = Mood::ALL[self.mood];
                        }
                        Key::Char('n') => {
                            t.pet
                                .demo_activity(Activity::ALL[self.index % Activity::ALL.len()]);
                            self.index += 1;
                        }
                        Key::Char('b') => {
                            t.pet.traits.boredom = (t.pet.traits.boredom + 0.25).min(1.0)
                        }
                        Key::Char('e') => {
                            t.pet.traits.energy = (t.pet.traits.energy - 0.25).max(0.0)
                        }
                        Key::Char('s') => t.pet.demo_activity(Activity::Sleep),
                        Key::Char('w') => t.event(PetEvent::Typing),
                        Key::Char('f') => {
                            t.event(PetEvent::User("Explain a useful idea"));
                            t.event(PetEvent::Speaking);
                            self.fake_end = Some(now + Duration::from_secs(3));
                        }
                        Key::Char('d') => t.event(PetEvent::Denied),
                        Key::Char('a') => {
                            self.auto = !self.auto;
                            t.pet.autonomous = self.auto;
                        }
                        Key::Char('r') => {
                            *t = Terri::new(
                                PetConfig {
                                    seed: self.seed,
                                    autonomous: self.auto,
                                    diagnostics: t.diagnostics,
                                    ..PetConfig::default()
                                },
                                Snapshot::new(self.seed),
                            );
                            self.fake_end = None;
                        }
                        Key::Char('i') => t.diagnostics = !t.diagnostics,
                        Key::Char('o') => t.pet.interact(crate::life::Action::Orb),
                        Key::Char('c') => t.pet.interact(crate::life::Action::Play),
                        Key::Char('h') => t.pet.interact(crate::life::Action::Attention),
                        Key::Char('u') => {
                            t.pet.traits.curiosity = (t.pet.traits.curiosity + 0.25).min(1.0)
                        }
                        Key::Char('l') => {
                            t.pet.traits.life.comfort = (t.pet.traits.life.comfort - 0.25).max(0.0)
                        }
                        Key::Char('t') => {
                            t.pet.set_hour(Some(if t.pet.is_night() { 12 } else { 23 }))
                        }
                        Key::Char('p') => t.animate = !t.animate,
                        Key::Char('-') => t.intensity = (t.intensity - 0.25).max(0.0),
                        Key::Char('+') => t.intensity = (t.intensity + 0.25).min(1.0),
                        _ => {}
                    }
                }
                c.redraw();
            }
            _ => {}
        }
        Ok(())
    }
    fn shutdown(&mut self, _: &mut Context<'_, ()>) -> io::Result<()> {
        self.metrics.set((self.frames, self.start.elapsed()));
        Ok(())
    }
}
pub fn run() -> io::Result<()> {
    let mut fps = 20;
    let mut auto = true;
    let mut showcase = false;
    let mut selected_clip = None;
    let mut diagnostics = false;
    let mut seed = PetConfig::default().seed;
    let mut seconds = None;
    let mut palette = Palette::TrueColor;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--auto" => auto = true,
            "--showcase" => showcase = true,
            "--clip" => {
                let name = args
                    .next()
                    .ok_or_else(|| io::Error::other("Missing clip"))?;
                selected_clip = Some(
                    ClipId::ALL
                        .into_iter()
                        .find(|id| format!("{id:?}").eq_ignore_ascii_case(&name))
                        .ok_or_else(|| io::Error::other("Unknown animation clip"))?,
                );
            }
            "--manual" => auto = false,
            "--diagnostics" => diagnostics = true,
            "--seed" => {
                seed = args
                    .next()
                    .ok_or_else(|| io::Error::other("Missing seed"))?
                    .parse()
                    .map_err(io::Error::other)?
            }
            "--fps" => {
                fps = args
                    .next()
                    .ok_or_else(|| io::Error::other("Missing FPS"))?
                    .parse()
                    .map_err(io::Error::other)?
            }
            "--seconds" => {
                let n: u64 = args
                    .next()
                    .ok_or_else(|| io::Error::other("Missing duration"))?
                    .parse()
                    .map_err(io::Error::other)?;
                if !(1..=3600).contains(&n) {
                    return Err(io::Error::other("Duration must be 1–3600 seconds"));
                }
                seconds = Some(n);
            }
            "--indexed" => palette = Palette::Indexed,
            "--ascii" => palette = Palette::Ascii,
            "--help" => {
                println!(
                    "Terri demo: --showcase --clip Jump --auto --manual --diagnostics --seed 7 --fps 20 --seconds 10 --indexed --ascii\nM mood · N next activity · B boredom · E lower energy · S sleep · W wake · F fake response · D denial · R reset · A autonomous · I diagnostics · O orb · C chase · H attention · U curiosity · L comfort · T day/night · P animation · +/- intensity · Q/Esc/Ctrl+C quit\nPreview: [/] clip · Space pause · . next frame · ,/; speed · X expression · V facing · G guides · Enter replay · Z life"
                );
                return Ok(());
            }
            _ => return Err(io::Error::other(format!("Unknown option: {arg}"))),
        }
    }
    let now = Instant::now();
    let interval = validate_fps(fps)?;
    let mut desktop = Desktop::default();
    let surface = Style::new(Color::Rgb(216, 222, 239), Color::Rgb(13, 20, 32));
    desktop.theme = Theme {
        surface,
        border: surface,
        focused: Style::new(Color::Rgb(120, 230, 211), surface.background),
        bar: surface,
        ..Theme::default()
    };
    let mut terri = Terri::new(
        PetConfig {
            seed,
            autonomous: auto,
            diagnostics,
            ..PetConfig::default()
        },
        Snapshot::new(seed),
    );
    terri.palette = palette;
    if let Some(id) = selected_clip {
        terri.preview_clip(id);
    }
    let id = desktop.add_window(terri);
    desktop.layout = Layout::Window(id);
    desktop.status.text=" TERRI · M mood · N activity · B bored · E energy · S sleep · W wake · F speak · D denial · R reset · A auto · I inspect · O orb · C chase · P pause · +/- motion · Q quit".into();
    let metrics = std::rc::Rc::new(std::cell::Cell::new((0, Duration::ZERO)));
    let demo = Demo {
        id,
        last: now,
        clock: FrameClock::new(interval, now)
            .ok_or_else(|| io::Error::other("Invalid frame interval"))?,
        auto,
        seed,
        index: 0,
        mood: 0,
        clip_index: selected_clip
            .and_then(|id| ClipId::ALL.iter().position(|v| *v == id))
            .unwrap_or(0),
        expression: 0,
        showcase,
        next_showcase: now,
        fake_end: None,
        stop: seconds.map(|s| now + Duration::from_secs(s)),
        frames: 0,
        start: now,
        metrics: metrics.clone(),
    };
    Application::new(demo, desktop).run()?;
    if seconds.is_some() {
        let (frames, elapsed) = metrics.get();
        println!(
            "terri-demo: frames={frames} elapsed={:.3}s observed_fps={:.2} target_fps={fps}",
            elapsed.as_secs_f64(),
            frames as f64 / elapsed.as_secs_f64().max(0.001)
        );
    }
    Ok(())
}
