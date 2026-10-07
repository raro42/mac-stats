// Prevents an extra console window on Windows in release; iOS does not use this binary.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    ios_stats_lib::run()
}
