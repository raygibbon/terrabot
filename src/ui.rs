use crate::{
    animation::Terri,
    config::Config,
    models, persistence,
    pet::{PetEvent, Snapshot},
    provider::{Message, Role, Update, Worker},
};
use std::{
    io,
    time::{Duration, Instant},
};
use terrabox::{Canvas, Color, Deadline, Event, FrameClock, Key, Modifiers, Rect, Style};
use terratui::{
    App, AppEvent, Application, Axis, CommandId, Context, Desktop, EventResult, Layout, Length,
    Theme, TimerId, Widget, WindowId,
    widgets::{Group, Label, Panel, TextInput},
};
use unicode_segmentation::UnicodeSegmentation;

const SEND: CommandId = CommandId(1);
const EDIT: CommandId = CommandId(2);
const INPUT_LIMIT: usize = 8192;
const HISTORY_LIMIT: usize = 512 * 1024;
const TICK: TimerId = TimerId(1);
const QUIT: CommandId = CommandId(4);
const SELECT_MODEL: CommandId = CommandId(3);
struct ModelPicker {
    names: Vec<String>,
    list: Panel,
}
impl Widget for ModelPicker {
    fn draw(&mut self, c: &mut Canvas<'_>, focused: bool, theme: &Theme) {
        self.list.draw(c, focused, theme);
    }
    fn event(&mut self, event: &Event) -> EventResult {
        if matches!(event, Event::Key(k) if k.modifiers.contains(Modifiers::CTRL) && matches!(k.key, Key::Char('q' | 'Q' | 'c' | 'C')))
        {
            EventResult::command(QUIT)
        } else if matches!(event, Event::Key(k) if k.key == Key::Enter) {
            EventResult::command(SELECT_MODEL)
        } else {
            self.list.event(event)
        }
    }
}
struct Input {
    editor: TextInput,
}
impl Widget for Input {
    fn draw(&mut self, c: &mut Canvas<'_>, focused: bool, t: &Theme) {
        c.fill(t.bar);
        c.put_str(0, 0, "> ", t.focused);
        let size = c.size();
        self.editor.draw(
            &mut c.clip(Rect::new(2, 0, size.width.saturating_sub(2), 1)),
            focused,
            t,
        );
    }
    fn event(&mut self, e: &Event) -> EventResult {
        if matches!(e,Event::Key(k) if k.key==Key::Enter) {
            return EventResult::command(SEND);
        }
        // Bound before delegating, retaining TerraTUI's grapheme editing behavior.
        let extra = match e {
            Event::Paste(p) => p.bytes.len(),
            Event::Key(k) if k.modifiers == Modifiers::NONE || k.modifiers == Modifiers::SHIFT => {
                match k.key {
                    Key::Char(ch) => ch.len_utf8(),
                    _ => 0,
                }
            }
            _ => 0,
        };
        if extra > 0 && self.editor.text.len() + extra > INPUT_LIMIT {
            return EventResult::handled();
        }
        let result = self.editor.event(e);
        if result.consumed {
            EventResult::command(EDIT)
        } else {
            result
        }
    }
}
struct Transcript {
    messages: Vec<(String, String)>,
    lines: Vec<(bool, String)>,
    width: u16,
    dirty: bool,
    scroll: usize,
}
impl Transcript {
    fn new() -> Self {
        Self {
            messages: vec![],
            lines: vec![],
            width: 0,
            dirty: true,
            scroll: 0,
        }
    }
    fn add(&mut self, who: &str, text: String) {
        self.messages.push((who.into(), text));
        while self.messages.len() > 128
            || self
                .messages
                .iter()
                .map(|(a, b)| a.len() + b.len())
                .sum::<usize>()
                > HISTORY_LIMIT
        {
            self.messages.remove(0);
        }
        self.dirty = true;
        self.scroll = 0;
    }
    fn replace_notice(&mut self, text: String) -> bool {
        if self.messages.first().is_some_and(|(_, old)| old == &text) {
            return false;
        }
        self.messages.clear();
        self.add("Ollama status", text);
        true
    }
    fn append(&mut self, text: &str) {
        if let Some((_, last)) = self.messages.last_mut() {
            last.push_str(text);
            self.dirty = true;
        }
    }
}
fn clean(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_control() || *c == '\n')
        .collect()
}
fn wrap(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![];
    }
    let mut lines = vec![];
    for paragraph in text.split('\n') {
        let mut line = String::new();
        let mut used = 0;
        let mut wrapped = false;
        for word in paragraph.split_word_bounds() {
            let word_width = terrabox::text_width(word);
            if word_width <= width && used + word_width > width && !line.is_empty() {
                lines.push(line.trim_end().to_owned());
                line.clear();
                used = 0;
                wrapped = true;
            }
            if wrapped && line.is_empty() && word.chars().all(char::is_whitespace) {
                continue;
            }
            for g in word.graphemes(true) {
                let w = terrabox::text_width(g);
                if used + w > width && !line.is_empty() {
                    lines.push(std::mem::take(&mut line));
                    used = 0;
                    wrapped = true;
                }
                if w <= width {
                    line.push_str(g);
                    used += w;
                }
            }
        }
        lines.push(line);
    }
    lines
}
impl Widget for Transcript {
    fn draw(&mut self, c: &mut Canvas<'_>, _: bool, t: &Theme) {
        let size = c.size();
        let width = size.width.saturating_sub(2);
        if self.dirty || self.width != width {
            let previous = self.lines.len();
            self.lines.clear();
            for (who, text) in &self.messages {
                self.lines.push((true, who.clone()));
                self.lines.extend(
                    wrap(&clean(text), usize::from(width))
                        .into_iter()
                        .map(|s| (false, s)),
                );
                self.lines.push((false, String::new()));
            }
            if self.scroll > 0 && self.width == width {
                self.scroll = self
                    .scroll
                    .saturating_add(self.lines.len().saturating_sub(previous));
            }
            self.width = width;
            self.dirty = false;
        }
        self.scroll = self
            .scroll
            .min(self.lines.len().saturating_sub(usize::from(size.height)));
        let start = self
            .lines
            .len()
            .saturating_sub(usize::from(size.height) + self.scroll);
        for (row, (label, line)) in self
            .lines
            .iter()
            .skip(start)
            .take(usize::from(size.height))
            .enumerate()
        {
            c.put_str(
                1,
                row as u16,
                line,
                if *label { t.focused } else { t.surface },
            );
        }
        if self.scroll > 0 {
            c.put_str(1, 0, "↑ History · PgDn to return", t.selection);
        }
    }
}
pub struct Chat {
    group: WindowId,
    config: Config,
    input: WindowId,
    transcript: WindowId,
    terri: WindowId,
    header: WindowId,
    diagnostic: WindowId,
    models: Vec<models::Model>,
    configured_model: String,
    previous_model: Option<String>,
    picker: Option<WindowId>,
    retry_at: Option<Instant>,
    retries: u8,
    worker: Option<Worker>,
    history: Vec<Message>,
    pending: Option<String>,
    response: String,
    ready: bool,
    last_tick: Instant,
    cue_worker: Option<Worker>,
    state_path: Option<std::path::PathBuf>,
    clock: FrameClock,
    panel: u16,
}
impl Chat {
    fn widget<'a, T: Widget>(&self, c: &'a mut Context<'_, ()>, id: WindowId) -> Option<&'a mut T> {
        c.desktop
            .widget_mut::<Group>(self.group)?
            .widget_mut::<T>(id)
    }
    fn pet_event(&self, c: &mut Context<'_, ()>, event: PetEvent<'_>) {
        if let Some(t) = self.widget::<Terri>(c, self.terri) {
            t.event(event);
        }
    }
    fn transcript<'a>(&self, c: &'a mut Context<'_, ()>) -> Option<&'a mut Transcript> {
        self.widget::<Transcript>(c, self.transcript)
    }
    fn status(&mut self, c: &mut Context<'_, ()>, text: &str) {
        if let Some(h) = self.widget::<Label>(c, self.header) {
            h.0 = format!(
                " TERRABOT  /  {}     Ollama · {text}",
                self.config.agent.name
            );
        }
        c.desktop.status.text = format!(
            " {} · Tools disabled | Enter Send · Esc Cancel · PgUp/Dn Scroll · F2/F3 Panel · F5 Retry · F6 Models · Ctrl+Q Quit",
            self.config.ollama.model
        );
        c.redraw();
    }
    fn layout(&self, c: &mut Context<'_, ()>) {
        let size = c.size();
        let show = self.config.interface.show_terri && size.width >= 52 && size.height >= 14;
        let panel = if size.width < 90 || size.height < 23 {
            20
        } else {
            self.panel.min(size.width / 2)
        };
        let body = Layout::Split {
            axis: Axis::Horizontal,
            children: vec![
                (
                    Length::Fixed(panel),
                    if show {
                        Layout::Window(self.terri)
                    } else {
                        Layout::Hidden
                    },
                ),
                (Length::Flex(1), Layout::Window(self.transcript)),
            ],
        };
        let layout = Layout::Split {
            axis: Axis::Vertical,
            children: vec![
                (
                    Length::Fixed(u16::from(size.height >= 5)),
                    Layout::Window(self.header),
                ),
                (Length::Flex(1), body),
                (Length::Fixed(4), Layout::Window(self.diagnostic)),
                (Length::Fixed(1), Layout::Window(self.input)),
            ],
        };
        if let Some(group) = c.desktop.widget_mut::<Group>(self.group) {
            group.layout = layout;
            group.focus(self.input);
        }
        c.redraw();
    }
    fn schedule(&mut self, c: &mut Context<'_, ()>) {
        let size = c.size();
        let animated = self.config.interface.animation
            && self.config.interface.show_terri
            && size.width >= 52
            && size.height >= 14;
        let healthy = self
            .widget::<Terri>(c, self.terri)
            .is_some_and(|t| t.fault.is_none());
        let interval = if animated && healthy {
            let remaining = self
                .widget::<Terri>(c, self.terri)
                .map(|t| t.pet.remaining())
                .unwrap_or(Duration::from_millis(50));
            self.clock
                .deadline()
                .map(|d| d.remaining(Instant::now()))
                .unwrap_or(remaining)
                .min(remaining)
        } else if self.worker.is_some() || self.cue_worker.is_some() {
            Duration::from_millis(50)
        } else {
            Duration::from_secs(1)
        };
        c.set_timer(TICK, Deadline(Instant::now() + interval));
    }
    fn notice(&self, c: &mut Context<'_, ()>, text: String) {
        if let Some(t) = self.widget::<Transcript>(c, self.diagnostic)
            && !t.replace_notice(text)
        {
            return;
        }
        c.redraw();
    }
    fn picker(&mut self, c: &mut Context<'_, ()>) {
        if c.desktop.has_modal() {
            return;
        }
        if self.models.is_empty() {
            self.notice(c, "No discovered models. F5 refreshes discovery.".into());
            return;
        }
        let mut list = Panel::new(
            "Models · ↑/↓ · Enter selects · Esc closes",
            self.models.iter().map(models::Model::label).collect(),
        );
        if let Some(index) = self
            .models
            .iter()
            .position(|m| m.name == self.config.ollama.model)
        {
            list.select(index);
        }
        self.picker = Some(c.desktop.show_modal(
            ModelPicker {
                list,
                names: self.models.iter().map(|m| m.name.clone()).collect(),
            },
            terrabox::Size {
                width: 80,
                height: 16,
            },
        ));
        c.redraw();
    }
    fn activate(&mut self, c: &mut Context<'_, ()>, name: String) {
        // Dropping the worker cancels I/O and destroys its receiver, including queued stale text.
        self.worker = None;
        self.cue_worker = None;
        self.pending = None;
        self.response.clear();
        self.retry_at = None;
        self.config.ollama.model = name.clone();
        self.previous_model = Some(name.clone());
        self.ready = true;
        let saved = models::path()
            .ok_or_else(|| "No state directory available".to_string())
            .and_then(|p| models::save(&p, &self.config.ollama.endpoint, &name));
        self.notice(
            c,
            match saved {
                Ok(()) => format!("Selected {name}. Completed conversation history retained."),
                Err(e) => format!("Selected {name}; selection could not be saved: {e}"),
            },
        );
        self.status(c, "Connected");
        self.pet_event(c, PetEvent::Idle);
        self.schedule(c);
    }
    fn check(&mut self, c: &mut Context<'_, ()>) {
        self.worker = None;
        self.cue_worker = None;
        self.ready = false;
        self.retry_at = None;
        match Worker::start(self.config.ollama.clone(), None) {
            Ok(w) => {
                self.worker = Some(w);
                self.status(c, "Checking connection");
                self.pet_event(c, PetEvent::Idle);
                if let Some(t) = self.widget::<Terri>(c, self.terri) {
                    t.pet.traits.mood = crate::pet::Mood::Curious;
                }
            }
            Err(e) => self.fail(c, format!("Cannot start model worker: {e}")),
        }
        self.schedule(c);
    }
    fn fail(&mut self, c: &mut Context<'_, ()>, error: String) {
        self.worker = None;
        self.pending = None;
        self.response.clear();
        self.ready = false;
        if self.retries < 2
            && (error.contains("unavailable at")
                || error.contains("timeout")
                || error.contains("Ollama check failed"))
        {
            self.retries += 1;
            self.retry_at = Some(Instant::now() + Duration::from_secs(2 * u64::from(self.retries)));
        }
        self.notice(c, error);
        self.status(c, "Error · F5 retries");
        self.pet_event(c, PetEvent::Error);
    }
    fn send(&mut self, c: &mut Context<'_, ()>) {
        if self.worker.is_some() {
            return;
        }
        if !self.ready {
            self.check(c);
            return;
        }
        let text = self
            .widget::<Input>(c, self.input)
            .map(|i| i.editor.text.trim().to_owned())
            .unwrap_or_default();
        if text.is_empty() {
            self.pet_event(c, PetEvent::Error);
            c.redraw();
            return;
        }
        if let Some(i) = self.widget::<Input>(c, self.input) {
            i.editor = TextInput::new("");
        }
        // Drop complete oldest turns, preserving role ordering. Display and provider history are separate.
        while self.history.len() >= 64
            || self.history.iter().map(|m| m.content.len()).sum::<usize>() + text.len()
                > HISTORY_LIMIT / 2
        {
            if self.history.len() < 2 {
                break;
            }
            self.history.drain(..2);
        }
        self.cue_worker = None;
        self.pet_event(c, PetEvent::User(&text));
        let mut messages = vec![Message {
            role: Role::System,
            content: crate::cues::profile(&self.config.agent.name),
        }];
        messages.extend(self.history.clone());
        messages.push(Message {
            role: Role::User,
            content: text.clone(),
        });
        if let Some(t) = self.transcript(c) {
            t.add("You", text.clone());
            t.add(&self.config.agent.name, String::new());
        }
        self.pending = Some(text);
        self.response.clear();
        match Worker::start(self.config.ollama.clone(), Some(messages)) {
            Ok(w) => {
                self.worker = Some(w);
                self.status(c, "Loading model / thinking · Esc cancels");
                self.pet_event(c, PetEvent::Thinking);
            }
            Err(e) => self.fail(c, e.to_string()),
        }
        self.schedule(c);
    }
    fn tick(&mut self, c: &mut Context<'_, ()>) {
        for _ in 0..32 {
            let update = self.worker.as_mut().map(|w| w.receiver.try_recv());
            match update {
                Some(Ok(Update::Discovered(models))) => {
                    self.worker = None;
                    self.models = models;
                    self.retries = 0;
                    if let Some(model) = models::select(
                        &self.models,
                        &self.configured_model,
                        self.previous_model.as_deref(),
                    ) {
                        let name = model.name.clone();
                        let missing = self.configured_model != "auto"
                            && models::configured(&self.models, &self.configured_model).is_none();
                        self.config.ollama.model = name.clone();
                        self.previous_model = Some(name.clone());
                        self.ready = true;
                        self.notice(c, if missing { format!("Configured model '{}' unavailable. Using {name}; F6 shows installed alternatives. Configuration preserved.", self.configured_model) } else { format!("Using {name}. F6 changes models.") });
                        self.status(c, "Connected");
                        self.pet_event(c, PetEvent::Idle);
                        if let Some(t) = self.widget::<Terri>(c, self.terri) {
                            t.pet.traits.mood = if missing {
                                crate::pet::Mood::Confused
                            } else {
                                crate::pet::Mood::Happy
                            };
                        }
                    } else {
                        self.fail(c, if self.models.is_empty() { "Ollama reachable but no models installed. Install a chat model using Ollama, then F5." } else { "No suitable chat models found. F6 lists installed models; F5 refreshes discovery." }.into());
                    }
                }
                Some(Ok(Update::Text(s))) => {
                    self.response.push_str(&s);
                    if let Some(t) = self.transcript(c) {
                        t.append(&s);
                    }
                    self.pet_event(c, PetEvent::Speaking);
                    self.status(c, "Responding · Esc cancels");
                }
                Some(Ok(Update::Done)) => {
                    self.worker = None;
                    self.pet_event(c, PetEvent::Done(&self.response));
                    if self.config.ollama.emotion_cues {
                        let excerpt = format!(
                            "User: {}\nAssistant: {}",
                            self.pending
                                .as_deref()
                                .unwrap_or_default()
                                .chars()
                                .take(1000)
                                .collect::<String>(),
                            self.response.chars().take(2000).collect::<String>()
                        );
                        self.cue_worker = Worker::cue(self.config.ollama.clone(), excerpt).ok();
                    }
                    if let Some(user) = self.pending.take() {
                        self.history.push(Message {
                            role: Role::User,
                            content: user,
                        });
                        self.history.push(Message {
                            role: Role::Assistant,
                            content: std::mem::take(&mut self.response),
                        });
                    }
                    self.status(c, "Connected");
                }
                Some(Ok(Update::Error(e))) => self.fail(c, e),
                Some(Ok(Update::Cue(_))) => {}
                Some(Err(tokio::sync::mpsc::error::TryRecvError::Disconnected)) => {
                    self.fail(c, "Model worker stopped unexpectedly".into())
                }
                _ => break,
            }
        }
        match self.cue_worker.as_mut().map(|w| w.receiver.try_recv()) {
            Some(Ok(update)) => {
                if let Update::Cue(cue) = update {
                    self.pet_event(c, PetEvent::Cue(cue));
                    c.redraw();
                }
                self.cue_worker = None;
            }
            Some(Err(tokio::sync::mpsc::error::TryRecvError::Disconnected)) => {
                self.cue_worker = None
            }
            _ => {}
        }
        let now = Instant::now();
        if self.worker.is_none() && self.retry_at.is_some_and(|at| now >= at) {
            self.check(c);
        }
        let elapsed = now.saturating_duration_since(self.last_tick);
        self.last_tick = now;
        if let Some(t) = self.widget::<Terri>(c, self.terri) {
            t.advance(elapsed);
        }
        if self.clock.poll(now).is_some()
            && self.config.interface.animation
            && self.config.interface.show_terri
            && c.size().width >= 52
            && c.size().height >= 14
            && self
                .widget::<Terri>(c, self.terri)
                .is_some_and(|t| t.fault.is_none())
        {
            c.redraw();
        }
        self.schedule(c);
    }
}
impl App for Chat {
    type Message = ();
    fn init(&mut self, c: &mut Context<'_, ()>) -> io::Result<()> {
        if self.config.interface.palette == "auto" {
            let caps = c.capabilities();
            let palette = if caps.unicode.support == terrabox::Support::Unsupported {
                crate::animation::Palette::Ascii
            } else if caps.truecolor.support == terrabox::Support::Unsupported {
                crate::animation::Palette::Indexed
            } else {
                crate::animation::Palette::TrueColor
            };
            if let Some(t) = self.widget::<Terri>(c, self.terri) {
                t.palette = palette;
            }
        }
        self.layout(c);
        self.check(c);
        Ok(())
    }
    fn event(&mut self, e: AppEvent<()>, c: &mut Context<'_, ()>) -> io::Result<()> {
        match e {
            AppEvent::Timer(TICK) => self.tick(c),
            AppEvent::Command(SELECT_MODEL) => {
                let name = self
                    .picker
                    .and_then(|id| c.desktop.widget_mut::<ModelPicker>(id))
                    .and_then(|p| p.list.selected().and_then(|i| p.names.get(i)).cloned());
                c.desktop.close_modal();
                self.picker = None;
                if let Some(name) = name {
                    self.activate(c, name);
                }
            }
            AppEvent::Command(QUIT) => c.quit(),
            AppEvent::Command(SEND) => self.send(c),
            AppEvent::Command(EDIT) => {
                self.cue_worker = None;
                self.pet_event(c, PetEvent::Typing);
            }
            AppEvent::Terminal(Event::Resize(_)) => {
                self.layout(c);
                self.schedule(c);
            }
            AppEvent::Terminal(Event::Key(k)) => {
                if k.modifiers.contains(Modifiers::CTRL)
                    && matches!(k.key, Key::Char('c' | 'C' | 'q' | 'Q'))
                {
                    c.quit();
                } else {
                    match k.key {
                        Key::Escape if self.worker.is_some() || self.retry_at.is_some() => {
                            self.worker = None;
                            self.pending = None;
                            self.response.clear();
                            self.retry_at = None;
                            self.notice(c, "Cancelled. Incomplete turn excluded from model history. Your draft is preserved.".into());
                            self.status(
                                c,
                                if self.ready {
                                    "Connected"
                                } else {
                                    "Disconnected · F5 retries"
                                },
                            );
                            self.pet_event(c, PetEvent::Error);
                            self.schedule(c);
                        }
                        Key::PageUp => {
                            if let Some(t) = self.transcript(c) {
                                t.scroll = t.scroll.saturating_add(8);
                            }
                            c.redraw();
                        }
                        Key::PageDown => {
                            if let Some(t) = self.transcript(c) {
                                t.scroll = t.scroll.saturating_sub(8);
                            }
                            c.redraw();
                        }
                        Key::Function(2) => {
                            self.panel = self.panel.saturating_sub(2).max(20);
                            self.layout(c);
                        }
                        Key::Function(3) => {
                            self.panel = (self.panel + 2).min(48);
                            self.layout(c);
                        }
                        Key::Function(5) => {
                            self.pending = None;
                            self.response.clear();
                            self.retries = 0;
                            self.check(c);
                        }
                        Key::Function(6) => self.picker(c),
                        Key::Function(9) => {
                            self.config.interface.animation = !self.config.interface.animation;
                            if let Some(t) = self.widget::<Terri>(c, self.terri) {
                                t.animate = self.config.interface.animation;
                            }
                            self.schedule(c);
                            c.redraw();
                        }
                        Key::Function(7) => {
                            let value = if self.config.interface.animation_intensity > 0.5 {
                                0.35
                            } else {
                                1.0
                            };
                            self.config.interface.animation_intensity = value;
                            if let Some(t) = self.widget::<Terri>(c, self.terri) {
                                t.intensity = value;
                            }
                            c.redraw();
                        }
                        Key::Function(8) => {
                            self.config.interface.show_terri = !self.config.interface.show_terri;
                            self.layout(c);
                            self.schedule(c);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
    fn shutdown(&mut self, c: &mut Context<'_, ()>) -> io::Result<()> {
        self.worker = None;
        self.cue_worker = None;
        if let Some(path) = &self.state_path
            && let Some(t) = self.widget::<Terri>(c, self.terri)
        {
            persistence::save(path, &t.pet.snapshot()).map_err(io::Error::other)?;
        }
        Ok(())
    }
}
pub fn run(config: Config) -> io::Result<()> {
    let now = Instant::now();
    let mut desktop = Desktop::default();
    desktop.tab_navigation = false;
    let (bg, fg, bar) = if config.interface.theme == "light" {
        (
            Color::Rgb(237, 243, 247),
            Color::Rgb(28, 46, 63),
            Color::Rgb(213, 228, 236),
        )
    } else {
        (
            Color::Rgb(13, 20, 32),
            Color::Rgb(207, 222, 234),
            Color::Rgb(24, 38, 54),
        )
    };
    let surface = Style::new(fg, bg);
    let accent = Style::new(Color::Rgb(53, 181, 170), bg);
    desktop.theme = Theme {
        surface,
        border: surface,
        focused: accent,
        selection: Style::new(fg, bar),
        bar: Style::new(fg, bar),
        dialog: surface,
    };
    let mut controls = Group::default();
    let header = controls.add(Label(String::new()), false);
    let diagnostic = controls.add(Transcript::new(), false);
    let mut state_path = if config.pet.persist {
        persistence::default_path()
    } else {
        None
    };
    let mut warning = None;
    let mut snapshot = match state_path
        .as_deref()
        .map(|p| persistence::load(p, config.pet.seed))
    {
        Some(Ok(s)) => s,
        Some(Err(e)) => {
            warning = Some(format!(
                "Terri state could not be loaded: {e}. Existing file preserved; this session will not save."
            ));
            state_path = None;
            Snapshot::new(config.pet.seed)
        }
        None => Snapshot::new(config.pet.seed),
    };
    snapshot.sessions = snapshot.sessions.saturating_add(1);
    let mut character = Terri::new(config.pet.clone(), snapshot);
    character.animate = config.interface.animation;
    character.intensity = config.interface.animation_intensity;
    character.palette = match config.interface.palette.as_str() {
        "indexed" => crate::animation::Palette::Indexed,
        "ascii" => crate::animation::Palette::Ascii,
        _ => crate::animation::Palette::TrueColor,
    };
    let terri = controls.add(character, false);
    let mut transcript_widget = Transcript::new();
    transcript_widget.add("Welcome",format!("I'm {}. Let's make something useful.\nChat stays in memory and is sent only to your configured Ollama endpoint.\nType below; Enter sends, Escape cancels, Page Up/Down scrolls.",config.agent.name));
    if let Some(warning) = warning {
        transcript_widget.add("State notice", warning);
    }
    let transcript = controls.add(transcript_widget, false);
    let input = controls.add(
        Input {
            editor: TextInput::new(""),
        },
        true,
    );
    let group = desktop.add_window(controls);
    desktop.layout = Layout::Window(group);
    let clock = FrameClock::new(
        Duration::from_secs_f64(1.0 / f64::from(config.interface.animation_fps)),
        now,
    )
    .ok_or_else(|| io::Error::other("Invalid frame interval"))?;
    Application::new(
        Chat {
            configured_model: config.ollama.model.clone(),
            previous_model: models::path()
                .and_then(|p| models::load(&p, &config.ollama.endpoint).ok().flatten()),
            models: vec![],
            picker: None,
            diagnostic,
            retry_at: None,
            retries: 0,
            group,
            config,
            input,
            transcript,
            terri,
            header,
            worker: None,
            history: vec![],
            pending: None,
            response: String::new(),
            ready: false,
            last_tick: now,
            cue_worker: None,
            state_path,
            clock,
            panel: 42,
        },
        desktop,
    )
    .run()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn input_focus_returns_after_tiny_resize() {
        let mut desktop = Desktop::default();
        let mut group = Group::default();
        let label = group.add(Label("Terri".into()), false);
        let input = group.add(
            Input {
                editor: TextInput::new(""),
            },
            true,
        );
        group.layout = Layout::Split {
            axis: Axis::Vertical,
            children: vec![
                (Length::Flex(1), Layout::Window(label)),
                (Length::Fixed(1), Layout::Window(input)),
            ],
        };
        let root = desktop.add_window(group);
        desktop.layout = Layout::Window(root);
        for (width, height) in [(80, 24), (0, 0), (1, 1), (80, 24)] {
            desktop.render(terrabox::Size { width, height });
        }
        desktop.route(&Event::Key(terrabox::KeyEvent::new(
            Key::Char('x'),
            Modifiers::NONE,
        )));
        assert_eq!(
            desktop
                .widget_mut::<Group>(root)
                .unwrap()
                .widget_mut::<Input>(input)
                .unwrap()
                .editor
                .text,
            "x"
        );
    }
    #[test]
    fn input_limit_never_swallows_quit() {
        let mut input = Input {
            editor: TextInput::new("x".repeat(INPUT_LIMIT)),
        };
        let key = Event::Key(terrabox::KeyEvent::new(Key::Char('Q'), Modifiers::CTRL));
        assert!(!input.event(&key).consumed);
        input.event(&Event::Paste(terrabox::PasteEvent {
            bytes: vec![b'z'; 100],
            complete: true,
        }));
        assert_eq!(input.editor.text.len(), INPUT_LIMIT);
    }
    #[test]
    fn unicode_wrap_and_control_filter() {
        assert_eq!(wrap("hello world", 8), vec!["hello", "world"]);
        assert_eq!(wrap("a界e\u{301}", 3), vec!["a界", "e\u{301}"]);
        assert_eq!(clean("a\x1b\x07\nb"), "a\nb");
        assert!(wrap("x", 0).is_empty());
    }
    #[test]
    fn diagnostic_replaces_and_suppresses_duplicates() {
        let mut t = Transcript::new();
        assert!(t.replace_notice("offline".into()));
        assert!(!t.replace_notice("offline".into()));
        assert!(t.replace_notice("restored".into()));
        assert_eq!(
            t.messages,
            vec![("Ollama status".into(), "restored".into())]
        );
    }
    #[test]
    fn transcript_is_bounded() {
        let mut t = Transcript::new();
        for _ in 0..200 {
            t.add("You", "hello".into());
        }
        assert_eq!(t.messages.len(), 128);
    }
}
