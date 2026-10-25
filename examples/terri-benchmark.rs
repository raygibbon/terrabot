//! CPU-only measurements; the PTY benchmark measures actual paced terminal output.
use std::{
    hint::black_box,
    time::{Duration, Instant},
};
use terrabot::{animation::default_terri, pet::Activity};
use terrabox::{Rect, Size, Surface};
use terratui::{Theme, Widget};
fn main() {
    for fps in [20, 30, 60] {
        let mut terri = default_terri();
        let mut surface = Surface::new(Size {
            width: 80,
            height: 30,
        });
        let mut samples = Vec::with_capacity(fps * 20);
        let step = Duration::from_secs_f64(1.0 / fps as f64);
        for frame in 0..fps * 20 {
            if frame % fps == 0 {
                terri.pet.demo_activity(Activity::ALL[frame / fps % 10]);
            }
            let start = Instant::now();
            terri.advance(step);
            terri.draw(
                &mut surface.canvas(Rect::new(0, 0, 80, 30)),
                false,
                &Theme::default(),
            );
            black_box(&surface);
            samples.push(start.elapsed().as_nanos());
        }
        samples.sort_unstable();
        let mean = samples.iter().sum::<u128>() as f64 / samples.len() as f64 / 1000.0;
        println!(
            "target_fps={fps} composition_mean_us={mean:.2} p99_us={:.2} simulation_ticks={}",
            samples[samples.len() * 99 / 100] as f64 / 1000.0,
            terri.pet.traits.active_ticks
        );
    }
}
