//! Reproducible accelerated life soak; no terminal, clock, files or model requests.
use std::time::Duration;
use terrabot::{
    config::PetConfig,
    pet::{Activity, Pet, Snapshot},
};
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let seconds: u64 = args
        .get(1)
        .map(|s| s.parse().expect("seconds"))
        .unwrap_or(86400);
    let seed: u64 = args.get(2).map(|s| s.parse().expect("seed")).unwrap_or(7);
    assert!((1..=604800).contains(&seconds), "duration 1–604800 seconds");
    let mut pet = Pet::new(
        PetConfig {
            seed,
            ..PetConfig::default()
        },
        Snapshot::new(seed),
    );
    let mut ticks = [0u64; terrabot::life::COUNT];
    let mut entries = [0u64; terrabot::life::COUNT];
    let mut previous = pet.activity;
    let mut longest = 0;
    let mut streak = 0;
    for second in 0..seconds {
        pet.set_hour(Some(((second / 3600 + 12) % 24) as u8));
        for _ in 0..20 {
            pet.advance(Duration::from_millis(50));
            pet.snapshot().validate().expect("bounded finite state");
            assert!((0.0..=1.0).contains(&pet.x));
            assert!((0.0..=1.0).contains(&pet.orb.x));
            assert!(pet.orb.height.is_finite() && pet.orb.height <= 40.0);
            ticks[pet.activity as usize] += 1;
            if pet.activity != previous {
                entries[pet.activity as usize] += 1;
                longest = longest.max(streak);
                streak = 0;
                previous = pet.activity;
            }
            streak += 1;
        }
    }
    let activities: Vec<_> = Activity::ALL.iter().enumerate().map(|(i,a)| serde_json::json!({"activity":format!("{a:?}"),"seconds":ticks[i] as f64/20.0,"entries":entries[i]})).collect();
    println!(
        "{}",
        serde_json::json!({"seconds":seconds,"seed":seed,"longest_activity_seconds":longest.max(streak) as f64/20.0,"activities":activities,"state":pet.snapshot()})
    );
}
