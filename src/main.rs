use anyhow::{Result, anyhow};
use constitute_build::{build_fixture, build_status, default_now};
use std::env;

fn main() -> Result<()> {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() || args[0] == "help" || args[0] == "--help" {
        print_help();
        return Ok(());
    }

    match args.remove(0).as_str() {
        "fixture" => fixture_command(args),
        "run" => run_command(args),
        "status" => print_json(&build_status()?),
        command => Err(anyhow!("unsupported constitute-build command: {command}")),
    }
}

fn fixture_command(args: Vec<String>) -> Result<()> {
    match args.first().map(String::as_str) {
        Some("run") | None => print_json(&build_fixture(default_now(), "succeeded")?),
        Some(name) => Err(anyhow!("unsupported fixture: {name}")),
    }
}

fn run_command(args: Vec<String>) -> Result<()> {
    let mut state = "succeeded".to_string();
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        let value = iter
            .next()
            .ok_or_else(|| anyhow!("{flag} requires a value"))?;
        match flag.as_str() {
            "--state" => state = value,
            _ => return Err(anyhow!("unsupported run flag: {flag}")),
        }
    }
    print_json(&build_fixture(default_now(), &state)?)
}

fn print_json(value: &impl serde::Serialize) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

fn print_help() {
    println!(
        "constitute-build\n\nCommands:\n  fixture run\n  run --state succeeded|blocked\n  status"
    );
}
