//! `ewe-companion`: the companion's CLI and agent hook relay.
//!
//! Every verb prints one JSON object and exits 0 (rule 5). `hook` speaks
//! its host's dialect instead: Claude Code's hook JSON, or nothing at all.

mod client;
mod hook;
mod statusline;
mod verbs;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("hook") => return hook::run(&args[1..]),
        Some("statusline") => return statusline::run(),
        _ => {}
    }
    println!("{}", verbs::run(&args));
}
