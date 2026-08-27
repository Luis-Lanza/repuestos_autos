#[cfg(feature = "desktop")]
use tauri::Manager;

pub mod application;
pub mod commands;
pub mod domain;
pub mod infrastructure;

#[cfg(feature = "desktop")]
struct AppState(std::sync::Mutex<rusqlite::Connection>);

#[cfg(feature = "desktop")]
fn command_builder<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
    builder.invoke_handler(tauri::generate_handler![
        search_products_command,
        confirm_sale_command,
        confirm_stock_entry_command,
        confirm_physical_count_command,
        list_inventory_alerts_command,
        list_categories_command,
        create_category_command,
        create_product_command
    ])
}

#[cfg(feature = "desktop")]
pub fn run() -> Result<(), tauri::Error> {
    command_builder(tauri::Builder::default())
        .setup(|app: &mut tauri::App<tauri::Wry>| {
            let app_data_directory = app.path().app_data_dir()?;
            let database_config =
                infrastructure::sqlite::production_database_config(app_data_directory);
            let connection = infrastructure::sqlite::open_database(&database_config)
                .map_err(|error| std::io::Error::other(error.to_string()))?;
            app.manage(AppState(std::sync::Mutex::new(connection)));
            Ok(())
        })
        .run(tauri::generate_context!())
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn search_products_command(
    state: tauri::State<AppState>,
    request: commands::catalog::SearchProductsRequest,
) -> Result<Vec<commands::catalog::ProductSearchResult>, String> {
    let connection = state.0.lock().map_err(|_| "persistence_failure")?;
    commands::catalog::search_products(&connection, request)
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn confirm_sale_command(
    state: tauri::State<AppState>,
    request: commands::confirm_sale::ConfirmSaleRequest,
) -> Result<commands::confirm_sale::ConfirmSaleResponse, String> {
    let mut connection = state.0.lock().map_err(|_| "persistence_failure")?;
    commands::confirm_sale::confirm_sale(&mut connection, request)
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn confirm_stock_entry_command(
    state: tauri::State<AppState>,
    request: commands::inventory::StockEntryRequest,
) -> Result<commands::inventory::InventoryCommandResponse, String> {
    let mut connection = state.0.lock().map_err(|_| "persistence_failure")?;
    commands::inventory::confirm_stock_entry_command(&mut connection, request)
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn confirm_physical_count_command(
    state: tauri::State<AppState>,
    request: commands::inventory::PhysicalCountRequest,
) -> Result<commands::inventory::InventoryCommandResponse, String> {
    let mut connection = state.0.lock().map_err(|_| "persistence_failure")?;
    commands::inventory::confirm_physical_count_command(&mut connection, request)
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn list_inventory_alerts_command(
    state: tauri::State<AppState>,
) -> Result<commands::inventory::InventoryCommandResponse, String> {
    let mut connection = state.0.lock().map_err(|_| "persistence_failure")?;
    commands::inventory::list_inventory_alerts_command(&mut connection)
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn list_categories_command(
    state: tauri::State<AppState>,
) -> Result<commands::onboarding::ListCategoriesResponse, String> {
    let connection = state.0.lock().map_err(|_| "persistence_failure")?;
    commands::onboarding::list_categories(&connection)
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn create_category_command(
    state: tauri::State<AppState>,
    request: application::catalog::CreateCategoryInput,
) -> Result<commands::onboarding::CreateCategoryResponse, String> {
    let mut connection = state.0.lock().map_err(|_| "persistence_failure")?;
    commands::onboarding::create_category(&mut connection, request)
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn create_product_command(
    state: tauri::State<AppState>,
    request: application::catalog::CreateProductInput,
) -> Result<commands::onboarding::CreateProductResponse, String> {
    let mut connection = state.0.lock().map_err(|_| "persistence_failure")?;
    commands::onboarding::create_product(&mut connection, request)
}

#[cfg(all(test, feature = "desktop"))]
mod command_surface_tests {
    use super::*;
    use tauri::{
        ipc::CallbackFn,
        test::{get_ipc_response, mock_builder, mock_context, noop_assets, INVOKE_KEY},
        webview::InvokeRequest,
        WebviewWindowBuilder,
    };

    #[derive(Debug, PartialEq, Eq)]
    struct PersistenceSnapshot {
        stock: Vec<(i64, i64)>,
        sales: i64,
        payments: i64,
        sale_lines: i64,
        movements: i64,
    }

    fn snapshot(connection: &rusqlite::Connection) -> PersistenceSnapshot {
        let count = |table| {
            connection
                .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                    row.get(0)
                })
                .unwrap()
        };
        let stock = connection
            .prepare("SELECT product_id, quantity FROM stock_balances ORDER BY product_id")
            .unwrap()
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .unwrap()
            .collect::<Result<Vec<(i64, i64)>, _>>()
            .unwrap();
        PersistenceSnapshot {
            stock,
            sales: count("sales"),
            payments: count("sale_payments"),
            sale_lines: count("sale_lines"),
            movements: count("inventory_movements"),
        }
    }

    fn request(command: &str) -> InvokeRequest {
        InvokeRequest {
            cmd: command.into(),
            callback: CallbackFn(0),
            error: CallbackFn(1),
            url: "tauri://localhost".parse().unwrap(),
            body: Default::default(),
            headers: Default::default(),
            invoke_key: INVOKE_KEY.to_owned(),
        }
    }

    fn test_window() -> (
        tauri::App<tauri::test::MockRuntime>,
        tauri::WebviewWindow<tauri::test::MockRuntime>,
    ) {
        let app = command_builder(mock_builder())
            .manage(AppState(std::sync::Mutex::new(
                infrastructure::sqlite::open_seeded_catalog().unwrap(),
            )))
            .build(mock_context(noop_assets()))
            .unwrap();
        let window = WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();
        (app, window)
    }

    #[test]
    fn rejects_excluded_onboarding_operations_without_persistence_mutation() {
        let (app, window) = test_window();
        let before = snapshot(&app.state::<AppState>().0.lock().unwrap());

        for command in [
            "set_cart_line_price_command",
            "create_return_command",
            "cancel_sale_command",
            "create_supplier_command",
            "record_supplier_cost_command",
            "update_product_command",
            "inventory_report_command",
            "backup_database_command",
            "restore_database_command",
            "create_role_command",
            "sync_cloud_command",
            "transfer_stock_command",
        ] {
            assert!(
                get_ipc_response(&window, request(command)).is_err(),
                "{command}"
            );
        }

        assert_eq!(snapshot(&app.state::<AppState>().0.lock().unwrap()), before);
    }

    #[test]
    fn rejecting_draft_removal_and_discard_leaves_persistence_unchanged() {
        let (app, window) = test_window();
        let before = snapshot(&app.state::<AppState>().0.lock().unwrap());

        for command in [
            "remove_draft_cart_line_command",
            "discard_draft_cart_command",
        ] {
            assert!(
                get_ipc_response(&window, request(command)).is_err(),
                "{command}"
            );
        }

        assert_eq!(snapshot(&app.state::<AppState>().0.lock().unwrap()), before);
    }
}

pub mod catalog {
    pub use crate::application::catalog::{search_active_products, ProductSearchResult};
    pub use crate::infrastructure::sqlite::open_seeded_catalog;
}
