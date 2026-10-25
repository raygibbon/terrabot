mod config;
mod provider;
mod terry;
mod ui;

fn main() -> std::process::ExitCode {
    match start() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("terrabot: {e}");
            std::process::ExitCode::FAILURE
        }
    }
}
fn start() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let mut path = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                println!(
                    "Terrabot v0.1\nUsage: terrabot [--config PATH]\nDefault: ~/.config/terrabot/config.toml\nEnter send · Esc cancel · PgUp/PgDn scroll · F2/F3 resize Terry · F5 reconnect · Ctrl+C/Q quit"
                );
                return Ok(());
            }
            "--version" | "-V" => {
                println!("terrabot {}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            "--config" => path = Some(args.next().ok_or("--config requires a path")?.into()),
            _ => return Err(format!("Unknown argument: {arg}; use --help").into()),
        }
    }
    let config = config::Config::load(path).map_err(std::io::Error::other)?;
    ui::run(config)?;
    Ok(())
}
