#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if let Err(error) = mochi_lib::run() {
        eprintln!("mochi failed: {error}");
        std::process::exit(1);
    }
}
