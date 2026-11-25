#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if let Err(error) = sayt_desktop::run() {
        eprintln!("SAYT desktop shell failed to start: {error}");
        std::process::exit(1);
    }
}
