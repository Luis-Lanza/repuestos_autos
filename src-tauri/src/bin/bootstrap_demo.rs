use std::path::PathBuf;

use repuestos_autos::application::catalog::{
    bootstrap_demo_catalog, BootstrapDemoError, BootstrapDemoOutcome,
};
use repuestos_autos::infrastructure::sqlite::{
    database_config, default_application_database_config, open_existing_database,
};

fn main() {
    if let Err(error) = run(std::env::args().skip(1)) {
        eprintln!("bootstrap-demo: {error}");
        std::process::exit(1);
    }
}

fn run(arguments: impl IntoIterator<Item = String>) -> Result<(), String> {
    let mut database_path = None;
    let mut arguments = arguments.into_iter();
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            "--database" => {
                let path = arguments
                    .next()
                    .ok_or_else(|| "--database requires a path".to_string())?;
                database_path = Some(PathBuf::from(path));
            }
            _ => return Err(format!("unknown argument `{argument}`; use --help")),
        }
    }

    let config = match database_path {
        Some(path) => database_config(path),
        None => default_application_database_config()?,
    };
    let mut connection = open_existing_database(&config)?;
    match bootstrap_demo_catalog(&mut connection) {
        Ok(BootstrapDemoOutcome::Loaded(summary)) => {
            println!(
                "Loaded demo catalog: {} categories added, {} products added, {} product reactivated.",
                summary.categories_added, summary.products_added, summary.products_reactivated
            );
            Ok(())
        }
        Ok(BootstrapDemoOutcome::AlreadyComplete(_)) => {
            println!("Demo catalog is already complete; no changes were made.");
            Ok(())
        }
        Err(error) => Err(match error {
            BootstrapDemoError::Refused => {
                "refused: database is not the recognized pristine bootstrap state".into()
            }
            BootstrapDemoError::InvalidPlan => "bootstrap demo plan is invalid".into(),
            BootstrapDemoError::Persistence => "persistence failed; no bootstrap changes committed".into(),
        }),
    }
}

fn print_help() {
    println!(
        r#"Usage: bootstrap-demo [--database <path>]

Bootstrap the deterministic automotive demo catalog only on the exact pristine
application bootstrap database. A complete recognized demo catalog is idempotent.

Options:
  --database <path>  Use an explicit SQLite database path
  -h, --help         Show this help

Without --database, the application data path is used (Windows:
%APPDATA%\com.repuestosautos.app\repuestos-autos.sqlite3)."#
    );
}
