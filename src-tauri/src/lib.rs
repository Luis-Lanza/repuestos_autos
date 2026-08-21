#[cfg(feature = "desktop")]
use tauri::Manager;

pub mod application;
pub mod commands;
pub mod domain;
pub mod infrastructure;

#[cfg(feature = "desktop")]
struct AppState(std::sync::Mutex<rusqlite::Connection>);

#[cfg(feature = "desktop")]
pub fn run() -> Result<(), tauri::Error> {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_directory = app.path().app_data_dir()?;
            let database_config =
                infrastructure::sqlite::production_database_config(app_data_directory);
            let connection = infrastructure::sqlite::open_database(&database_config)
                .map_err(|error| std::io::Error::other(error.to_string()))?;
            app.manage(AppState(std::sync::Mutex::new(connection)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            search_products_command,
            confirm_sale_command
        ])
        .run(tauri::generate_context!())
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn search_products_command(
    state: tauri::State<AppState>,
    request: commands::catalog::SearchProductsRequest,
) -> Result<Vec<commands::catalog::ProductSearchResult>, String> {
    commands::catalog::search_products(&state.0.lock().map_err(|_| "persistence_failure")?, request)
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn confirm_sale_command(
    state: tauri::State<AppState>,
    request: commands::confirm_sale::ConfirmSaleRequest,
) -> Result<commands::confirm_sale::ConfirmSaleResponse, String> {
    commands::confirm_sale::confirm_sale(
        &mut state.0.lock().map_err(|_| "persistence_failure")?,
        request,
    )
}

pub mod catalog {
    pub use crate::application::catalog::{search_active_products, ProductSearchResult};
    pub use crate::infrastructure::sqlite::open_seeded_catalog;
}
