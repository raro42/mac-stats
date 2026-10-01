// Evita la consola extra en Windows en release; en iOS no se usa este binario.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    ios_stats_lib::run()
}
