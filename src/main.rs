use anyhow::{Result, anyhow};
use constitute_build::{
    append_build_run, build_fixture, build_state_status, build_status, default_build_output_plan,
    default_build_run_request, default_build_state, default_now, load_build_state,
    save_build_state,
};
use std::env;

fn main() -> Result<()> {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() || args[0] == "help" || args[0] == "--help" {
        print_help();
        return Ok(());
    }

    match args.remove(0).as_str() {
        "fixture" => fixture_command(args),
        "init" => init_command(args),
        "run" => run_command(args),
        "status" => status_command(args),
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
    let mut state_file = String::new();
    let mut iter = args.into_iter();
    while let Some(flag) = iter.next() {
        let value = iter
            .next()
            .ok_or_else(|| anyhow!("{flag} requires a value"))?;
        match flag.as_str() {
            "--state" => state = value,
            "--state-file" | "--store" => state_file = value,
            _ => return Err(anyhow!("unsupported run flag: {flag}")),
        }
    }
    if state_file.is_empty() {
        return print_json(&build_fixture(default_now(), &state)?);
    }
    let now = default_now();
    let mut build_state = load_build_state(&state_file, now)?;
    let fixture = build_fixture(now, "succeeded")?;
    let mut request = default_build_run_request(now);
    if state == "blocked" {
        request.resource_available = false;
    }
    let artifact = fixture
        .artifact
        .as_ref()
        .ok_or_else(|| anyhow!("fixture missing artifact plan"))?;
    let output = default_build_output_plan(artifact, &fixture.proof);
    let outcome = append_build_run(&mut build_state, request, output)?;
    save_build_state(&state_file, &build_state)?;
    print_json(&outcome)
}

fn init_command(args: Vec<String>) -> Result<()> {
    let state_file = option_value(&args, "--state-file")
        .or_else(|| option_value(&args, "--store"))
        .ok_or_else(|| anyhow!("init requires --state-file or --store"))?;
    let state = default_build_state(default_now())?;
    save_build_state(&state_file, &state)?;
    print_json(&build_state_status(&state)?)
}

fn status_command(args: Vec<String>) -> Result<()> {
    let state_file = option_value(&args, "--state-file").or_else(|| option_value(&args, "--store"));
    if let Some(state_file) = state_file {
        let state = load_build_state(&state_file, default_now())?;
        print_json(&build_state_status(&state)?)
    } else {
        print_json(&build_status()?)
    }
}

fn option_value(args: &[String], name: &str) -> Option<String> {
    args.windows(2)
        .find(|pair| pair[0] == name)
        .map(|pair| pair[1].clone())
}

fn print_json(value: &impl serde::Serialize) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

fn print_help() {
    println!(
        "constitute-build\n\nCommands:\n  fixture run\n  init --state-file target/build-state.json\n  run --state succeeded|blocked [--state-file target/build-state.json]\n  status [--state-file target/build-state.json]"
    );
}
