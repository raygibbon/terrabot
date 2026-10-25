//! Interactive character laboratory. No Ollama or personality-file access.
use crate::{
    animation::{Palette, Terri, default_terri, validate_fps},
    pet::{Activity, Mood, PetEvent},
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
    next_cycle: Instant,
    index: usize,
    mood: usize,
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
                    t.advance(elapsed);
                    if self.auto && now >= self.next_cycle {
                        t.pet.demo_activity(Activity::ALL[self.index % 10]);
                        self.index += 1;
                        self.next_cycle = now + Duration::from_secs(4);
                    }
                    if self.fake_end.is_some_and(|end| now >= end) {
                        t.event(PetEvent::Done("Here is a careful explanation."));
                        self.fake_end = None;
                    }
                }
                if self.clock.poll(now).is_some()
                    && c.desktop
                        .widget_mut::<Terri>(self.id)
                        .is_some_and(|t| t.animate && t.fault.is_none())
                {
                    self.frames += 1;
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
                        Key::Char('m') => {
                            self.mood = (self.mood + 1) % Mood::ALL.len();
                            t.pet.traits.mood = Mood::ALL[self.mood];
                        }
                        Key::Char('n') => {
                            t.pet.demo_activity(Activity::ALL[self.index % 10]);
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
                            self.next_cycle = now;
                        }
                        Key::Char('r') => {
                            *t = default_terri();
                            self.fake_end = None;
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
    let mut auto = false;
    let mut seconds = None;
    let mut palette = Palette::TrueColor;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--auto" => auto = true,
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
                    "Terri demo: --auto --fps 20 --seconds 10 --indexed --ascii\nM mood · N next activity · B boredom · E lower energy · S sleep · W wake · F fake response · D denial · R reset · A cycle · P animation · +/- intensity · Q/Esc/Ctrl+C quit"
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
    let mut terri = default_terri();
    terri.palette = palette;
    let id = desktop.add_window(terri);
    desktop.layout = Layout::Window(id);
    desktop.status.text=" TERRI · M mood · N pose · B bored · E energy · S sleep · W wake · F speak · D denial · R reset · A auto · P pause · +/- motion · Q quit".into();
    let metrics = std::rc::Rc::new(std::cell::Cell::new((0, Duration::ZERO)));
    let demo = Demo {
        id,
        last: now,
        clock: FrameClock::new(interval, now)
            .ok_or_else(|| io::Error::other("Invalid frame interval"))?,
        auto,
        next_cycle: now,
        index: 0,
        mood: 0,
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
