//! Full-colour terminal-cell references, sampled through the real TerraTUI Widget renderer.
//! Candidate generation is explicit and never overwrites the checked-in references.
use std::{collections::BTreeSet, fmt::Write, path::PathBuf, time::Duration};
use terrabot::{animation::default_trixie, motion::ClipId};
use terrabox::{Color, Rect, Size, Style, Surface};
use terratui::{Theme, Widget};
fn rgb(color: Color) -> (u8, u8, u8) {
    match color {
        Color::Rgb(r, g, b) => (r, g, b),
        other => panic!("unexpected {other:?}"),
    }
}
fn capture(id: ClipId, ms: u64) -> String {
    let mut trixie = default_trixie();
    trixie.preview_clip(id);
    let mut remaining = ms;
    while remaining > 0 {
        let step = remaining.min(20);
        trixie.advance(Duration::from_millis(step));
        remaining -= step;
    }
    let mut surface = Surface::new(Size {
        width: 80,
        height: 26,
    });
    let theme = Theme {
        surface: Style::new(Color::Rgb(207, 222, 234), Color::Rgb(13, 20, 32)),
        ..Theme::default()
    };
    surface.clear(theme.surface);
    trixie.draw(&mut surface.canvas(Rect::new(0, 0, 80, 26)), false, &theme);
    let mut pixels = Vec::new();
    // Exclude the changing textual status row; retain the entire character/habitat viewport.
    for y in 0..25 {
        for half in 0..2 {
            for x in 0..80 {
                let cell = surface.cell(x, y).unwrap();
                assert!(matches!(cell.scalar(), ' ' | '▀'));
                pixels.push(rgb(if cell.scalar() == '▀' && half == 0 {
                    cell.style.foreground
                } else {
                    cell.style.background
                }));
            }
        }
    }
    let palette: Vec<_> = pixels
        .iter()
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let alphabet = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    assert!(palette.len() <= alphabet.len());
    let mut text = format!("# {id:?} at {ms}ms; 80x50 logical pixels from 80x25 terminal cells\n");
    for (i, (r, g, b)) in palette.iter().enumerate() {
        write!(text, "{}={r:02x}{g:02x}{b:02x} ", alphabet[i] as char).unwrap();
    }
    text.push('\n');
    for row in pixels.chunks(80) {
        for pixel in row {
            text.push(alphabet[palette.binary_search(pixel).unwrap()] as char);
        }
        text.push('\n');
    }
    text
}
#[test]
fn reviewed_terminal_cell_references() {
    for id in [
        ClipId::Idle,
        ClipId::Curious,
        ClipId::Bored,
        ClipId::Think,
        ClipId::Talk,
        ClipId::Jump,
        ClipId::Sit,
        ClipId::Sleep,
        ClipId::Wake,
        ClipId::Happy,
        ClipId::Celebrate,
        ClipId::Play,
    ] {
        for ms in [0, 520, 1200, 2000, 3200, 4300] {
            let name = format!("{id:?}-{ms}.cells");
            let actual = capture(id, ms);
            if let Some(dir) = std::env::var_os("TERRABOT_REVIEW_REFERENCES") {
                let dir = PathBuf::from(dir);
                std::fs::create_dir_all(&dir).unwrap();
                std::fs::write(dir.join(name), actual).unwrap();
            } else {
                let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("tests/visual/v050")
                    .join(&name);
                let expected = std::fs::read_to_string(&path)
                    .unwrap_or_else(|_| panic!("missing reviewed reference {}", path.display()));
                assert_eq!(
                    actual, expected,
                    "visual regression {name}; inspect a candidate, do not blindly replace the reference"
                );
            }
        }
    }
}
