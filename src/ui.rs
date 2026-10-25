use crate::{
    config::Config,
    provider::{Message, Role, Update, Worker},
    terry::{State, Terry},
};
use std::{
    io,
    time::{Duration, Instant},
};
use terrabox::{Canvas, Color, Deadline, Event, FrameClock, Key, Modifiers, Rect, Style};
use terratui::{
    App, AppEvent, Application, Axis, CommandId, Context, Desktop, EventResult, Layout, Length,
    Theme, TimerId, Widget, WindowId,
    widgets::{Group, Label, TextInput},
};
use unicode_segmentation::UnicodeSegmentation;

const SEND: CommandId = CommandId(1);
const EDIT: CommandId = CommandId(2);
const INPUT_LIMIT: usize = 8192;
const HISTORY_LIMIT: usize = 512 * 1024;
const TICK: TimerId = TimerId(1);
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
    terry: WindowId,
    header: WindowId,
    worker: Option<Worker>,
    history: Vec<Message>,
    pending: Option<String>,
    response: String,
    ready: bool,
    started: Instant,
    clock: FrameClock,
    panel: u16,
}
impl Chat {
    fn widget<'a, T: Widget>(&self, c: &'a mut Context<'_, ()>, id: WindowId) -> Option<&'a mut T> {
        c.desktop
            .widget_mut::<Group>(self.group)?
            .widget_mut::<T>(id)
    }
    fn set_state(&self, c: &mut Context<'_, ()>, state: State) {
        if let Some(t) = self.widget::<Terry>(c, self.terry) {
            t.set(state, Instant::now());
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
            " {} · Tools disabled | Enter Send · Esc Cancel · PgUp/Dn Scroll · F2/F3 Panel · F5 Retry · Ctrl+Q Quit",
            self.config.ollama.model
        );
        c.redraw();
    }
    fn layout(&self, c: &mut Context<'_, ()>) {
        let size = c.size();
        let show = self.config.interface.show_terry && size.width >= 52 && size.height >= 14;
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
                        Layout::Window(self.terry)
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
            && self.config.interface.show_terry
            && size.width >= 52
            && size.height >= 14;
        let happy = self
            .widget::<Terry>(c, self.terry)
            .is_some_and(|t| t.state == State::Happy);
        if animated {
            if let Some(deadline) = self.clock.deadline() {
                c.set_timer(TICK, deadline);
            }
        } else if self.worker.is_some() || happy {
            c.set_timer(TICK, Deadline(Instant::now() + Duration::from_millis(50)));
        } else {
            c.cancel_timer(TICK);
        }
    }
    fn check(&mut self, c: &mut Context<'_, ()>) {
        self.worker = None;
        self.ready = false;
        match Worker::start(self.config.ollama.clone(), None) {
            Ok(w) => {
                self.worker = Some(w);
                self.status(c, "Checking connection");
                self.set_state(c, State::Thinking);
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
        if let Some(t) = self.transcript(c) {
            t.add("Connection / request error", error);
        }
        self.status(c, "Error · F5 retries");
        self.set_state(c, State::Error);
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
            self.set_state(c, State::Confused);
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
        let mut messages = self.history.clone();
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
                self.status(c, "Thinking · Esc cancels");
                self.set_state(c, State::Thinking);
            }
            Err(e) => self.fail(c, e.to_string()),
        }
        self.schedule(c);
    }
    fn tick(&mut self, c: &mut Context<'_, ()>) {
        for _ in 0..32 {
            let update = self.worker.as_mut().map(|w| w.receiver.try_recv());
            match update {
                Some(Ok(Update::Ready)) => {
                    self.worker = None;
                    self.ready = true;
                    self.status(c, "Connected");
                    self.set_state(c, State::Idle);
                }
                Some(Ok(Update::Text(s))) => {
                    self.response.push_str(&s);
                    if let Some(t) = self.transcript(c) {
                        t.append(&s);
                    }
                    self.set_state(c, State::Responding);
                    self.status(c, "Responding · Esc cancels");
                }
                Some(Ok(Update::Done)) => {
                    self.worker = None;
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
                    self.set_state(c, State::Happy);
                }
                Some(Ok(Update::Error(e))) => self.fail(c, e),
                Some(Err(tokio::sync::mpsc::error::TryRecvError::Disconnected)) => {
                    self.fail(c, "Model worker stopped unexpectedly".into())
                }
                _ => break,
            }
        }
        let now = Instant::now();
        let typing = self
            .widget::<Input>(c, self.input)
            .is_some_and(|i| !i.editor.text.is_empty());
        if let Some(t) = self.widget::<Terry>(c, self.terry) {
            let before = t.state;
            t.settle(now, typing);
            if before != t.state {
                c.redraw();
            }
        }
        if self.clock.poll(now).is_some()
            && self.config.interface.animation
            && self.config.interface.show_terry
            && c.size().width >= 52
            && c.size().height >= 14
        {
            if let Some(t) = self.widget::<Terry>(c, self.terry) {
                t.elapsed = now - self.started;
            }
            c.redraw();
        }
        self.schedule(c);
    }
}
impl App for Chat {
    type Message = ();
    fn init(&mut self, c: &mut Context<'_, ()>) -> io::Result<()> {
        self.layout(c);
        self.check(c);
        Ok(())
    }
    fn event(&mut self, e: AppEvent<()>, c: &mut Context<'_, ()>) -> io::Result<()> {
        match e {
            AppEvent::Timer(TICK) => self.tick(c),
            AppEvent::Command(SEND) => self.send(c),
            AppEvent::Command(EDIT) => {
                if self.worker.is_none() && self.ready {
                    let typing = self
                        .widget::<Input>(c, self.input)
                        .is_some_and(|i| !i.editor.text.is_empty());
                    self.set_state(
                        c,
                        if typing {
                            State::Listening
                        } else {
                            State::Idle
                        },
                    );
                }
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
                        Key::Escape if self.worker.is_some() => {
                            self.worker = None;
                            self.pending = None;
                            self.response.clear();
                            if let Some(t) = self.transcript(c) {
                                t.add("Cancelled","Incomplete turn excluded from model history. Your draft is preserved.".into());
                            }
                            self.status(
                                c,
                                if self.ready {
                                    "Connected"
                                } else {
                                    "Disconnected · F5 retries"
                                },
                            );
                            self.set_state(c, State::Confused);
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
                        Key::Function(5) if self.worker.is_none() => self.check(c),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
    fn shutdown(&mut self, _: &mut Context<'_, ()>) -> io::Result<()> {
        self.worker = None;
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
    let terry = controls.add(Terry::new(now), false);
    let mut transcript_widget = Transcript::new();
    transcript_widget.add("Welcome",format!("I'm {}. Let's make something useful.\nChat stays in memory and is sent only to your configured Ollama endpoint.\nType below; Enter sends, Escape cancels, Page Up/Down scrolls.",config.agent.name));
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
            group,
            config,
            input,
            transcript,
            terry,
            header,
            worker: None,
            history: vec![],
            pending: None,
            response: String::new(),
            ready: false,
            started: now,
            clock,
            panel: 38,
        },
        desktop,
    )
    .run()
}
#[cfg(test)]
mod tests {
    use super::*;
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
    fn transcript_is_bounded() {
        let mut t = Transcript::new();
        for _ in 0..200 {
            t.add("You", "hello".into());
        }
        assert_eq!(t.messages.len(), 128);
    }
    #[test]
    fn tiny_layouts_and_all_expressions_render() {
        let now = Instant::now();
        let mut desktop = Desktop::default();
        let id = desktop.add_window(Terry::new(now));
        desktop.layout = Layout::Window(id);
        for state in [
            State::Idle,
            State::Listening,
            State::Thinking,
            State::Responding,
            State::Happy,
            State::Confused,
            State::Error,
            State::SecurityBlocked,
        ] {
            desktop.widget_mut::<Terry>(id).unwrap().set(state, now);
            for (width, height) in [(0, 0), (1, 1), (18, 12), (38, 22)] {
                let surface = desktop.render(terrabox::Size { width, height });
                assert_eq!(surface.size().width, width);
            }
        }
    }
}
