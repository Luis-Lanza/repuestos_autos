#![cfg_attr(all(target_os = "windows", feature = "desktop", not(debug_assertions)), windows_subsystem = "windows")]

#[cfg(feature = "desktop")]
fn main() {
    if let Err(error) = repuestos_autos::run() {
        eprintln!("failed to run Repuestos Autos: {error}");
        std::process::exit(1);
    }
}

#[cfg(not(feature = "desktop"))]
fn main() {}
