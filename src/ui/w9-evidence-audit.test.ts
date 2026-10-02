import { strict as assert } from "node:assert";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const read = (relative: string) => readFileSync(resolve(root, relative), "utf8");
const readFromHead = (relative: string) => execFileSync("git", ["show", `HEAD:${relative}`], { cwd: root, encoding: "utf8" });
const parseJson = (source: string) => JSON.parse(source) as Record<string, any>;
const clone = <T>(value: T): T => JSON.parse(JSON.stringify(value)) as T;
const css = read("src/ui/styles.css");

function assertTicket14PackageAllowlist(
  currentPackage: Record<string, any>,
  baselinePackage: Record<string, any>,
  currentLock: Record<string, any>,
  baselineLock: Record<string, any>,
) {
  assert.equal(currentPackage.scripts["typecheck:tests"], "tsc --project tsconfig.tests.json --noEmit");
  assert.equal(currentPackage.devDependencies["@types/jsdom"], "^30.0.0");

  const comparablePackage = clone(currentPackage);
  const comparableBaselinePackage = clone(baselinePackage);
  delete comparablePackage.scripts["typecheck:tests"];
  delete comparablePackage.devDependencies["@types/jsdom"];
  delete comparableBaselinePackage.scripts["typecheck:tests"];
  delete comparableBaselinePackage.devDependencies["@types/jsdom"];
  assert.deepEqual(comparablePackage, comparableBaselinePackage, "unexpected package.json drift");

  const currentRoot = clone(currentLock.packages[""]);
  const baselineRoot = clone(baselineLock.packages[""]);
  delete currentRoot.devDependencies["@types/jsdom"];
  delete baselineRoot.devDependencies["@types/jsdom"];
  assert.deepEqual(currentRoot, baselineRoot, "unexpected package-lock root drift");

  const allowedLockEntries = new Set([
    "node_modules/@types/jsdom",
    "node_modules/@types/jsdom/node_modules/undici-types",
    "node_modules/@types/tough-cookie",
  ]);
  const packageKeys = new Set([...Object.keys(currentLock.packages), ...Object.keys(baselineLock.packages)]);
  for (const key of packageKeys) {
    if (key === "" || allowedLockEntries.has(key)) continue;
    assert.deepEqual(currentLock.packages[key], baselineLock.packages[key], `unexpected package-lock drift: ${key}`);
  }

  assert.deepEqual(currentLock.packages["node_modules/@types/jsdom"], {
    version: "30.0.0",
    resolved: "https://registry.npmjs.org/@types/jsdom/-/jsdom-30.0.0.tgz",
    integrity: "sha512-uAHGxujGE0cDaKGdK28zgDotFtNA7MKq5DXl8LrfdxdCI8VHcg15oJz+amHTChPNI5JpgEPQWc2xFdrw3em/nQ==",
    dev: true,
    license: "MIT",
    dependencies: {
      "@types/node": "*",
      "@types/tough-cookie": "*",
      "parse5": "^8.0.0",
      "undici-types": "^8.9.0",
    },
  });
  assert.deepEqual(currentLock.packages["node_modules/@types/jsdom/node_modules/undici-types"], {
    version: "8.10.2",
    resolved: "https://registry.npmjs.org/undici-types/-/undici-types-8.10.2.tgz",
    integrity: "sha512-7/+aSjzkUoLc92hV22bTW4aGanXf800zbwguhcICs0OAoCF9wDOE4wkopQ+SqfhXZm8mCK8gHpdTs7pZUWzK3w==",
    dev: true,
    license: "MIT",
  });
  assert.deepEqual(currentLock.packages["node_modules/@types/tough-cookie"], {
    version: "4.0.5",
    resolved: "https://registry.npmjs.org/@types/tough-cookie/-/tough-cookie-4.0.5.tgz",
    integrity: "sha512-/Ad8+nIOV7Rl++6f1BdKxFSMgmoqEoYbHRpPcx3JEfv8VRsQe9Z4mCXeJBzxs7mbHY/XOZZuXlRNfhpVPbs6ZA==",
    dev: true,
    license: "MIT",
  });
}

const ticket11RegistrationMarkers = [
  "command_builder",
  "choose_backup_destination_command",
  "choose_restore_source_command",
  "create_backup_command",
  "prepare_restore_command",
  "confirm_restore_command",
];

const dashboardRegistrationLineAllowlist = new Set([
  "",
  "dashboard_command,",
  "#[cfg(feature = \"desktop\")]",
  "#[tauri::command]",
  "fn dashboard_command(",
  "state: tauri::State<AppState>,",
  "request: commands::dashboard::DashboardRequest,",
  ") -> commands::dashboard::DashboardResponse {",
  "state",
  ".with_read(|connection| Ok(commands::dashboard::dashboard(connection, request)))",
  ".unwrap_or_else(|_| commands::dashboard::DashboardResponse::Error(commands::dashboard::persistence_failure()))",
  "}",
  "#[test]",
  "fn registers_read_only_dashboard_command_at_the_tauri_command_seam() {",
  "let (app, window) = test_window();",
  "let before = app.state::<AppState>().with_read(|connection| Ok(snapshot(connection))).unwrap();",
  "assert!(get_ipc_response(&window, request_with(\"dashboard_command\", serde_json::json!({",
  "\"today_from_utc\": \"2024-03-10T05:00:00Z\",",
  "\"today_to_exclusive_utc\": \"2024-03-11T04:00:00Z\",",
  "\"month_from_utc\": \"2024-03-01T05:00:00Z\",",
  "\"month_to_exclusive_utc\": \"2024-04-01T04:00:00Z\"",
  "}))).is_ok());",
  "assert_eq!(app.state::<AppState>().with_read(|connection| Ok(snapshot(connection))).unwrap(), before);",
]);

// These are complete trimmed lines from the ticket-11 registration/test seam. Keeping
// complete lines (rather than matching marker substrings) prevents appended production
// or test text from riding on an otherwise permitted line.
const ticket11RegistrationLineAllowlist = new Set([
  "use tauri::Manager;",
  "#[cfg(feature = \"desktop\")]",
  "#[tauri::command]",
  "use tauri::{Manager, Runtime};",
  "#[cfg(all(feature = \"desktop\", not(test)))]",
  "#[cfg(all(feature = \"desktop\", test))]",
  "fn command_builder<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {",
  "fn command_builder<R: Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {",
  "sale_history_detail_command",
  "sale_history_detail_command,",
  "choose_backup_destination_command,",
  "choose_restore_source_command,",
  "create_backup_command,",
  "prepare_restore_command,",
  "confirm_restore_command",
  "builder",
  ".plugin(tauri_plugin_dialog::init())",
  ".invoke_handler(tauri::generate_handler![",
  "search_products_command,",
  "browse_products_command,",
  "confirm_sale_command,",
  "create_sale_return_command,",
  "cancel_sale_command,",
  "confirm_stock_entry_command,",
  "confirm_physical_count_command,",
  "list_inventory_alerts_command,",
  "list_catalog_maintenance_command,",
  "list_catalog_categories_command,",
  "maintain_catalog_command,",
  "edit_catalog_command",
  "catalog_metadata_detail_command,",
  "list_categories_command,",
  "create_category_command,",
  "create_product_command,",
  "list_sales_history_command,",
  "])",
  "command_builder(builder.plugin(tauri_plugin_dialog::init()))",
  "fn browse_products_command(",
  "state: tauri::State<AppState>,",
  "request: commands::catalog::BrowseProductsRequest,",
  ") -> Result<commands::catalog::ProductBrowseResponse, String> {",
  "state.with_read(|connection| Ok(commands::catalog::browse_products(connection, request)))",
  "fn list_catalog_categories_command(",
  ") -> Result<commands::catalog::CatalogMaintenanceListResponse, String> {",
  "state.with_read(commands::catalog::list_catalog_categories)",
  "commands::catalog::list_catalog_categories",
  "async fn choose_backup_destination_command(",
  "window: tauri::WebviewWindow,",
  "async fn choose_backup_destination_command<R: Runtime>(",
  "window: tauri::WebviewWindow<R>,",
  "window",
  ".app_handle()",
  ".dialog()",
  ".file()",
  ".pick_folder(move |path| {",
  "complete(path.and_then(|path| path.into_path().ok()));",
  "});",
  "#[cfg(test)]",
  "{",
  "let _ = window;",
  "complete(None);",
  "}",
  "#[cfg(not(test))]",
  "async fn choose_restore_source_command(",
  "async fn choose_restore_source_command<R: Runtime>(",
  ".add_filter(\"SQLite backup\", &[\"sqlite3\"])",
  ".pick_file(move |path| {",
  "#[cfg(any(windows, target_os = \"android\"))]",
  "const IPC_URL: &str = \"http://tauri.localhost\";",
  "#[cfg(not(any(windows, target_os = \"android\")))]",
  "const IPC_URL: &str = \"tauri://localhost\";",
  "",
  "url: \"tauri://localhost\".parse().unwrap(),",
  "url: IPC_URL.parse().unwrap(),",
  ".manage(Mutex::new(commands::backup::BackupCommandState::new(",
  "std::env::temp_dir(),",
  ")))",
  "#[test]",
  "fn registers_backup_commands_at_the_tauri_command_seam() {",
  "let (_app, window) = test_window();",
  "for command in [",
  "\"choose_backup_destination_command\",",
  "\"choose_restore_source_command\",",
  "] {",
  "let response = get_ipc_response(&window, request(command)).unwrap();",
  "assert_eq!(",
  "response.deserialize::<serde_json::Value>().unwrap(),",
  "serde_json::json!({ \"kind\": \"cancelled\" }),",
  "\"{command} must return the test cancellation response\",",
  ");",
  "assert!(get_ipc_response(",
  "&window,",
  "request_with(",
  "\"create_backup_command\",",
  "serde_json::json!({ \"destination\": \"/definitely/missing\" }),",
  "),",
  ")",
  ".is_ok());",
  "\"prepare_restore_command\",",
  "serde_json::json!({ \"source\": \"/definitely/missing.sqlite3\" }),",
  "\"confirm_restore_command\",",
  "serde_json::json!({ \"token\": \"unknown\", \"confirmed\": true }),",
  "}",
]);

const categorySchemaRegistrationLineAllowlist = new Set([
  "edit_category_schema_command,",
  "#[cfg(feature = \"desktop\")]",
  "#[tauri::command]",
  "fn edit_category_schema_command(",
  "state: tauri::State<AppState>,",
  "request: commands::catalog::EditCategorySchemaRequest,",
  ") -> commands::catalog::CatalogMaintenanceResponse {",
  "state",
  ".with_write(|connection| Ok(commands::catalog::edit_category_schema(connection, request)))",
  ".unwrap_or_else(|error| {",
  "commands::catalog::CatalogMaintenanceResponse::Error(",
  "commands::catalog::map_command_state_error(&error),",
  ")",
  "})",
  "}",
  "",
  "assert!(get_ipc_response(&window, request_with(\"edit_category_schema_command\", serde_json::json!({ \"category_id\": 1, \"expected_revision\": 1, \"fields\": [] }))).is_ok());",
]);

function assertCategorySchemaRegistrationAllowlist(libDiff: string) {
  const changedLines = libDiff
    .split("\n")
    .filter((line) => /^[+-](?![+-])/.test(line));
  assert.match(libDiff, /edit_category_schema_command/);
  assert.ok(
    changedLines.every((line) => categorySchemaRegistrationLineAllowlist.has(line.slice(1).trim())),
    "unexpected Category Field Management command registration drift",
  );
}

function assertCatalogRegistrationAllowlist(libDiff: string) {
  const changedLines = libDiff
    .split("\n")
    .filter((line) => /^[+-](?![+-])/.test(line));
  assert.match(libDiff, /browse_products_command/);
  assert.match(libDiff, /list_catalog_categories_command/);
  assert.ok(
    changedLines.every((line) => ticket11RegistrationLineAllowlist.has(line.slice(1).trim())),
    "unexpected Catalog command registration drift",
  );
}

const grossProfitCommandRegistrationLines = [
  "gross_profit_report_command,",
  "gross_profit_operations_command,",
  "",
  "#[cfg(feature = \"desktop\")]",
  "#[tauri::command]",
  "fn gross_profit_report_command(",
  "state: tauri::State<AppState>,",
  "request: commands::gross_profit::GrossProfitRequest,",
  ") -> commands::gross_profit::GrossProfitResponse {",
  "state.with_read(|connection| Ok(commands::gross_profit::gross_profit(connection, request)))",
  ".unwrap_or_else(|_| commands::gross_profit::GrossProfitResponse::Error(commands::gross_profit::GrossProfitError {",
  "code: \"persistence_failure\", message: \"The gross-profit report could not be loaded.\",",
  "}))",
  "}",
  "#[cfg(feature = \"desktop\")]",
  "#[tauri::command]",
  "fn gross_profit_operations_command(",
  "state: tauri::State<AppState>,",
  "request: commands::gross_profit_operations::GrossProfitOperationsRequest,",
  ") -> commands::gross_profit_operations::GrossProfitOperationsResponse {",
  "state.with_read(|connection| Ok(commands::gross_profit_operations::gross_profit_operations(connection, request)))",
  ".unwrap_or_else(|_| commands::gross_profit_operations::GrossProfitOperationsResponse::Error(",
  "commands::gross_profit_operations::GrossProfitOperationsError {",
  "code: \"persistence_failure\",",
  "message: \"The gross-profit operations could not be loaded.\",",
  ")",
  "));",
  "fn registers_gross_profit_operations_command_with_strict_paged_request_at_ipc_seam() {",
  "let (app, window) = test_window();",
  "let before = app.state::<AppState>().with_read(|connection| Ok(snapshot(connection))).unwrap();",
  "let response = get_ipc_response(&window, request_with(\"gross_profit_operations_command\", serde_json::json!({",
  "\"from_utc\": \"2024-03-10T05:00:00.000Z\",",
  "\"to_exclusive_utc\": \"2024-03-11T04:00:00.000Z\",",
  "\"page\": 1,",
  "\"page_size\": 20",
  "}))).unwrap();",
  "let payload = response.deserialize::<serde_json::Value>().unwrap();",
  "assert_eq!(payload[\"kind\"], \"success\");",
  "assert_eq!(payload[\"report\"][\"total\"], 0);",
  "assert_eq!(app.state::<AppState>().with_read(|connection| Ok(snapshot(connection))).unwrap(), before);",
];

function assertGrossProfitCommandRegistrationDiff(libDiff: string) {
  const changedLines = libDiff.split("\n").filter((line) => /^[+-](?![+-])/.test(line));
  const operationCommandLines = new Set([
    "gross_profit_operations_command,", "#[cfg(feature = \"desktop\")]", "#[tauri::command]",
    "fn gross_profit_operations_command(", "state: tauri::State<AppState>,",
    "request: commands::gross_profit_operations::GrossProfitOperationsRequest,",
    ") -> commands::gross_profit_operations::GrossProfitOperationsResponse {",
    "state.with_read(|connection| Ok(commands::gross_profit_operations::gross_profit_operations(connection, request)))",
    ".unwrap_or_else(|_| commands::gross_profit_operations::GrossProfitOperationsResponse::Error(",
    "commands::gross_profit_operations::GrossProfitOperationsError {",
    "code: \"persistence_failure\",",
    "message: \"The gross-profit operations could not be loaded.\",",
    ")", "));", "))", "}", "},", "",
    "#[test]", "fn registers_gross_profit_operations_command_with_strict_paged_request_at_ipc_seam() {",
    "let (app, window) = test_window();",
    "let before = app.state::<AppState>().with_read(|connection| Ok(snapshot(connection))).unwrap();",
    "let response = get_ipc_response(&window, request_with(\"gross_profit_operations_command\", serde_json::json!({",
    "\"from_utc\": \"2024-03-10T05:00:00.000Z\",", "\"to_exclusive_utc\": \"2024-03-11T04:00:00.000Z\",",
    "\"page\": 1,", "\"page_size\": 20", "}))).unwrap();",
    "let payload = response.deserialize::<serde_json::Value>().unwrap();", "assert_eq!(payload[\"kind\"], \"success\");",
    "assert_eq!(payload[\"report\"][\"total\"], 0);",
    "assert_eq!(app.state::<AppState>().with_read(|connection| Ok(snapshot(connection))).unwrap(), before);",
  ]);
  if (libDiff.includes("gross_profit_operations_command")) {
    assert.deepEqual(changedLines.filter((line) => !operationCommandLines.has(line.slice(1).trim())).map((line) => line.slice(1).trim()), [], "unexpected gross-profit operations command registration drift");
    return;
  }
  assert.deepEqual(changedLines.map((line) => line.slice(1).trim()).sort(), [...grossProfitCommandRegistrationLines].sort(), "unexpected gross-profit command registration drift");
  assert.match(libDiff, /gross_profit_report_command/);
}

function assertDashboardRegistrationAllowlist(libDiff: string) {
  const changedLines = libDiff
    .split("\n")
    .filter((line) => /^[+-](?![+-])/.test(line));
  assert.match(libDiff, /dashboard_command/);
  assert.match(libDiff, /registers_read_only_dashboard_command_at_the_tauri_command_seam/);
  assert.ok(
    changedLines.every((line) => dashboardRegistrationLineAllowlist.has(line.slice(1).trim())),
    "unexpected Dashboard command registration drift",
  );
}

const commandSeamPricingLineAllowlist = new Set([
  "                acknowledged_price_centavos: None,",
  "            acknowledged_price_centavos: None,",
  "fn exposes_catalog_price_without_physical_storage_terminology() {",
  "fn exposes_sale_price_without_physical_storage_terminology() {",
  "    assert_eq!(results[0].catalog_unit_price_centavos, 2_500);",
  "    assert_eq!(results[0].sale_price_centavos, 2_500);",
  "            list_price_centavos: 5_000,",
  "            purchase_price_centavos: 3_000,",
  "            sale_price_centavos: 5_000,",
  `    let product = r#"{"sku":"BRG-1","name":"Wheel bearing","category_id":1,"list_price_centavos":5000,"minimum_sale_price_centavos":4000,"opening_quantity":3,"attribute_values":[],"unexpected":true}"#;`,
  `    let product = r#"{"sku":"BRG-1","name":"Wheel bearing","category_id":1,"purchase_price_centavos":3000,"sale_price_centavos":5000,"minimum_sale_price_centavos":4000,"opening_quantity":3,"attribute_values":[],"unexpected":true}"#;`,
  "fn onboarded_product_searches_and_sells_at_its_backend_catalog_price() {",
  "fn onboarded_product_searches_and_sells_at_its_backend_sale_price() {",
  "            low_stock_threshold: None,",
  "        \"attribute_values\", \"available_quantity\", \"category_id\", \"category_name\", \"minimum_sale_price_centavos\", \"name\", \"primary_location_code\", \"product_id\", \"purchase_price_centavos\", \"revision\", \"sale_price_centavos\", \"sku\"",
  "        \"attribute_values\", \"available_quantity\", \"category_id\", \"category_name\", \"minimum_sale_price_centavos\", \"name\", \"product_id\", \"purchase_price_centavos\", \"revision\", \"sale_price_centavos\", \"sku\"",
  "    assert_eq!(results[0].catalog_unit_price_centavos, 5_000);",
  "            low_stock_threshold: None,",
  "        assert_eq!(results[0].list_price_centavos, 5_000);",
  "        assert_eq!(results[0].minimum_sale_price_centavos, 4_000);",
  "    assert_eq!(results[0].purchase_price_centavos, Some(3_000));",
  "    assert_eq!(results[0].sale_price_centavos, 5_000);",
  "    assert_eq!(results[0].minimum_sale_price_centavos, 4_000);",
  "                captured_unit_price_centavos: 4_000,",
  "                captured_unit_price_centavos: 5_000,",
  "                final_unit_price_centavos: None,",
  "                final_unit_price_centavos: Some(5_000),",
  "                qr_applied_centavos: Some(4_000),",
  "                qr_applied_centavos: Some(5_000),",
  "    assert_eq!(summary.lines[0].unit_price_centavos, 4_000);",
  "    assert_eq!(summary.lines[0].unit_price_centavos, 5_000);",
  "    assert_eq!(product.list_price_centavos, 5_000);",
  "    assert_eq!(product.purchase_price_centavos, 3_000);",
  "    assert_eq!(product.sale_price_centavos, 5_000);",
  "    assert_eq!(product.minimum_sale_price_centavos, 4_000);",
]);

const salesBrowseCommandSeamLineAllowlist = new Set([
  "-use repuestos_autos::commands::catalog::{browse_products, search_products, BrowseProductsRequest, ProductBrowseResponse, SearchProductsRequest};",
  "+use repuestos_autos::commands::catalog::{browse_products, browse_sale_products, search_products, BrowseProductsRequest, ProductBrowseResponse, SearchProductsRequest};",
  "+#[test]",
  "+fn sales_browse_command_serializes_only_bounded_safe_product_attributes() {",
  "+    let connection = open_seeded_catalog().unwrap();",
  "+    connection.execute(\"UPDATE products SET purchase_price_centavos = 7777, revision = 9 WHERE id = 1\", []).unwrap();",
  "+    connection.execute(\"INSERT INTO attribute_definitions (id, category_id, label, field_type, required) VALUES (7, 1, 'Material', 'text', 0)\", []).unwrap();",
  "+    connection.execute(\"INSERT INTO product_attribute_values (product_id, definition_id, text_value, searchable_value) VALUES (1, 7, 'Paper', 'Paper')\", []).unwrap();",
  "+    let page = browse_sale_products(&connection, BrowseProductsRequest {",
  "+        query: Some(\"filtro\".into()), category_id: Some(1), stock_state: \"all\".into(), activity: \"all\".into(), page: 1, page_size: 20,",
  "+    }).unwrap();",
  "+    let value = serde_json::to_value(&page).unwrap();",
  "+    assert_eq!(value[\"products\"][0].as_object().unwrap().keys().map(String::as_str).collect::<Vec<_>>(), vec![",
  "+        \"attribute_values\", \"available_quantity\", \"category_id\", \"category_name\", \"minimum_sale_price_centavos\", \"name\", \"product_id\", \"sale_price_centavos\", \"sku\"",
  "+    ]);",
  "+    assert_eq!(value[\"products\"][0][\"attribute_values\"], serde_json::json!([",
  "+        { \"definition_id\": 7, \"label\": \"Material\", \"value\": \"Paper\" }",
  "+    ]));",
  "+    assert!(!value.to_string().contains(\"7777\"));",
  "+    assert!(!value.to_string().contains(\"revision\"));",
  "+    assert!(!value.to_string().contains(\"location\"));",
  "+    assert!(!value.to_string().contains(\"activity\"));",
  "+}",
  "+",
]);

function assertCommandSeamPricingAllowlist(commandSeamDiff: string) {
  if (commandSeamDiff.includes("sales_browse_command_serializes_only_bounded_safe_product_attributes")) {
    const changed = commandSeamDiff.split("\n").filter((line) => /^[+-](?![+-])/.test(line));
    const authorized = changed.filter((line) => salesBrowseCommandSeamLineAllowlist.has(line));
    assert.deepEqual(authorized.sort(), [...salesBrowseCommandSeamLineAllowlist].sort());
    commandSeamDiff = commandSeamDiff.split("\n").filter((line) => !salesBrowseCommandSeamLineAllowlist.has(line)).join("\n");
  }
  const changedLines = commandSeamDiff
    .split("\n")
    .filter((line) => /^[+-](?![+-])/.test(line));
  if (changedLines.length === 0) return;
  if (commandSeamDiff.includes("low_stock_threshold: None")) {
    assert.match(commandSeamDiff, /onboarded_product_searches_and_sells_at_its_backend_sale_price/);
    assert.match(commandSeamDiff, /primary_location_code/);
    assert.equal(changedLines.filter((line) => line === "+            low_stock_threshold: None,").length, 2);
    assert.equal(changedLines.filter((line) => line.includes("primary_location_code")).length, 1);
  } else {
    assert.match(commandSeamDiff, /onboarded_product_searches_and_sells_at_its_backend_sale_price/);
    assert.match(commandSeamDiff, /captured_unit_price_centavos: 5_000/);
    assert.match(commandSeamDiff, /qr_applied_centavos: Some\(5_000\)/);
    assert.match(commandSeamDiff, /summary\.lines\[0\]\.unit_price_centavos, 5_000/);
  }
  assert.ok(
    changedLines.every((line) => commandSeamPricingLineAllowlist.has(line.slice(1))),
    "unexpected command-seam contract-test drift",
  );
}

const catalogImageRegistrationLineAllowlist = new Set([
  "choose_product_image_command,",
  "remove_product_image_command,",
  "catalog_product_image_thumbnail_command,",
  "#[cfg(feature = \"desktop\")]",
  "#[tauri::command]",
  "async fn choose_product_image_command<R: Runtime>(",
  "state: tauri::State<'_, AppState>,",
  "window: tauri::WebviewWindow<R>,",
  "request: commands::catalog::ProductImageRequest,",
  ") -> commands::catalog::ProductImageResponse {",
  "let Ok(request) = commands::catalog::parse_product_image_request(request) else {",
  "return commands::catalog::ProductImageResponse::Error(commands::catalog::CatalogMaintenanceError {",
  "code: \"validation_error\", message: \"Review the catalog values and try again.\",",
  "});",
  "};",
  "let selection = commands::backup::select_callback_path(|complete| {",
  "#[cfg(test)]",
  "{ let _ = window; complete(None); }",
  "#[cfg(not(test))]",
  "{ window.app_handle().dialog().file().add_filter(\"Product image\", &[\"png\", \"jpg\", \"jpeg\", \"webp\"]).pick_file(move |path| {",
  "complete(path.and_then(|path| path.into_path().ok()));",
  "}); }",
  "}).await;",
  "let commands::backup::PathSelection::Selected { path } = selection else {",
  "return commands::catalog::ProductImageResponse::Cancelled;",
  "};",
  "let read_result = read_selected_image(&path);",
  "let (mime, bytes) = match read_result {",
  "Ok(value) => value,",
  "Err(()) => return commands::catalog::ProductImageResponse::Error(commands::catalog::CatalogMaintenanceError {",
  "code: \"image_unavailable\", message: \"The selected image could not be used.\",",
  "}),",
  "};",
  "state.with_write(|connection| Ok(commands::catalog::persist_selected_product_image(",
  "connection, request.product_id, request.expected_revision, mime, bytes,",
  "))).unwrap_or_else(|_| commands::catalog::ProductImageResponse::Error(commands::catalog::CatalogMaintenanceError {",
  "code: \"persistence_failure\", message: \"The catalog could not be completed.\",",
  "}))",
  "fn read_selected_image(path: &std::path::Path) -> Result<(&'static str, Vec<u8>), ()> {",
  "use std::io::Read;",
  "const MAX_BYTES: u64 = application::catalog::MAX_PRODUCT_IMAGE_BYTES as u64;",
  "let mut file = std::fs::File::open(path).map_err(|_| ())?;",
  "if file.metadata().map_err(|_| ())?.len() > MAX_BYTES { return Err(()); }",
  "let mut bytes = Vec::new();",
  "file.take(MAX_BYTES + 1).read_to_end(&mut bytes).map_err(|_| ())?;",
  "if bytes.len() as u64 > MAX_BYTES { return Err(()); }",
  "let extension = path.extension().and_then(|value| value.to_str()).unwrap_or_default().to_ascii_lowercase();",
  "let mime = match extension.as_str() { \"png\" => \"image/png\", \"jpg\" | \"jpeg\" => \"image/jpeg\", \"webp\" => \"image/webp\", _ => return Err(()) };",
  "Ok((mime, bytes))",
  "fn remove_product_image_command(",
  "state: tauri::State<AppState>, request: commands::catalog::ProductImageRequest,",
  ") -> commands::catalog::ProductImageResponse {",
  "state.with_write(|connection| Ok(commands::catalog::remove_product_image(connection, request)))",
  ".unwrap_or_else(|_| commands::catalog::ProductImageResponse::Error(commands::catalog::CatalogMaintenanceError {",
  "fn catalog_product_image_thumbnail_command(",
  ") -> commands::catalog::ProductImageThumbnailResponse {",
  "state.with_read(|connection| Ok(commands::catalog::catalog_product_image_thumbnail(connection, request)))",
  ".unwrap_or_else(|_| commands::catalog::ProductImageThumbnailResponse::Error(commands::catalog::CatalogMaintenanceError {",
  "#[test]",
  "fn registers_catalog_image_commands_without_exposing_picker_paths() {",
  "let (_app, window) = test_window();",
  "let picker = get_ipc_response(&window, request_with(\"choose_product_image_command\", serde_json::json!({ \"product_id\": 1, \"expected_revision\": 0 }))).unwrap();",
  "assert_eq!(picker.deserialize::<serde_json::Value>().unwrap(), serde_json::json!({ \"kind\": \"cancelled\" }));",
  "for command in [\"remove_product_image_command\", \"catalog_product_image_thumbnail_command\"] {",
  "let response = get_ipc_response(&window, request_with(command, serde_json::json!({ \"product_id\": 1, \"expected_revision\": 0 }))).unwrap();",
  "let value = response.deserialize::<serde_json::Value>().unwrap();",
  "assert!(!value.to_string().contains(\"path\"));",
  "}",
  "",
]);

function assertCatalogImageRegistrationAllowlist(libDiff: string) {
  const changedLines = libDiff
    .split("\n")
    .filter((line) => /^[+-](?![+-])/.test(line));
  for (const marker of ["choose_product_image_command", "remove_product_image_command", "catalog_product_image_thumbnail_command"]) {
    assert.match(libDiff, new RegExp(marker), `missing Catalog image marker: ${marker}`);
  }
  assert.ok(
    changedLines.every((line) => catalogImageRegistrationLineAllowlist.has(line.slice(1).trim())),
    "unexpected Catalog image command registration drift",
  );
}

const productLocationRegistrationLineAllowlist = new Set([
  "",
  "location_schema_command,",
  "save_location_schema_command,",
  "list_product_locations_command,",
  "create_product_location_command,",
  "activate_product_location_command,",
  "deactivate_product_location_command,",
  "delete_product_location_command,",
  "assign_product_primary_location_command,",
  "#[cfg(feature = \"desktop\")]",
  "#[tauri::command]",
  "fn location_schema_command(state: tauri::State<AppState>) -> commands::catalog::ProductLocationResponse {",
  "state.with_read(|connection| Ok(commands::catalog::location_schema(connection)))",
  ".unwrap_or_else(|_| commands::catalog::ProductLocationResponse::Error(commands::catalog::CatalogLocationError { code: \"persistence_failure\", message: \"The location change could not be completed.\" }))",
  "}",
  "fn save_location_schema_command(state: tauri::State<AppState>, request: commands::catalog::SaveLocationSchemaRequest) -> commands::catalog::ProductLocationResponse {",
  "state.with_write(|connection| Ok(commands::catalog::save_location_schema(connection, request)))",
  "fn list_product_locations_command(state: tauri::State<AppState>, request: commands::catalog::ListProductLocationsRequest) -> commands::catalog::ProductLocationResponse {",
  "state.with_read(|connection| Ok(commands::catalog::list_product_locations(connection, request)))",
  "fn create_product_location_command(state: tauri::State<AppState>, request: commands::catalog::CreateProductLocationRequest) -> commands::catalog::ProductLocationResponse {",
  "state.with_write(|connection| Ok(commands::catalog::create_product_location(connection, request)))",
  "fn activate_product_location_command(state: tauri::State<AppState>, request: commands::catalog::ProductLocationLifecycleRequest) -> commands::catalog::ProductLocationResponse {",
  "state.with_write(|connection| Ok(commands::catalog::set_product_location_activity(connection, request, true)))",
  "fn deactivate_product_location_command(state: tauri::State<AppState>, request: commands::catalog::ProductLocationLifecycleRequest) -> commands::catalog::ProductLocationResponse {",
  "state.with_write(|connection| Ok(commands::catalog::set_product_location_activity(connection, request, false)))",
  "fn delete_product_location_command(state: tauri::State<AppState>, request: commands::catalog::ProductLocationLifecycleRequest) -> commands::catalog::ProductLocationResponse {",
  "state.with_write(|connection| Ok(commands::catalog::delete_product_location(connection, request)))",
  "fn assign_product_primary_location_command(state: tauri::State<AppState>, request: commands::catalog::AssignProductLocationRequest) -> commands::catalog::ProductLocationResponse {",
  "state.with_write(|connection| Ok(commands::catalog::assign_product_primary_location(connection, request)))",
  "#[test]",
  "fn registers_product_location_contract_commands_at_the_tauri_command_seam() {",
  "let (_app, window) = test_window();",
  "assert!(get_ipc_response(&window, request(\"location_schema_command\")).is_ok());",
  "assert!(get_ipc_response(&window, request_with(\"save_location_schema_command\", serde_json::json!({ \"expected_revision\": 0, \"segments\": [\"Zone\"] }))).is_ok());",
  "assert!(get_ipc_response(&window, request_with(\"list_product_locations_command\", serde_json::json!({ \"include_inactive\": false }))).is_ok());",
  "assert!(get_ipc_response(&window, request_with(\"create_product_location_command\", serde_json::json!({ \"values\": [\"A1\"] }))).is_ok());",
  "for (command, payload) in [",
  "(\"activate_product_location_command\", serde_json::json!({ \"location_id\": 1, \"expected_revision\": 0 })),",
  "(\"deactivate_product_location_command\", serde_json::json!({ \"location_id\": 1, \"expected_revision\": 0 })),",
  "(\"delete_product_location_command\", serde_json::json!({ \"location_id\": 1, \"expected_revision\": 0 })),",
  "(\"assign_product_primary_location_command\", serde_json::json!({ \"product_id\": 1, \"expected_revision\": 0, \"location_id\": 1 })),",
  "] {",
  "assert!(get_ipc_response(&window, request_with(command, payload)).is_ok(), \"{command}\");",
  "}",
]);

function assertProductLocationRegistrationAllowlist(libDiff: string) {
  const changedLines = libDiff
    .split("\n")
    .filter((line) => /^[+-](?![+-])/.test(line));
  assert.match(libDiff, /registers_product_location_contract_commands_at_the_tauri_command_seam/);
  assert.ok(
    changedLines.every((line) => productLocationRegistrationLineAllowlist.has(line.slice(1).trim())),
    "unexpected product location command registration drift",
  );
}

const reportsRegistrationLineAllowlist = new Set([
  "",
  "list_movement_ledger_command,",
  "list_movement_ledger_product_options_command,",
  "export_movement_ledger_command,",
  "#[cfg(feature = \"desktop\")]",
  "#[tauri::command]",
  "fn list_movement_ledger_command(",
  "state: tauri::State<AppState>,",
  "request: commands::movement_ledger::MovementLedgerRequest,",
  ") -> commands::movement_ledger::MovementLedgerResponse {",
  "state",
  ".with_read(|connection| Ok(commands::movement_ledger::list_movement_ledger(connection, request)))",
  ".unwrap_or_else(|_| {",
  "commands::movement_ledger::MovementLedgerResponse::Error(",
  "commands::movement_ledger::MovementLedgerCommandError {",
  "code: \"persistence_failure\",",
  "message: \"The movement ledger could not be loaded.\",",
  "}",
  ")",
  "},",
  "),",
  "))",
  "})",
  "fn list_movement_ledger_product_options_command(",
  "request: commands::movement_ledger::MovementLedgerProductOptionsRequest,",
  ") -> commands::movement_ledger::MovementLedgerProductOptionsResponse {",
  "state.with_read(|connection| Ok(commands::movement_ledger::list_movement_ledger_product_options(connection, request)))",
  ".unwrap_or_else(|_| commands::movement_ledger::MovementLedgerProductOptionsResponse::Error(",
  "message: \"The movement ledger products could not be loaded.\",",
  "async fn export_movement_ledger_command<R: Runtime>(",
  "app_handle: tauri::AppHandle<R>,",
  "request: commands::movement_ledger::MovementLedgerExportRequest,",
  ") -> Result<commands::movement_ledger::MovementLedgerExportResponse, String> {",
  "#[cfg(test)]",
  "{",
  "let _ = (app_handle, request);",
  "Ok(commands::movement_ledger::MovementLedgerExportResponse::Cancelled)",
  "}",
  "#[cfg(not(test))]",
  "let selection = commands::backup::select_callback_path(|complete| {",
  "app_handle.dialog().file()",
  ".add_filter(\"PDF document\", &[\"pdf\"])",
  ".set_file_name(\"movement-ledger.pdf\")",
  ".save_file(move |path| {",
  "complete(path.and_then(|path| path.into_path().ok()));",
  "});",
  "}).await;",
  "let commands::backup::PathSelection::Selected { path } = selection else {",
  "return commands::movement_ledger::MovementLedgerExportResponse::Cancelled;",
  "};",
  "let generated_at = time::OffsetDateTime::now_utc()",
  ".format(&time::format_description::well_known::Rfc3339)",
  ".unwrap_or_else(|_| \"Unavailable\".to_string());",
  "let state = app_handle.state::<AppState>();",
  "let response = state.with_read(|connection| Ok(commands::movement_ledger::export_movement_ledger(",
  ".unwrap_or_else(|_| commands::movement_ledger::MovementLedgerExportResponse::Error(",
  "connection, request, &generated_at, |bytes| {",
  "std::fs::write(&path, bytes).map_or(",
  "commands::movement_ledger::ExportSaveResult::Failed,",
  "|_| commands::movement_ledger::ExportSaveResult::Saved,",
  "},",
  "),",
  "}))).unwrap_or_else(|_| commands::movement_ledger::MovementLedgerExportResponse::Error(",
  "))).unwrap_or_else(|_| commands::movement_ledger::MovementLedgerExportResponse::Error(",
  "}),",
  "));",
  "Ok(response)",
  "return Ok(commands::movement_ledger::MovementLedgerExportResponse::Cancelled);",
  "}",
]);

const removedReportsRegistrationLineAllowlist = new Set([
  "state.with_read(|connection| Ok(commands::movement_ledger::list_movement_ledger_product_options(connection)))",
  "state: tauri::State<'_, AppState>,",
  "window: tauri::WebviewWindow<R>,",
  ") -> commands::movement_ledger::MovementLedgerExportResponse {",
  "let _ = (state, window, request);",
  "commands::movement_ledger::MovementLedgerExportResponse::Cancelled",
  "return commands::movement_ledger::MovementLedgerExportResponse::Cancelled;",
  "let app_handle = window.app_handle().clone();",
  "drop(window);",
  "state.with_read(|connection| Ok(commands::movement_ledger::export_movement_ledger(",
  "))",
]);

function assertReportsRegistrationAllowlist(libDiff: string, contract: "export" | "product-options" = "export") {
  const changedLines = libDiff
    .split("\n")
    .filter((line) => /^[+-](?![+-])/.test(line));
  if (contract === "product-options") {
    assert.match(libDiff, /MovementLedgerProductOptionsRequest/, "missing Reports bounded product-options request marker");
  } else {
    assert.match(libDiff, /export_movement_ledger_command/, "missing Reports export command marker");
  }
  const unexpectedLines = changedLines.filter((line) => {
    const content = line.slice(1).trim();
    return line.startsWith("+")
      ? !reportsRegistrationLineAllowlist.has(content)
      : !removedReportsRegistrationLineAllowlist.has(content);
  });
  assert.deepEqual(unexpectedLines, [], "unexpected Reports command registration drift");
}

function assertBackupRecoveryStartupAllowlist(libDiff: string) {
  const changedLines = libDiff.split("\n").filter((line) => /^[+-](?![+-])/.test(line));
  assert.match(libDiff, /retained_recovery_evidence_is_valid/);
  assert.match(libDiff, /has_ambiguous_temporary_artifacts/);
  const allowedStartupHunks = new Set([51, 53, 66, 178, 204, 210, 218, 224, 242, 280, 288, 304, 311, 360, 377]);
  const allowedBackupRecoveryHunks = new Set([...allowedStartupHunks, 58, 192, 210, 243, 261, 385, 698, 706, 710, 717, 725, 729, 734, 742, 746, 754, 762, 766, 863, 866, 871, 874, 875, 878, 1516, 1524, 1528, 1547, 1555, 1559, 1689, 1697, 1701, 1705, 1709]);
  const hunks = libDiff.split(/(?=^@@ )/m);
  const changedHunks = hunks.filter((hunk) => /^[+-](?![+-])/m.test(hunk));
  const unexpectedHunks = changedHunks.filter((hunk) => {
    const header = hunk.match(/^@@ -\d+(?:,\d+)? \+(\d+)(?:,\d+)? @@/);
    return !header || !allowedBackupRecoveryHunks.has(Number(header[1]));
  });
  assert.deepEqual(unexpectedHunks, [], "unexpected backup recovery/lib command-seam hunk drift");
  const startupLines = hunks.flatMap((hunk) => {
    const header = hunk.match(/^@@ -\d+(?:,\d+)? \+(\d+)(?:,\d+)? @@/);
    if (!header || !allowedStartupHunks.has(Number(header[1]))) return [];
    return hunk.split("\n").filter((line) => /^[+-](?![+-])/.test(line));
  });
  assert.ok(startupLines.length > 0, "expected bounded backup recovery startup hunks");
  assert.ok(changedLines.length >= startupLines.length, "recovery audit must not omit unrelated lib.rs changes");
  const registrationLines = changedLines.filter((line) => ticket11RegistrationLineAllowlist.has(line.slice(1).trim()));
  const registrationDiff = [
    "+fn command_builder<R: Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {",
    "+choose_backup_destination_command,",
    "+choose_restore_source_command,",
    "+create_backup_command,",
    "+prepare_restore_command,",
    "+confirm_restore_command",
    ...registrationLines,
    "+command_builder(builder.plugin(tauri_plugin_dialog::init()))",
  ].join("\n");
  assertTicket11RegistrationAllowlist(registrationDiff);
}

const windowsPickerCommandDiffLineAllowlist = new Set([
  "commands: tauri::State<'_, Mutex<commands::backup::BackupCommandState>>,",
  "let _ = window;",
  "window",
  ".app_handle()",
  ") -> commands::backup::BackupDestinationSelection {",
  ") -> commands::backup::RestoreSourceSelection {",
  "return commands::backup::BackupDestinationSelection::Cancelled;",
  "return commands::backup::BackupDestinationSelection::Error {",
  "};",
  "commands.select_backup_destination(path)",
  "return commands::backup::RestoreSourceSelection::Cancelled;",
  "return commands::backup::RestoreSourceSelection::Error {",
  "commands.select_restore_source(path)",
  ") -> Result<commands::backup::BackupDestinationSelection, String> {",
  "let app_handle = window.app_handle().clone();",
  "drop(window);",
  "let picker_app_handle = app_handle.clone();",
  "let _ = picker_app_handle;",
  "picker_app_handle",
  "return Ok(commands::backup::BackupDestinationSelection::Cancelled);",
  "let commands = app_handle.state::<Mutex<commands::backup::BackupCommandState>>();",
  "return Ok(commands::backup::BackupDestinationSelection::Error {",
  "});",
  "Ok(commands.select_backup_destination(path))",
  ") -> Result<commands::backup::RestoreSourceSelection, String> {",
  "return Ok(commands::backup::RestoreSourceSelection::Cancelled);",
  "let commands = app_handle.state::<Mutex<commands::backup::BackupCommandState>>();",
  "return Ok(commands::backup::RestoreSourceSelection::Error {",
  "Ok(commands.select_restore_source(path))",
]);

function assertWindowsPickerCommandDiffAllowlist(libDiff: string) {
  const changedLines = libDiff
    .split("\n")
    .filter((line) => /^[+-](?![+-])/.test(line));
  assert.match(libDiff, /Result<commands::backup::BackupDestinationSelection, String>/);
  assert.match(libDiff, /Result<commands::backup::RestoreSourceSelection, String>/);
  assert.ok(
    changedLines.every((line) => windowsPickerCommandDiffLineAllowlist.has(line.slice(1).trim())),
    "unexpected Windows picker-command diff drift",
  );
}

function assertTicket11RegistrationAllowlist(libDiff: string) {
  const changedLines = libDiff
    .split("\n")
    .filter((line) => /^[+-](?![+-])/.test(line));
  assert.ok(changedLines.length > 0, "expected ticket-11 lib.rs diff");
  for (const marker of ticket11RegistrationMarkers) {
    assert.match(libDiff, new RegExp(marker), `missing ticket-11 marker: ${marker}`);
  }
  assert.match(
    libDiff,
    /command_builder\(builder\.plugin\(tauri_plugin_dialog::init\(\)\)\)/,
    "missing production dialog/plugin registration seam",
  );
  assert.ok(
    changedLines.every((line) => ticket11RegistrationLineAllowlist.has(line.slice(1).trim())),
    "unexpected src-tauri/src/lib.rs drift outside the ticket-11 registration seam",
  );
}

function assertReportsCapabilityAllowlist(currentContent: string, baselineContent: string, capabilityDiff: string) {
  const current = parseJson(currentContent);
  const baseline = parseJson(baselineContent);
  const changedLines = capabilityDiff
    .split("\n")
    .filter((line) => /^[+-](?![+-])/.test(line))
    .map((line) => line.trim());

  if (baseline.permissions.includes("dialog:allow-save")) {
    assert.deepEqual(current, baseline, "unexpected uncommitted Reports capability drift");
    assert.deepEqual(changedLines, [], "committed Reports permission must not appear as uncommitted drift");
    return;
  }

  assert.deepEqual(baseline.permissions, ["core:default", "dialog:allow-open"]);
  assert.deepEqual(current, {
    ...baseline,
    permissions: [...baseline.permissions, "dialog:allow-save"],
  }, "Reports may only append dialog:allow-save to the existing capability");
  assert.deepEqual(changedLines, [
    '-  "permissions": ["core:default", "dialog:allow-open"]',
    '+  "permissions": ["core:default", "dialog:allow-open", "dialog:allow-save"]',
  ], "unexpected Reports capability diff");
}

const backupIntegrityDiffSha256Allowlist: Record<string, string> = {
  "src-tauri/src/lib.rs": "42db0ab1b23a27786df7d03d1ea2d0eb18d774d197cca95bef361590fa45d0cc",
  "src-tauri/src/commands/backup.rs": "7169b1ec4fae68e8e9b08aa8c92011130e9f03e8ad716f2af706fe173166853b",
  "src-tauri/tests/backup_restore.rs": "bfd547aef8a58b83b871dab889922dd7642f4df0ad3d8dd5cfffca6ca4301ab8",
  "odd/tasks/backup-restore-integrity-hardening.md": "7bb231e22c5095e2007c15e420ad1a6a39f6dbe829080a2c16f21b0f631a72cb",
  "src/ui/w9-evidence-audit.test.ts": "e2be820d50eebb4a27d1153677f46e8188540eaf4948d5f507a72e53ec6425a6",
  "src-tauri/src/infrastructure/filesystem/backup_store.rs": "5c93997017b3b59ac6069207867586788a381df42ac296415cbcf25347b4700c",
  "src-tauri/src/infrastructure/filesystem/restore_transitions.rs": "48e94bec5899ce2826548309281a9fe25d679f165fe47aceb0ca7ab722c8dd9d",
  "src-tauri/src/infrastructure/sqlite/backup.rs": "6f3d639a36f312b70af70641a9e3b723a563e02589b8876f791551f37d3eebf1",
  "src/commands/backup.ts": "22801231a00ee33709afa0e9ba5d67072775e9d1849edde32afafad693b00166",
  "src/commands/backup.test.ts": "2e832f565b509c1fe70cc25363153fcad69035a9ffd31e5f9d50fa3cd9110c84",
  "src/ui/backup/backup-flow.ts": "3070565d5f770fc9f8482c8951671663c612575390d8176ee7edf6b6b473de2f",
  "src/ui/backup/backup-flow.test.ts": "cfcf08a4834f1aac93b4eb89a9cdf1d21dbe5c337fc49eda465096adb655fe46",
  "src/ui/backup/backup-screen.ts": "b7a1fa63d38aec6719db217ee5fd0ad3f2495decfa29a4f96fc35f04234be809",
  "src/ui/backup/backup-screen.mounted.test.ts": "a8edf1386a28fdab5096581b3e5d41705e50214ada534825f44aa13bd17a3daf",
};

function sha256(value: string | Buffer): string {
  return createHash("sha256").update(value).digest("hex");
}

const catalogAccessCandidatePaths = [
  "odd/tasks/catalog-access-and-profit-report.md",
  "src-tauri/src/application/catalog/mod.rs",
  "src-tauri/src/commands/catalog.rs",
  "src-tauri/src/lib.rs",
  "src-tauri/tests/catalog_search.rs",
  "src/commands/catalog.test.ts",
  "src/commands/catalog.ts",
  "src/ui/inventory/inventory-screen.mounted.test.ts",
  "src/ui/inventory/inventory-screen.ts",
  "src/ui/onboarding/onboarding-screen.mounted.test.ts",
  "src/ui/onboarding/onboarding-screen.ts",
  "src/ui/w9-evidence-audit.test.ts",
];

const catalogAccessUnifiedDiffSha256 = "65df10d3e6bae980fd1b9b5df3606cd9756d15be528ba10e597e9ab0af78689d";

const grossProfitReportCandidatePaths = [
  "odd/tasks/catalog-access-and-profit-report.md",
  "src-tauri/src/application/reporting/mod.rs",
  "src-tauri/src/commands/mod.rs",
  "src-tauri/src/commands/gross_profit.rs",
  "src-tauri/src/infrastructure/sqlite/dashboard_repository.rs",
  "src-tauri/src/lib.rs",
  "src-tauri/tests/gross_profit_report.rs",
  "src/commands/gross-profit.ts",
  "src/commands/gross-profit.test.ts",
  "src/commands/sales-history.ts",
  "src/commands/sales-history.test.ts",
  "src/ui/app.ts",
  "src/ui/app-shell.mounted.test.ts",
  "src/ui/reports/gross-profit-report-screen.ts",
  "src/ui/reports/gross-profit-report-screen.mounted.test.ts",
  "src/ui/w9-evidence-audit.test.ts",
];

const grossProfitReportTaskBytesSha256 = "8485f4af59abeebdfc02b875593695e251adcb0cc0fc7cdad10df566afc39848";
const grossProfitReportCandidateDiffSha256 = "b7e94178ef05299dddfa5bd001206794afd4440564d802a519b156354b0826b3";

function grossProfitReportCandidateDiff() {
  return grossProfitReportCandidatePaths.map((path) => {
    const value = isTrackedPath(path)
      ? execFileSync("git", ["diff", "--unified=0", "HEAD", "--", path], { cwd: root, encoding: "utf8" })
      : readFileSync(resolve(root, path), "utf8");
    const normalized = path === "src/ui/w9-evidence-audit.test.ts"
      ? value
          .replaceAll(/^index \S+\.\.\S+.*$/gm, "index <SELF_INDEX>")
          .replaceAll(/^([+-])const grossProfitReportTaskBytesSha256 = "[^"]*";$/gm, '$1const grossProfitReportTaskBytesSha256 = "<SELF_TASK_HASH>";')
          .replaceAll(/^([+-])const grossProfitReportCandidateDiffSha256 = "[^"]*";$/gm, '$1const grossProfitReportCandidateDiffSha256 = "<SELF_DIFF_HASH>";')
          .replaceAll(/^([+-])(\s*"src\/ui\/w9-evidence-audit\.test\.ts": ")[^"]+(",)$/gm, '$1$2<SELF_W9_HASH>$3')
      : value;
    return `${path}\0${normalized}`;
  }).join("\0");
}

function assertGrossProfitReportCandidate(changedPaths: string[]) {
  const expected = [...grossProfitReportCandidatePaths].sort();
  assert.deepEqual(changedPaths.filter((path) => grossProfitReportCandidatePaths.includes(path)), expected);
  assert.equal(sha256(read("odd/tasks/catalog-access-and-profit-report.md")), grossProfitReportTaskBytesSha256, "unexpected T3 task document bytes");
  assert.equal(sha256(grossProfitReportCandidateDiff()), grossProfitReportCandidateDiffSha256, "unexpected T3 candidate diff or file bytes");
}

const catalogResponsiveTableCandidatePaths = [
  "odd/tasks/catalog-access-and-profit-report.md",
  "src/ui/styles.css",
  "src/ui/catalog/catalog-maintenance-screen.mounted.test.ts",
  "src/ui/visual-system/base-styles.mounted.test.ts",
  "src/ui/w9-evidence-audit.test.ts",
];

const catalogResponsiveTableUnifiedDiffSha256 = "59892e0e3e5eb56cbf47ea62be4eaf9a22e69380e9365d4fd2a354355b321126";

const windowsDesktopFixCandidatePaths = new Set([
  "src-tauri/Cargo.toml",
  "src-tauri/src/commands/backup.rs",
  "src-tauri/src/infrastructure/filesystem/backup_store.rs",
  "src-tauri/src/infrastructure/filesystem/restore_transitions.rs",
  "src-tauri/src/infrastructure/sqlite/backup.rs",
  "src-tauri/src/lib.rs",
  "src-tauri/tests/backup_restore.rs",
  "src/commands/backup.test.ts",
  "src/ui/w9-evidence-audit.test.ts",
  "odd/tasks/windows-desktop-build-fixes.md",
]);

const windowsDesktopFixDiffSha256Allowlist: Record<string, string> = {
  "src-tauri/Cargo.toml": "2cdaf77cce60c185d4fa2e3fe7f5b0c8a97ad4ef39847b5d467189000e0f79b5",
  "src-tauri/src/commands/backup.rs": "7169b1ec4fae68e8e9b08aa8c92011130e9f03e8ad716f2af706fe173166853b",
  "src-tauri/src/infrastructure/filesystem/backup_store.rs": "5c93997017b3b59ac6069207867586788a381df42ac296415cbcf25347b4700c",
  "src-tauri/src/infrastructure/sqlite/backup.rs": "6f3d639a36f312b70af70641a9e3b723a563e02589b8876f791551f37d3eebf1",
  "src-tauri/src/infrastructure/filesystem/restore_transitions.rs": "48e94bec5899ce2826548309281a9fe25d679f165fe47aceb0ca7ab722c8dd9d",
  "src-tauri/src/lib.rs": "42db0ab1b23a27786df7d03d1ea2d0eb18d774d197cca95bef361590fa45d0cc",
  "src-tauri/tests/backup_restore.rs": "bfd547aef8a58b83b871dab889922dd7642f4df0ad3d8dd5cfffca6ca4301ab8",
  "src/commands/backup.test.ts": "2e832f565b509c1fe70cc25363153fcad69035a9ffd31e5f9d50fa3cd9110c84",
  "src/ui/w9-evidence-audit.test.ts": "e2be820d50eebb4a27d1153677f46e8188540eaf4948d5f507a72e53ec6425a6",
  "odd/tasks/windows-desktop-build-fixes.md": "318ea8070661bb36109050be0cd16a2b4f22ca76043ce57fb80dacc1fe094c53",
};

const backupIntegrityCandidatePaths = new Set([
  "src-tauri/src/lib.rs",
  "src-tauri/src/commands/backup.rs",
  "src-tauri/src/infrastructure/filesystem/backup_store.rs",
  "src-tauri/src/infrastructure/filesystem/restore_transitions.rs",
  "src-tauri/src/infrastructure/sqlite/backup.rs",
  "src-tauri/tests/backup_restore.rs",
  "src/commands/backup.ts",
  "src/commands/backup.test.ts",
  "src/ui/backup/backup-flow.ts",
  "src/ui/backup/backup-flow.test.ts",
  "src/ui/backup/backup-screen.ts",
  "src/ui/backup/backup-screen.mounted.test.ts",
  "src/ui/w9-evidence-audit.test.ts",
  "odd/tasks/backup-restore-integrity-hardening.md",
]);

function assertBackupIntegrityDiffAllowlist(diffs: Record<string, string | Buffer>) {
  for (const [path, content] of Object.entries(diffs)) {
    if (path === "src-tauri/src/lib.rs" && typeof content === "string" && content.includes("gross_profit_report_command")) {
      assertGrossProfitCommandRegistrationDiff(content);
      continue;
    }
    if (path === "src-tauri/src/lib.rs" && typeof content === "string" && content.includes("onboarding_location_schema_command")) continue;
    const expected = backupIntegrityDiffSha256Allowlist[path];
    assert.ok(expected, `backup-integrity diff is not allowlisted: ${path}`);
    const normalized = typeof content === "string" && path === "src/ui/w9-evidence-audit.test.ts"
      ? content
          .replaceAll(/^index \S+\.\.\S+.*$/gm, "index <SELF_INDEX>")
          .replaceAll(/^([+-])(\s*"src\/ui\/w9-evidence-audit\.test\.ts": ")[^"]+(",)$/gm, '$1$2<SELF_DIFF_SHA256>$3')
          .replaceAll(/^([+-])const catalogAccessUnifiedDiffSha256 = "[^"]+";$/gm, '$1const catalogAccessUnifiedDiffSha256 = "<SELF_CATALOG_HASH>";')
          .replaceAll(/^([+-])const catalogResponsiveTableUnifiedDiffSha256 = "[^"]+";$/gm, '$1const catalogResponsiveTableUnifiedDiffSha256 = "<SELF_HASH>";')
          .replaceAll(/^([+-])const grossProfitReportTaskBytesSha256 = "[^"]*";$/gm, '$1const grossProfitReportTaskBytesSha256 = "<SELF_TASK_HASH>";')
          .replaceAll(/^([+-])const grossProfitReportCandidateDiffSha256 = "[^"]*";$/gm, '$1const grossProfitReportCandidateDiffSha256 = "<SELF_DIFF_HASH>";')
      : content;
    assert.equal(sha256(normalized), expected, `unexpected backup-integrity diff content: ${path}`);
  }
}

function assertWindowsDesktopFixDiffAllowlist(diffs: Record<string, string | Buffer>) {
  for (const [path, content] of Object.entries(diffs)) {
    if (path === "src-tauri/src/lib.rs" && typeof content === "string" && content.includes("gross_profit_report_command")) {
      assertGrossProfitCommandRegistrationDiff(content);
      continue;
    }
    if (path === "src-tauri/src/lib.rs" && typeof content === "string" && content.includes("onboarding_location_schema_command")) continue;
    const expected = windowsDesktopFixDiffSha256Allowlist[path];
    assert.ok(expected, `Windows desktop-fix diff is not allowlisted: ${path}`);
    const normalized = typeof content === "string" && path === "src/ui/w9-evidence-audit.test.ts"
      ? content
          .replaceAll(/^index \S+\.\.\S+.*$/gm, "index <SELF_INDEX>")
          .replaceAll(/^([+-])(\s*"src\/ui\/w9-evidence-audit\.test\.ts": ")[^"]+(",)$/gm, '$1$2<SELF_DIFF_SHA256>$3')
          .replaceAll(/^([+-])const catalogAccessUnifiedDiffSha256 = "[^"]+";$/gm, '$1const catalogAccessUnifiedDiffSha256 = "<SELF_CATALOG_HASH>";')
          .replaceAll(/^([+-])const catalogResponsiveTableUnifiedDiffSha256 = "[^"]+";$/gm, '$1const catalogResponsiveTableUnifiedDiffSha256 = "<SELF_HASH>";')
          .replaceAll(/^([+-])const grossProfitReportTaskBytesSha256 = "[^"]*";$/gm, '$1const grossProfitReportTaskBytesSha256 = "<SELF_TASK_HASH>";')
          .replaceAll(/^([+-])const grossProfitReportCandidateDiffSha256 = "[^"]*";$/gm, '$1const grossProfitReportCandidateDiffSha256 = "<SELF_DIFF_HASH>";')
      : content;
    assert.equal(sha256(normalized), expected, `unexpected Windows desktop-fix diff content: ${path}`);
  }
}

function changedPathsFromGitOutput(trackedChanges: string, untrackedFiles: string): string[] {
  return [...new Set(`${trackedChanges}\n${untrackedFiles}`.split("\n").map((path) => path.trim()).filter(Boolean))].sort();
}

function isTrackedPath(path: string): boolean {
  const trackedPaths = execFileSync(
    "git",
    ["ls-files", "--cached", "-z", "--", `:(literal)${path}`],
    { cwd: root },
  );
  return trackedPaths.length > 0;
}

const preExistingUnrelatedOddTaskBaseline = new Set([
  "odd/tasks/configurable-product-locations.md",
  "odd/tasks/developer-toolchain-rc-upgrade.md",
  "odd/tasks/inventory-pr-delivery.md",
  "odd/tasks/inventory-redesign.md",
  "odd/tasks/product-status-documentation.md",
  "odd/tasks/sales-explicit-product-details.md",
]);

function excludePreExistingUnrelatedOddTaskBaseline(changedPaths: string[]): string[] {
  return changedPaths.filter((path) => !preExistingUnrelatedOddTaskBaseline.has(path));
}

function assertW9ProtectedDiffPolicy(
  changedPaths: string[],
  currentPackageValue: Record<string, any>,
  baselinePackageValue: Record<string, any>,
  currentLockValue: Record<string, any>,
  baselineLockValue: Record<string, any>,
  libDiff: string,
  commandSeamDiff = "",
  capabilityDiff = "",
) {
  const allowedPaths = new Set([
    "package.json",
    "package-lock.json",
    "src-tauri/src/lib.rs",
    "src-tauri/src/commands/backup.rs",
    "src-tauri/src/infrastructure/filesystem/backup_store.rs",
    "src-tauri/src/infrastructure/filesystem/restore_transitions.rs",
    "src-tauri/src/infrastructure/sqlite/backup.rs",
    "src/commands/backup.ts",
    "src/commands/backup.test.ts",
    "src/ui/backup/backup-flow.ts",
    "src/ui/backup/backup-flow.test.ts",
    "src/ui/backup/backup-screen.ts",
    "src/ui/backup/backup-screen.mounted.test.ts",
    "odd/tasks/backup-restore-integrity-hardening.md",
    "src/commands/catalog.ts",
    "src/commands/catalog.test.ts",
    "src-tauri/src/application/catalog/mod.rs",
    "src-tauri/src/application/sales/application_contract.rs",
    "src-tauri/src/commands/confirm_sale.rs",
    "src-tauri/src/infrastructure/sqlite/sale_repository.rs",
    "src/commands/confirm-sale.ts",
    "src/commands/confirm-sale.test.ts",
    "src-tauri/src/application/catalog/access.rs",
    "src-tauri/src/commands/catalog.rs",
    "src-tauri/src/infrastructure/filesystem/catalog_access.rs",
    "src-tauri/src/infrastructure/filesystem/mod.rs",
    "src-tauri/tests/catalog_access.rs",
    "src-tauri/src/domain/catalog.rs",
    "odd/tasks/catalog-access-and-profit-report.md",
    "src-tauri/src/application/mod.rs",
    "src-tauri/src/commands/mod.rs",
    "src-tauri/src/commands/onboarding.rs",
    "src-tauri/tests/product_onboarding.rs",
    "src/commands/onboarding.ts",
    "src-tauri/src/infrastructure/sqlite/mod.rs",
    "src-tauri/src/infrastructure/sqlite/migrations/0016_product_images.sql",
    "src-tauri/src/infrastructure/sqlite/migrations/0021_sale_line_cost_snapshot.sql",
    "src-tauri/src/infrastructure/sqlite/sale_repository.rs",
    "src-tauri/src/infrastructure/sqlite/dashboard_repository.rs",
    "src-tauri/src/application/reporting/mod.rs",
    "src-tauri/src/commands/gross_profit.rs",
    "src-tauri/src/commands/gross_profit_operations.rs",
    "src-tauri/tests/gross_profit_report.rs",
    "src-tauri/tests/gross_profit_operations.rs",
    "odd/tasks/reporte-ganancia-operaciones-pdf.md",
    "src/commands/gross-profit.ts",
    "src/commands/gross-profit.test.ts",
    "src/commands/sales-history.ts",
    "src/commands/sales-history.test.ts",
    "src/ui/reports/gross-profit-report-screen.ts",
    "src/ui/reports/gross-profit-report-screen.mounted.test.ts",
    "src-tauri/tests/dashboard_reporting.rs",
    "src-tauri/tests/sale_cost_snapshot.rs",
    "src-tauri/src/infrastructure/sqlite/migrations/0017_product_image_thumbnails.sql",
    "src-tauri/src/infrastructure/sqlite/migrations/0019_category_field_lifecycle.sql",
    "src-tauri/src/infrastructure/sqlite/migrations/0022_product_low_stock_threshold.sql",
    "src-tauri/src/infrastructure/sqlite/catalog_repository.rs",
    "src-tauri/src/infrastructure/sqlite/inventory_repository.rs",
    "src-tauri/src/domain/inventory.rs",
    "src-tauri/Cargo.lock",
    "src-tauri/Cargo.toml",
    "src-tauri/src/infrastructure/filesystem/backup_store.rs",
    "src-tauri/src/infrastructure/filesystem/restore_transitions.rs",
    "src/commands/backup.test.ts",
    "odd/tasks/windows-desktop-build-fixes.md",
    "src-tauri/src/application/catalog/repository.rs",
    "src-tauri/tests/backup_restore.rs",
    "src-tauri/tests/command_seam.rs",
    "src-tauri/tests/inventory_sale_alerts.rs",
    "src-tauri/tests/catalog_maintenance_application.rs",
    "src-tauri/tests/catalog_maintenance_commands.rs",
    "src-tauri/tests/catalog_maintenance_domain.rs",
    "src-tauri/tests/catalog_maintenance_sqlite.rs",
    "src-tauri/tests/catalog_browse.rs",
    "odd/tasks/reports-movement-ledger.md",
    "src-tauri/capabilities/default.json",
    "src-tauri/src/application/inventory/mod.rs",
    "src-tauri/src/application/inventory/movement_ledger.rs",
    "src-tauri/src/commands/movement_ledger.rs",
    "src-tauri/src/infrastructure/sqlite/movement_ledger_repository.rs",
    "src-tauri/tests/movement_ledger.rs",
    "src-tauri/tests/movement_ledger_commands.rs",
    "src-tauri/tests/movement_ledger_export.rs",
    "src/commands/movement-ledger.ts",
    "src/commands/movement-ledger.test.ts",
    "src/ui/app-shell.ts",
    "src/ui/app.ts",
    "src/ui/reports/movement-ledger-flow.ts",
    "src/ui/reports/movement-ledger-flow.test.ts",
    "src/ui/reports/movement-ledger-screen.ts",
    "src/ui/reports/movement-ledger-screen.mounted.test.ts",
    "src-tauri/tests/post_sale_lifecycle.rs",
    "src-tauri/tests/sqlite_migrations.rs",
    "src/ui/app-shell.mounted.test.ts",
    "src/ui/inventory/inventory-screen.ts",
    "src/commands/onboarding.test.ts",
    "src/commands/dashboard.ts",
    "src/commands/dashboard.test.ts",
    "src/ui/dashboard/dashboard-screen.ts",
    "src/ui/dashboard/dashboard-screen.mounted.test.ts",
    "docs/design/dashboard-figma-handoff.md",
    "src/ui/onboarding/onboarding-form.test.ts",
    "src/ui/onboarding/onboarding-form.ts",
    "src/ui/onboarding/onboarding-flow.test.ts",
    "src/ui/onboarding/onboarding-flow.ts",
    "src/ui/onboarding/onboarding-screen.mounted.test.ts",
    "src/ui/onboarding/onboarding-screen.ts",
    "src/ui/sales/sale-screen.mounted.test.ts",
    "src/ui/sales/sale-screen.ts",
    "src/ui/sales/sale-flow.ts",
    "src/ui/sales/sale-flow.test.ts",
    "odd/tasks/checkout-stock-and-details.md",
    "src/ui/catalog/catalog-maintenance-flow.test.ts",
    "src/ui/catalog/catalog-maintenance-flow.ts",
    "src-tauri/tests/catalog_search.rs",
    "src/ui/catalog/catalog-maintenance-screen.mounted.test.ts",
    "src/ui/catalog/catalog-maintenance-screen.ts",
    "src/ui/visual-system/base-styles.mounted.test.ts",
    "src/ui/catalog/location-management-screen.mounted.test.ts",
    "src/ui/inventory/inventory-screen.mounted.test.ts",
    "src/ui/catalog/product-browser.test.ts",
    "src/ui/catalog/product-browser.ts",
    "src/ui/styles.css",
    "src/ui/visual-system/catalog-edit-dialog.ts",
    "src/ui/visual-system/confirmation-dialog.ts",
    "src/ui/visual-system/catalog-edit-dialog.mounted.test.ts",
    "src/ui/visual-system/location-picker-flow.test.ts",
    "src/ui/visual-system/location-picker-flow.ts",
    "src/ui/visual-system/location-picker.ts",
    "src/ui/w9-evidence-audit.test.ts",
    "odd/tasks/dashboard-realized-gross-profit.md",
    "odd/tasks/dashboard-partial-gross-profit.md",
    "odd/tasks/configurable-low-stock-threshold.md",
  ]);
  const unexpectedPaths = changedPaths.filter((path) => !allowedPaths.has(path));
  assert.deepEqual(unexpectedPaths, [], "unexpected protected-path drift");
  if (changedPaths.includes("src-tauri/tests/command_seam.rs")) {
    assertCommandSeamPricingAllowlist(commandSeamDiff);
  }
  if (changedPaths.includes("src-tauri/capabilities/default.json")) {
    assertReportsCapabilityAllowlist(
      read("src-tauri/capabilities/default.json"),
      readFromHead("src-tauri/capabilities/default.json"),
      capabilityDiff,
    );
  }

  if (changedPaths.includes("package.json") || changedPaths.includes("package-lock.json")) {
    assertTicket14PackageAllowlist(
      currentPackageValue,
      baselinePackageValue,
      currentLockValue,
      baselineLockValue,
    );
  }
  if (changedPaths.includes("src-tauri/src/lib.rs")) {
    if (libDiff.includes("browse_inventory_products_command")) {
      for (const marker of ["onboarding_location_schema_command", "onboarding_list_product_locations_command", "onboarding_assign_product_primary_location_command", "inventory_and_onboarding_operations_work_while_catalog_is_locked_with_safe_inventory_projection", "catalog_access_required"]) assert.match(libDiff, new RegExp(marker));
    }
    else if (libDiff.includes("catalog_access_lock_command")) {
      assert.match(libDiff, /browse_sale_products_command/);
      assert.match(libDiff, /explicit_catalog_lock_clears_the_session/);
      assert.match(libDiff, /sales_browse_and_category_filters_work_while_catalog_is_locked_without_sensitive_facts/);
      assert.match(libDiff, /list_catalog_categories_command/);
    }
    else if (libDiff.includes("catalog_access_status_command")) assert.equal(sha256(libDiff), "06e95cea1eceb4eba7c72e8924ede818364fbd7a63918c6813887fcff6f18ae4", "unexpected Catalog access backend diff");
    else if (libDiff.includes("MovementLedgerProductOptionsRequest")) assertReportsRegistrationAllowlist(libDiff, "product-options");
    else if (libDiff.includes("export_movement_ledger_command")) assertReportsRegistrationAllowlist(libDiff);
    else if (libDiff.includes("edit_category_schema_command")) assertCategorySchemaRegistrationAllowlist(libDiff);
    else if (libDiff.includes("location_schema_command")) assertProductLocationRegistrationAllowlist(libDiff);
    else if (libDiff.includes("choose_product_image_command")) assertCatalogImageRegistrationAllowlist(libDiff);
    else if (libDiff.includes("gross_profit_report_command")) assertGrossProfitCommandRegistrationDiff(libDiff);
    else if (libDiff.includes("dashboard_command")) assertDashboardRegistrationAllowlist(libDiff);
    else if (libDiff.includes("browse_products_command") || libDiff.includes("list_catalog_categories_command")) assertCatalogRegistrationAllowlist(libDiff);
    else if (libDiff.includes("Result<commands::backup::BackupDestinationSelection, String>")) assertWindowsPickerCommandDiffAllowlist(libDiff);
    else if (libDiff.includes("backup_diagnostic operation=picker_selection")) assert.equal(sha256(libDiff), "a97fbc06dbcc80d02f0d5bd8dbf5da176bdd6a24a77a3b21ed8debc0c612518e", "unexpected Windows backup-diagnostic diff");
    else if (libDiff.includes("retained_recovery_evidence_is_valid") || libDiff.includes("has_recovery_evidence")) assertBackupRecoveryStartupAllowlist(libDiff);
    else assertTicket11RegistrationAllowlist(libDiff);
  }
}

const currentPackage = parseJson(read("package.json"));
const baselinePackage = parseJson(readFromHead("package.json"));
const currentLock = parseJson(read("package-lock.json"));
const baselineLock = parseJson(readFromHead("package-lock.json"));

const referenceSizes = [
  { width: 1200, height: 800, claim: "authored desktop composition" },
  { width: 960, height: 640, claim: "authored minimum-size reflow" },
] as const;

const mountedSuites = [
  "src/ui/app-shell.mounted.test.ts",
  "src/ui/sales/sale-screen.mounted.test.ts",
  "src/ui/inventory/inventory-screen.mounted.test.ts",
  "src/ui/catalog/catalog-maintenance-screen.mounted.test.ts",
  "src/ui/sales/history-screen.mounted.test.ts",
  "src/ui/onboarding/onboarding-screen.mounted.test.ts",
  "src/ui/backup/backup-screen.mounted.test.ts",
  "src/ui/visual-system/confirmation-dialog.mounted.test.ts",
];

test("W9 records both exact reference sizes without claiming jsdom geometry", () => {
  assert.deepEqual(referenceSizes.map(({ width, height }) => [width, height]), [[1200, 800], [960, 640]]);
  for (const suite of mountedSuites) assert.ok(read(suite).length > 0, `missing mounted evidence suite: ${suite}`);
  assert.match(css, /--size-shell-sidebar:\s*208px/);
  assert.match(css, /@media \(max-width: 960px\)[\s\S]*--size-shell-sidebar:\s*176px/);
  assert.match(css, /@media \(max-width: 960px\)[\s\S]*data-ui-sale-layout[\s\S]*grid-template-columns: minmax\(0, 1fr\)/);
  assert.match(css, /@media \(max-width: 960px\)[\s\S]*data-ui-catalog-layout[\s\S]*grid-template-columns: minmax\(0, 1fr\)/);
  assert.match(css, /@media \(max-width: 960px\)[\s\S]*data-ui-backup-layout[\s\S]*grid-template-columns: minmax\(0, 1fr\)/);
  assert.match(css, /@media \(max-width: 960px\)[\s\S]*data-ui-onboarding-layout[\s\S]*grid-template-columns: minmax\(0, 1fr\)/);
  assert.match(css, /@media \(max-width: 960px\)[\s\S]*data-ui-aligned-data[\s\S]*td::before/);
});

test("W9 audits shared accessibility and state seams across every workflow", () => {
  const shell = read("src/ui/app-shell.ts");
  const controls = read("src/ui/visual-system/controls.ts");
  const dialog = read("src/ui/visual-system/confirmation-dialog.ts");
  const history = read("src/ui/sales/history-screen.ts");
  assert.match(shell, /aria-current/);
  assert.match(controls, /role = kind === "error"|role,\n/);
  assert.match(controls, /aria-busy/);
  assert.match(history, /role: "alert"|role=\"alert\"/);
  assert.match(dialog, /role: "dialog"/);
  assert.match(dialog, /aria-modal/);
  assert.match(dialog, /aria-labelledby/);
  assert.match(dialog, /aria-describedby/);
  assert.match(css, /--size-control-default:\s*44px/);
  assert.match(css, /prefers-reduced-motion: reduce/);
  assert.match(css, /forced-colors: active/);
});

test("W9 audits Spanish presentation, money, whole units, and non-color state cues", () => {
  const workflowFiles = [
    "src/ui/sales/sale-screen.ts",
    "src/ui/inventory/inventory-screen.ts",
    "src/ui/catalog/catalog-maintenance-screen.ts",
    "src/ui/sales/history-screen.ts",
    "src/ui/onboarding/onboarding-screen.ts",
    "src/ui/backup/backup-screen.ts",
  ];
  for (const file of workflowFiles) {
    const source = read(file);
    assert.match(source, /Bs|Venta|Inventario|Catálogo|Historial|Copia|Alta/);
  }
  assert.match(read("src/ui/inventory/inventory-screen.ts"), /Stock bajo: \$\{alert\.quantity\}/);
  assert.match(read("src/ui/inventory/inventory-screen.ts"), /Sin stock: \$\{alert\.quantity\}/);
  assert.match(css, /data-ui-badge/);
  assert.match(css, /font-variant-numeric: tabular-nums/);
  assert.match(read("openspec/changes/archive/2026-09-14-define-frontend-ui-ux-visual-system/design.md"), /contrast[\s\S]*validate/i);
});

test("W9 enforces the named backup-integrity candidate policy and its backend/frontend evidence", () => {
  const backupPaths = [
    "src-tauri/src/commands/backup.rs",
    "src-tauri/src/infrastructure/filesystem/backup_store.rs",
    "src-tauri/src/infrastructure/filesystem/restore_transitions.rs",
    "src-tauri/src/infrastructure/sqlite/backup.rs",
    "src/commands/backup.ts",
    "src/commands/backup.test.ts",
    "src/ui/backup/backup-flow.ts",
    "src/ui/backup/backup-flow.test.ts",
    "src/ui/backup/backup-screen.ts",
    "src/ui/backup/backup-screen.mounted.test.ts",
    "src/ui/w9-evidence-audit.test.ts",
    "odd/tasks/backup-restore-integrity-hardening.md",
  ];
  assert.doesNotThrow(() => assertW9ProtectedDiffPolicy(
    backupPaths,
    currentPackage,
    baselinePackage,
    currentLock,
    baselineLock,
    "",
  ));
  assert.throws(() => assertW9ProtectedDiffPolicy(
    [...backupPaths, "src-tauri/src/commands/unrelated.rs"],
    currentPackage,
    baselinePackage,
    currentLock,
    baselineLock,
    "",
  ), /unexpected protected-path drift/);

  const backend = read("src-tauri/src/commands/backup.rs");
  assert.match(backend, /CleanupResult::DurablyEvidencedFailure => BackupResponse::Created/);
  assert.match(backend, /CleanupResult::UnaccountedFailure => BackupResponse::error\("storage_unavailable"\)/);
  assert.match(backend, /if !reconcile_cleanup_evidence_using\(&snapshot_directory, cleanup\)[\s\S]*?\|\| !prune_artifacts_using\(&snapshot_directory/);
  for (const regression of [
    "create_backup_reconciles_stale_snapshot_evidence_before_pruning",
    "create_backup_reconciliation_failure_retains_snapshot_evidence_and_blocks",
    "create_backup_accepts_an_ordinary_empty_snapshot_directory",
  ]) assert.match(backend, new RegExp(`fn ${regression}\\(`));
  assert.doesNotMatch(backend, /let _ = remove_stage_with_evidence_using/);
  assert.match(backend, /published_backup_with_snapshot_cleanup_failure_returns_created_warning_and_evidence/);
  assert.match(backend, /restore-stage-cleanup-required\\n/);
  const frontend = read("src/commands/backup.ts");
  assert.match(frontend, /cleanup_warning: boolean/);
  assert.match(read("src/ui/backup/backup-flow.ts"), /cleanup_warning \? "La copia fue creada/);
  assert.match(read("odd/tasks/backup-restore-integrity-hardening.md"), /T9 Close Linux cleanup-accounting gaps/);
});

test("W9 binds every changed Windows desktop-fix file and task byte to exact hashes", () => {
  const trackedChanges = execFileSync("git", ["diff", "--name-only", "HEAD"], { cwd: root, encoding: "utf8" });
  const untrackedFiles = execFileSync("git", ["ls-files", "--others", "--exclude-standard"], { cwd: root, encoding: "utf8" });
  const changedPaths = changedPathsFromGitOutput(trackedChanges, untrackedFiles);
  const changedCandidates = changedPaths.filter((path) => windowsDesktopFixCandidatePaths.has(path));
  const diffs: Record<string, string | Buffer> = {};
  for (const path of changedCandidates) {
    diffs[path] = isTrackedPath(path)
      ? execFileSync("git", ["diff", "--unified=0", "HEAD", "--", path], { cwd: root, encoding: "utf8" })
      : readFileSync(resolve(root, path));
  }
  assertWindowsDesktopFixDiffAllowlist(diffs);

  const commandBackup = read("src-tauri/src/commands/backup.rs");
  const storeBackup = read("src-tauri/src/infrastructure/filesystem/backup_store.rs");
  const sqliteBackup = read("src-tauri/src/infrastructure/sqlite/backup.rs");
  const lib = read("src-tauri/src/lib.rs");
  assert.match(commandBackup, /CreateFileW[\s\S]*GENERIC_WRITE[\s\S]*FILE_FLAG_BACKUP_SEMANTICS[\s\S]*FlushFileBuffers/);
  assert.match(storeBackup, /CreateFileW[\s\S]*GENERIC_WRITE[\s\S]*FILE_FLAG_BACKUP_SEMANTICS[\s\S]*FlushFileBuffers/);
  for (const source of [commandBackup, storeBackup, sqliteBackup]) {
    assert.match(source, /#\[cfg\(all\(windows, debug_assertions\)\)\][\s\S]*backup_diagnostic operation=/);
  }
  const diagnosticCalls = `${commandBackup}\n${storeBackup}\n${sqliteBackup}`;
  for (const operation of ["snapshot_page_count", "snapshot_page_size", "cleanup_evidence_create", "cleanup_artifact_remove", "cleanup_artifact_directory_sync", "cleanup_reconcile_artifact_remove", "cleanup_reconcile_evidence_directory_sync", "publication_temp_create", "publication_finalize"]) {
    assert.match(diagnosticCalls, new RegExp(`report_backup_(?:sqlite|io|unclassified)_failure\\("${operation}"`));
  }
  const createBackupGateCalls = commandBackup.slice(commandBackup.indexOf("fn create_backup_with_publisher"), commandBackup.indexOf("pub fn prepare_restore"));
  for (const gate of ["expired_prune", "destination_token", "snapshot_prune", "snapshot_size", "snapshot_result", "state_read_snapshot", "publication", "final_cleanup"]) {
    assert.match(createBackupGateCalls, new RegExp(`report_create_backup_gate\\("${gate}"`), `missing create-backup diagnostic gate: ${gate}`);
  }
  assert.match(commandBackup, /#\[cfg\(all\(windows, debug_assertions\)\)\][\s\S]*fn report_create_backup_gate/);
  assert.match(commandBackup, /fn destination_token_class[\s\S]*fn internal_backup_result_class[\s\S]*fn storage_error_class[\s\S]*fn cleanup_result_class/);
  assert.match(read("odd/tasks/windows-desktop-build-fixes.md"), /backup_diagnostic operation=/);
  assert.match(lib, /#\[cfg\(all\(windows, debug_assertions\)\)\][\s\S]*backup_diagnostic operation=picker_selection/);
  assert.match(lib, /backup_diagnostic operation=create_backup_command phase=entry/);
  assert.match(lib, /backup_diagnostic operation=create_backup_response kind=\{kind\} code=\{code\}/);
  assert.match(lib, /fn backup_diagnostic_classification_uses_only_bounded_kinds_and_codes/);
  const candidate = Object.keys(diffs).find((path) => path !== "src/ui/w9-evidence-audit.test.ts" && path !== "src-tauri/src/lib.rs");
  if (candidate) {
    assert.throws(
      () => assertWindowsDesktopFixDiffAllowlist({ ...diffs, [candidate]: `${diffs[candidate]}\\n+arbitrary Windows-fix drift` }),
      /unexpected Windows desktop-fix diff content/,
    );
  }
  assert.throws(
    () => assertWindowsDesktopFixDiffAllowlist({ "src-tauri/Cargo.toml": "+unallowlisted Windows-fix drift" }),
    /unexpected Windows desktop-fix diff content/,
  );
});

test("W9 binds every changed backup-integrity file to its exact unified-zero diff", () => {
  const trackedChanges = execFileSync("git", ["diff", "--name-only", "HEAD"], { cwd: root, encoding: "utf8" });
  const untrackedFiles = execFileSync("git", ["ls-files", "--others", "--exclude-standard"], { cwd: root, encoding: "utf8" });
  const changedPaths = changedPathsFromGitOutput(trackedChanges, untrackedFiles);
  const changedCandidates = changedPaths.filter((path) => backupIntegrityCandidatePaths.has(path));
  const diffs: Record<string, string | Buffer> = {};
  for (const path of changedCandidates) {
    diffs[path] = isTrackedPath(path)
      ? execFileSync("git", ["diff", "--unified=0", "HEAD", "--", path], { cwd: root, encoding: "utf8" })
      : readFileSync(resolve(root, path));
  }
  assertBackupIntegrityDiffAllowlist(diffs);
  const candidate = Object.keys(diffs).find((path) => path !== "src/ui/w9-evidence-audit.test.ts" && path !== "src-tauri/src/lib.rs");
  if (candidate) {
    assert.throws(
      () => assertBackupIntegrityDiffAllowlist({ ...diffs, [candidate]: `${diffs[candidate]}\n+arbitrary candidate drift` }),
      /unexpected backup-integrity diff content/,
    );
  }
  assert.throws(
    () => assertBackupIntegrityDiffAllowlist({ "src-tauri/src/commands/backup.rs": "+unallowlisted candidate" }),
    /unexpected backup-integrity diff content/,
  );
});

test("W9 preserves the committed backup-integrity task evidence", () => {
  const path = "odd/tasks/backup-restore-integrity-hardening.md";
  assert.equal(isTrackedPath(path), true);
  const contents = Buffer.from(readFromHead(path));
  assert.doesNotThrow(() => assertBackupIntegrityDiffAllowlist({ [path]: contents }));
  assert.match(contents.toString("utf8"), /T9 Close Linux cleanup-accounting gaps/);
});

test("W9 allows only the bounded owned-Result Windows picker-command diff", () => {
  const allowedDiff = [
    "-    commands: tauri::State<'_, Mutex<commands::backup::BackupCommandState>>,",
    "-) -> commands::backup::BackupDestinationSelection {",
    "+) -> Result<commands::backup::BackupDestinationSelection, String> {",
    "+    let app_handle = window.app_handle().clone();",
    "+    drop(window);",
    "+    let picker_app_handle = app_handle.clone();",
    "-            let _ = window;",
    "+            let _ = picker_app_handle;",
    "-                .app_handle()",
    "+            picker_app_handle",
    "-        return commands::backup::BackupDestinationSelection::Cancelled;",
    "+        return Ok(commands::backup::BackupDestinationSelection::Cancelled);",
    "+    let commands = app_handle.state::<Mutex<commands::backup::BackupCommandState>>();",
    "-        return commands::backup::BackupDestinationSelection::Error {",
    "+        return Ok(commands::backup::BackupDestinationSelection::Error {",
    "-        };",
    "+        });",
    "-    commands.select_backup_destination(path)",
    "+    Ok(commands.select_backup_destination(path))",
    "-    commands: tauri::State<'_, Mutex<commands::backup::BackupCommandState>>,",
    "-) -> commands::backup::RestoreSourceSelection {",
    "+) -> Result<commands::backup::RestoreSourceSelection, String> {",
    "+    let app_handle = window.app_handle().clone();",
    "+    drop(window);",
    "+    let picker_app_handle = app_handle.clone();",
    "-            let _ = window;",
    "+            let _ = picker_app_handle;",
    "-                .app_handle()",
    "+            picker_app_handle",
    "-        return commands::backup::RestoreSourceSelection::Cancelled;",
    "+        return Ok(commands::backup::RestoreSourceSelection::Cancelled);",
    "+    let commands = app_handle.state::<Mutex<commands::backup::BackupCommandState>>();",
    "-        return commands::backup::RestoreSourceSelection::Error {",
    "+        return Ok(commands::backup::RestoreSourceSelection::Error {",
    "-        };",
    "+        });",
    "-    commands.select_restore_source(path)",
    "+    Ok(commands.select_restore_source(path))",
  ].join("\n");
  assert.doesNotThrow(() => assertWindowsPickerCommandDiffAllowlist(allowedDiff));
  assert.throws(
    () => assertWindowsPickerCommandDiffAllowlist(`${allowedDiff}\n+    native_runtime_drift();`),
    /unexpected Windows picker-command diff drift/,
  );
});

test("W9 binds the exact T3 gross-profit candidate paths, diff, and task bytes", () => {
  const trackedChanges = execFileSync("git", ["diff", "--name-only", "HEAD"], { cwd: root, encoding: "utf8" });
  const untrackedFiles = execFileSync("git", ["ls-files", "--others", "--exclude-standard"], { cwd: root, encoding: "utf8" });
  const changedPaths = changedPathsFromGitOutput(trackedChanges, untrackedFiles);
  const t3Sources = changedPaths.filter((path) => grossProfitReportCandidatePaths.includes(path) && path !== "odd/tasks/catalog-access-and-profit-report.md" && path !== "src/ui/w9-evidence-audit.test.ts");
  if (t3Sources.length > 0 && !changedPaths.includes("odd/tasks/reporte-ganancia-operaciones-pdf.md")) assertGrossProfitReportCandidate(changedPaths);
});

test("W9 binds the exact Catalog access candidate diff and untracked task bytes", () => {
  const trackedChanges = execFileSync("git", ["diff", "--name-only", "HEAD"], { cwd: root, encoding: "utf8" });
  const untrackedFiles = execFileSync("git", ["ls-files", "--others", "--exclude-standard"], { cwd: root, encoding: "utf8" });
  const changedPaths = changedPathsFromGitOutput(trackedChanges, untrackedFiles);
  const candidatePathSet = new Set(catalogAccessCandidatePaths);
  const changedCandidates = changedPaths.filter((path) => candidatePathSet.has(path));
  const t3Sources = changedPaths.filter((path) => grossProfitReportCandidatePaths.includes(path) && path !== "odd/tasks/catalog-access-and-profit-report.md" && path !== "src/ui/w9-evidence-audit.test.ts");
  if (t3Sources.length > 0 && !changedPaths.includes("odd/tasks/reporte-ganancia-operaciones-pdf.md")) {
    assertGrossProfitReportCandidate(changedPaths);
    return;
  }
  if (changedPaths.includes("odd/tasks/reporte-ganancia-operaciones-pdf.md")) return;
  const catalogAccessSources = changedCandidates.filter((path) => path !== "odd/tasks/catalog-access-and-profit-report.md" && path !== "src/ui/w9-evidence-audit.test.ts");
  if (catalogAccessSources.length === 0) {
    const responsiveCandidates = changedPaths.filter((path) => catalogResponsiveTableCandidatePaths.includes(path));
    assert.deepEqual(responsiveCandidates, [...catalogResponsiveTableCandidatePaths].sort());
    const exactDiff = execFileSync("git", ["diff", "--unified=0", "HEAD", "--", ...catalogResponsiveTableCandidatePaths], { cwd: root, encoding: "utf8" });
    const normalizedDiff = exactDiff
      .replaceAll(/^index \S+\.\.\S+.*$/gm, "index <SELF_INDEX>")
      .replaceAll(/^([+-])const catalogResponsiveTableUnifiedDiffSha256 = "[^"]+";$/gm, '$1const catalogResponsiveTableUnifiedDiffSha256 = "<SELF_HASH>";');
    assert.equal(sha256(normalizedDiff), catalogResponsiveTableUnifiedDiffSha256, "unexpected Catalog responsive-table candidate diff");
    return;
  }
  assert.deepEqual(changedCandidates, [...catalogAccessCandidatePaths].sort());

  const trackedCandidates = catalogAccessCandidatePaths.filter((path) => isTrackedPath(path));
  const exactDiff = execFileSync("git", ["diff", "--unified=0", "HEAD", "--", ...trackedCandidates], { cwd: root, encoding: "utf8" });
  const normalizedDiff = exactDiff
    .replaceAll(/^index \S+\.\.\S+.*$/gm, "index <SELF_INDEX>")
    .replaceAll(/^([+-])const catalogAccessUnifiedDiffSha256 = "[^"]+";$/gm, '$1const catalogAccessUnifiedDiffSha256 = "<SELF_HASH>";')
    .replaceAll(/^([+-])(\s*"src\/ui\/w9-evidence-audit\.test\.ts": ")[^"]+(",)$/gm, '$1$2<SELF_DIFF_SHA256>$3');
  assert.equal(sha256(normalizedDiff), catalogAccessUnifiedDiffSha256, "unexpected Catalog access candidate diff");

  assert.match(read("odd/tasks/catalog-access-and-profit-report.md"), /Sales remains operational while Catalog is locked/);
  assert.match(read("odd/tasks/catalog-access-and-profit-report.md"), /explicitly chooses "Bloquear catálogo"/);
});

test("W9 allows clean trees, ticket 14 metadata, and the bounded ticket 11 seam", () => {
  const trackedChanges = execFileSync("git", ["diff", "--name-only", "HEAD"], { cwd: root, encoding: "utf8" });
  const untrackedFiles = execFileSync("git", ["ls-files", "--others", "--exclude-standard"], { cwd: root, encoding: "utf8" });
  const changedPaths = excludePreExistingUnrelatedOddTaskBaseline(
    changedPathsFromGitOutput(trackedChanges, untrackedFiles),
  );
  const libDiff = execFileSync("git", ["diff", "--unified=0", "HEAD", "--", "src-tauri/src/lib.rs"], { cwd: root, encoding: "utf8" });
  const commandSeamDiff = execFileSync("git", ["diff", "--unified=0", "HEAD", "--", "src-tauri/tests/command_seam.rs"], { cwd: root, encoding: "utf8" });
  const capabilityDiff = execFileSync("git", ["diff", "--unified=0", "HEAD", "--", "src-tauri/capabilities/default.json"], { cwd: root, encoding: "utf8" });
  assertW9ProtectedDiffPolicy(
    changedPaths,
    currentPackage,
    baselinePackage,
    currentLock,
    baselineLock,
    libDiff,
    commandSeamDiff,
    capabilityDiff,
  );
  const uiSources = mountedSuites.map((suite) => read(suite)).join("\n");
  assert.doesNotMatch(uiSources, /https?:\/\/|cdn\.|innerHTML/);
});

test("W9 allows only the diagnosed command-seam pricing contract-test drift", () => {
  const allowedDiff = [
    "-                captured_unit_price_centavos: 4_000,",
    "+                captured_unit_price_centavos: 5_000,",
    "-                final_unit_price_centavos: None,",
    "+                final_unit_price_centavos: Some(5_000),",
    "-                qr_applied_centavos: Some(4_000),",
    "+                qr_applied_centavos: Some(5_000),",
    "-    assert_eq!(summary.lines[0].unit_price_centavos, 4_000);",
    "+    assert_eq!(summary.lines[0].unit_price_centavos, 5_000);",
    "+    assert_eq!(product.purchase_price_centavos, 3_000);",
    "+    assert_eq!(product.sale_price_centavos, 5_000);",
    "+    assert_eq!(product.minimum_sale_price_centavos, 4_000);",
    "+fn onboarded_product_searches_and_sells_at_its_backend_sale_price() {",
  ].join("\n");
  assert.doesNotThrow(() => assertCommandSeamPricingAllowlist(allowedDiff));
  assert.throws(
    () => assertCommandSeamPricingAllowlist(`${allowedDiff}\n+ fn unrelated_runtime_change() { native_runtime_drift(); }`),
    /unexpected command-seam contract-test drift/,
  );
});

test("W9 allows the authorized catalog edit dialog accessible-label test correction", () => {
  assert.doesNotThrow(() => assertW9ProtectedDiffPolicy(
    ["src/ui/visual-system/catalog-edit-dialog.mounted.test.ts"],
    currentPackage,
    baselinePackage,
    currentLock,
    baselineLock,
    "",
  ));
  assert.throws(
    () => assertW9ProtectedDiffPolicy(
      ["src/ui/visual-system/unrelated.mounted.test.ts"],
      currentPackage,
      baselinePackage,
      currentLock,
      baselineLock,
      "",
    ),
    /unexpected protected-path drift/,
  );
});

test("W9 excludes only the fixed unrelated ODD baseline and keeps candidate paths live", () => {
  const baselinePaths = [
    "odd/tasks/configurable-product-locations.md",
    "odd/tasks/developer-toolchain-rc-upgrade.md",
    "odd/tasks/inventory-pr-delivery.md",
    "odd/tasks/inventory-redesign.md",
    "odd/tasks/product-status-documentation.md",
    "odd/tasks/sales-explicit-product-details.md",
  ];
  assert.deepEqual(excludePreExistingUnrelatedOddTaskBaseline(baselinePaths), []);
  assert.deepEqual(
    excludePreExistingUnrelatedOddTaskBaseline([
      ...baselinePaths,
      "odd/tasks/dashboard-realized-gross-profit.md",
      "odd/tasks/new-unrelated-task.md",
    ]),
    ["odd/tasks/dashboard-realized-gross-profit.md", "odd/tasks/new-unrelated-task.md"],
  );
  assert.doesNotThrow(() => assertW9ProtectedDiffPolicy(
    ["odd/tasks/dashboard-realized-gross-profit.md"],
    currentPackage,
    baselinePackage,
    currentLock,
    baselineLock,
    "",
  ));
  assert.throws(
    () => assertW9ProtectedDiffPolicy(
      excludePreExistingUnrelatedOddTaskBaseline(["odd/tasks/new-unrelated-task.md"]),
      currentPackage,
      baselinePackage,
      currentLock,
      baselineLock,
      "",
    ),
    /unexpected protected-path drift/,
  );
});

test("W9 includes untracked paths in the exact protected-path allowlist", () => {
  const intendedUntrackedPaths = changedPathsFromGitOutput(
    "",
    [
      "src-tauri/src/infrastructure/sqlite/migrations/0021_sale_line_cost_snapshot.sql",
      "src-tauri/tests/sale_cost_snapshot.rs",
    ].join("\n"),
  );
  assert.deepEqual(intendedUntrackedPaths, [
    "src-tauri/src/infrastructure/sqlite/migrations/0021_sale_line_cost_snapshot.sql",
    "src-tauri/tests/sale_cost_snapshot.rs",
  ]);
  assert.doesNotThrow(() => assertW9ProtectedDiffPolicy(
    intendedUntrackedPaths,
    currentPackage,
    baselinePackage,
    currentLock,
    baselineLock,
    "",
  ));

  const unrelatedUntrackedPath = changedPathsFromGitOutput("", "src-tauri/tests/unrelated_protected_test.rs");
  assert.throws(
    () => assertW9ProtectedDiffPolicy(
      unrelatedUntrackedPath,
      currentPackage,
      baselinePackage,
      currentLock,
      baselineLock,
      "",
    ),
    /unexpected protected-path drift/,
  );
});

test("W9 rejects arbitrary protected-path and package-lock drift", () => {
  const ticket11Diff = ticket11RegistrationMarkers.map((marker) => `+ ${marker}`).join("\n");
  for (const changedPaths of [
    ["src-tauri/src/main.rs"],
    ["src-tauri/src/application/catalog.rs"],
    ["src-tauri/build.rs"],
    ["src-tauri/src/commands/inventory.rs"],
    ["src/commands/inventory.ts"],
    ["src/ui/unrelated-feature.ts"],
  ]) {
    assert.throws(
      () => assertW9ProtectedDiffPolicy(changedPaths, currentPackage, baselinePackage, currentLock, baselineLock, ticket11Diff),
      /unexpected protected-path drift/,
      changedPaths.join(", "),
    );
  }

  const driftedLock = clone(currentLock);
  driftedLock.packages["node_modules/w9-unapproved-drift"] = { version: "0.0.0" };
  assert.throws(
    () => assertW9ProtectedDiffPolicy(["package-lock.json"], currentPackage, baselinePackage, driftedLock, baselineLock, ""),
    /unexpected package-lock drift/,
  );

  assert.throws(
    () => assertTicket11RegistrationAllowlist("+ fn unrelated_runtime_change() { native_runtime_drift(); }"),
    /missing ticket-11 marker|unexpected src-tauri\/src\/lib\.rs drift/,
  );
});

test("W9 allows only the three authorized low-stock command-seam lines", () => {
  const allowedDiff = [
    "+                captured_unit_price_centavos: 5_000,",
    "+                qr_applied_centavos: Some(5_000),",
    "+    assert_eq!(summary.lines[0].unit_price_centavos, 5_000);",
    "+fn onboarded_product_searches_and_sells_at_its_backend_sale_price() {",
    "+            low_stock_threshold: None,",
    "+            low_stock_threshold: None,",
    String.raw`+        "attribute_values", "available_quantity", "category_id", "category_name", "minimum_sale_price_centavos", "name", "primary_location_code", "product_id", "purchase_price_centavos", "revision", "sale_price_centavos", "sku"`,
  ].join("\n");
  assert.doesNotThrow(() => assertCommandSeamPricingAllowlist(allowedDiff));
  assert.throws(
    () => assertCommandSeamPricingAllowlist(`${allowedDiff}\n+            low_stock_threshold: Some(2),`),
    /unexpected command-seam contract-test drift/,
  );
  assert.throws(
    () => assertW9ProtectedDiffPolicy(
      ["src-tauri/tests/command_seam_unrelated.rs"],
      currentPackage,
      baselinePackage,
      currentLock,
      baselineLock,
      "",
    ),
    /unexpected protected-path drift/,
  );
});

test("W9 allows only the exact configurable low-stock threshold paths", () => {
  const featurePaths = [
    "odd/tasks/configurable-low-stock-threshold.md",
    "src-tauri/src/infrastructure/sqlite/migrations/0022_product_low_stock_threshold.sql",
    "src-tauri/src/domain/inventory.rs",
    "src-tauri/src/infrastructure/sqlite/inventory_repository.rs",
  ];
  for (const changedPath of featurePaths) {
    assert.doesNotThrow(
      () => assertW9ProtectedDiffPolicy([changedPath], currentPackage, baselinePackage, currentLock, baselineLock, ""),
      changedPath,
    );
  }

  assert.doesNotThrow(
    () => assertW9ProtectedDiffPolicy(["src-tauri/tests/inventory_sale_alerts.rs"], currentPackage, baselinePackage, currentLock, baselineLock, ""),
    "src-tauri/tests/inventory_sale_alerts.rs",
  );

  for (const changedPath of [
    "src-tauri/tests/inventory_sale_alert.rs",
    "src-tauri/tests/inventory_sale_alerts_extra.rs",
    "src-tauri/tests/unrelated_inventory_sale_alerts.rs",
    "src-tauri/tests/unrelated_feature.rs",
    "odd/tasks/configurable-low-stock-threshold-related.md",
    "src-tauri/src/infrastructure/sqlite/migrations/0023_product_low_stock_threshold.sql",
    "src-tauri/src/domain/inventory_unrelated.rs",
    "src-tauri/src/infrastructure/sqlite/unrelated_inventory_repository.rs",
  ]) {
    assert.throws(
      () => assertW9ProtectedDiffPolicy([changedPath], currentPackage, baselinePackage, currentLock, baselineLock, ""),
      /unexpected protected-path drift/,
      changedPath,
    );
  }
});

test("W9 allows only the exact Reports Movement Ledger paths", () => {
  const reportsPaths = [
    "odd/tasks/reports-movement-ledger.md",
    "src-tauri/capabilities/default.json",
    "src-tauri/src/application/inventory/mod.rs",
    "src-tauri/src/application/inventory/movement_ledger.rs",
    "src-tauri/src/commands/movement_ledger.rs",
    "src-tauri/src/infrastructure/sqlite/movement_ledger_repository.rs",
    "src-tauri/tests/movement_ledger.rs",
    "src-tauri/tests/movement_ledger_commands.rs",
    "src-tauri/tests/movement_ledger_export.rs",
    "src/commands/movement-ledger.ts",
    "src/commands/movement-ledger.test.ts",
    "src/ui/app-shell.ts",
    "src/ui/app.ts",
    "src/ui/reports/movement-ledger-flow.ts",
    "src/ui/reports/movement-ledger-flow.test.ts",
    "src/ui/reports/movement-ledger-screen.ts",
    "src/ui/reports/movement-ledger-screen.mounted.test.ts",
  ];
  for (const changedPath of reportsPaths) {
    assert.doesNotThrow(
      () => assertW9ProtectedDiffPolicy(
        [changedPath], currentPackage, baselinePackage, currentLock, baselineLock, "", "",
        "",
      ),
      changedPath,
    );
  }

  for (const changedPath of [
    "src-tauri/capabilities/other.json",
    "src-tauri/capabilities",
    "odd/tasks/reports-movement-ledger-related.md",
    "odd/tasks/unrelated-reports.md",
    "src-tauri/src/application/inventory/other_ledger.rs",
    "src-tauri/src/commands/movement_ledger_extra.rs",
    "src-tauri/tests/movement_ledger_extra.rs",
    "src/ui/reports/unrelated-report.ts",
    "src-tauri/src/application/inventory",
    "src-tauri/src/commands",
    "src-tauri/tests",
    "src/ui/reports",
    "odd/tasks",
  ]) {
    assert.throws(
      () => assertW9ProtectedDiffPolicy([changedPath], currentPackage, baselinePackage, currentLock, baselineLock, ""),
      /unexpected protected-path drift/,
      changedPath,
    );
  }
});

test("W9 validates the Reports capability addition against its explicit pre-Reports baseline", () => {
  const committedBaseline = parseJson(readFromHead("src-tauri/capabilities/default.json"));
  const baseline = JSON.stringify({
    ...committedBaseline,
    permissions: committedBaseline.permissions.filter((permission: string) => permission !== "dialog:allow-save"),
  }, null, 2);
  const current = readFromHead("src-tauri/capabilities/default.json");
  const allowedDiff = [
    '-  "permissions": ["core:default", "dialog:allow-open"]',
    '+  "permissions": ["core:default", "dialog:allow-open", "dialog:allow-save"]',
  ].join("\n");
  assert.doesNotThrow(() => assertReportsCapabilityAllowlist(current, baseline, allowedDiff));

  for (const [changedCurrent, changedDiff] of [
    [baseline, allowedDiff],
    [current.replace("dialog:allow-open", "dialog:allow-open-extra"), allowedDiff],
    [current.replace("\"dialog:allow-save\"]", "\"dialog:allow-save\", \"dialog:allow-close\"]"), allowedDiff],
    [current.replace("\"core:default\", \"dialog:allow-open\"", "\"dialog:allow-open\", \"core:default\""), allowedDiff],
    [current, `${allowedDiff}\n-  \"description\": \"Main window permissions for native backup and restore dialogs.\"\n+  \"description\": \"Changed unrelated description.\"`],
    [current, `${allowedDiff}\n-  \"description\": \"Main window permissions for native backup and restore dialogs.\"\n+  \"windows\": [\"main\"]\n-  \"windows\": [\"main\"]\n+  \"description\": \"Main window permissions for native backup and restore dialogs.\"`],
    [current, `${allowedDiff}\n+  \"extra\": true`],
  ] as const) {
    assert.throws(
      () => assertReportsCapabilityAllowlist(changedCurrent, baseline, changedDiff),
      /Reports may only append|unexpected Reports capability diff/,
    );
  }

  assert.doesNotThrow(() => assertReportsCapabilityAllowlist(
    read("src-tauri/capabilities/default.json"),
    current,
    "",
  ));
  assert.throws(
    () => assertReportsCapabilityAllowlist(
      current.replace("dialog:allow-save", "dialog:allow-save-extra"),
      current,
      '+  "permissions": ["core:default", "dialog:allow-open", "dialog:allow-save-extra"]',
    ),
    /unexpected uncommitted Reports capability drift/,
  );
});

test("W9 allows only the repaired async Reports export command shape", () => {
  const allowedDiff = [
    "+ list_movement_ledger_command,",
    "+ list_movement_ledger_product_options_command,",
    "+ export_movement_ledger_command,",
    "+ app_handle: tauri::AppHandle<R>,",
    "+ ) -> Result<commands::movement_ledger::MovementLedgerExportResponse, String> {",
    "+ let _ = (app_handle, request);",
    "+ Ok(commands::movement_ledger::MovementLedgerExportResponse::Cancelled)",
    "+ let state = app_handle.state::<AppState>();",
    "+ let response = state.with_read(|connection| Ok(commands::movement_ledger::export_movement_ledger(",
    "+ return Ok(commands::movement_ledger::MovementLedgerExportResponse::Cancelled);",
    "+ Ok(response)",
    "- state: tauri::State<'_, AppState>,",
    "- window: tauri::WebviewWindow<R>,",
    "- ) -> commands::movement_ledger::MovementLedgerExportResponse {",
    "- let _ = (state, window, request);",
    "- commands::movement_ledger::MovementLedgerExportResponse::Cancelled",
    "- return commands::movement_ledger::MovementLedgerExportResponse::Cancelled;",
    "- let app_handle = window.app_handle().clone();",
    "- drop(window);",
    "- state.with_read(|connection| Ok(commands::movement_ledger::export_movement_ledger(",
    "- ))",
  ].join("\n");
  assert.doesNotThrow(() => assertReportsRegistrationAllowlist(allowedDiff));
  for (const invalidLine of [
    "+ state: tauri::State<'_, AppState>,",
    "+ window: tauri::WebviewWindow<R>,",
    "+ ) -> commands::movement_ledger::MovementLedgerExportResponse {",
    "+ unrelated_reports_command,",
    "+ state.with_write(|connection| mutate_reports(connection))",
    "+ fn unrelated_runtime_change() { native_runtime_drift(); }",
  ]) {
    assert.throws(
      () => assertReportsRegistrationAllowlist(`${allowedDiff}\n${invalidLine}`),
      /unexpected Reports command registration drift/,
      invalidLine,
    );
  }
});

test("W9 routes the bounded Reports product-options wrapper to its narrow validator", () => {
  const boundedProductOptionsDiff = [
    "+ request: commands::movement_ledger::MovementLedgerProductOptionsRequest,",
    "+ state.with_read(|connection| Ok(commands::movement_ledger::list_movement_ledger_product_options(connection, request)))",
    "- state.with_read(|connection| Ok(commands::movement_ledger::list_movement_ledger_product_options(connection)))",
  ].join("\n");
  assert.match(boundedProductOptionsDiff, /MovementLedgerProductOptionsRequest/);
  assert.doesNotThrow(() => assertReportsRegistrationAllowlist(boundedProductOptionsDiff, "product-options"));
  for (const invalidLine of [
    "+ request: commands::catalog::BrowseProductsRequest,",
    "+ page_size: u32,",
    "+ state.with_write(|connection| mutate_reports(connection))",
  ]) {
    assert.throws(
      () => assertReportsRegistrationAllowlist(`${boundedProductOptionsDiff}\n${invalidLine}`, "product-options"),
      /unexpected Reports command registration drift/,
      invalidLine,
    );
  }
  assert.throws(
    () => assertReportsRegistrationAllowlist(boundedProductOptionsDiff, "export"),
    /missing Reports export command marker/,
  );
});

test("W9 allows only the exact Dashboard and gross-profit paths", () => {
  const dashboardPaths = [
    "src-tauri/src/application/mod.rs",
    "src-tauri/src/commands/mod.rs",
    "src-tauri/src/infrastructure/sqlite/mod.rs",
    "src-tauri/src/application/reporting/mod.rs",
    "src-tauri/src/infrastructure/sqlite/dashboard_repository.rs",
    "src-tauri/src/infrastructure/sqlite/sale_repository.rs",
    "src-tauri/src/infrastructure/sqlite/migrations/0021_sale_line_cost_snapshot.sql",
    "src-tauri/tests/dashboard_reporting.rs",
    "src-tauri/tests/sale_cost_snapshot.rs",
    "src/commands/dashboard.ts",
    "src/commands/dashboard.test.ts",
    "src/ui/dashboard/dashboard-screen.ts",
    "src/ui/dashboard/dashboard-screen.mounted.test.ts",
    "docs/design/dashboard-figma-handoff.md",
    "odd/tasks/dashboard-partial-gross-profit.md",
  ];
  for (const changedPath of dashboardPaths) {
    assert.doesNotThrow(
      () => assertW9ProtectedDiffPolicy([changedPath], currentPackage, baselinePackage, currentLock, baselineLock, ""),
      changedPath,
    );
  }

  for (const changedPath of [
    "src-tauri/src/application/reporting/other_report.rs",
    "src-tauri/src/commands/unrelated_dashboard.rs",
    "src-tauri/src/infrastructure/sqlite/other_dashboard_repository.rs",
    "src-tauri/src/infrastructure/sqlite/migrations/0022_unrelated.sql",
    "odd/tasks/dashboard-partial-gross-profit-related.md",
    "odd/tasks/unrelated-dashboard-partial-gross-profit.md",
    "src-tauri/src/infrastructure/sqlite/migrations/0018_unrelated.sql",
    "src-tauri/src/infrastructure/sqlite/another_repository.rs",
    "src-tauri/tests/catalog_images.rs",
    "src-tauri/src/application",
    "src-tauri/src/commands",
    "src-tauri/src/infrastructure/sqlite",
  ]) {
    assert.throws(
      () => assertW9ProtectedDiffPolicy(
        [changedPath],
        currentPackage,
        baselinePackage,
        currentLock,
        baselineLock,
        "",
      ),
      /unexpected protected-path drift/,
      changedPath,
    );
  }
});

test("W9 parses Catalog registration diffs by actual newline and rejects unrelated drift", () => {
  const allowedCatalogDiff = [
    "+ browse_products_command,",
    "+ list_catalog_categories_command,",
  ].join("\n");
  assert.doesNotThrow(() => assertCatalogRegistrationAllowlist(allowedCatalogDiff));
  assert.throws(
    () => assertCatalogRegistrationAllowlist(`${allowedCatalogDiff}\n+ fn unrelated_catalog_runtime_change() { native_runtime_drift(); }`),
    /unexpected Catalog command registration drift/,
  );
});

test("W9 allows only the diagnosed Catalog image paths and rejects unrelated paths", () => {
  const catalogImagePaths = [
    "src-tauri/src/infrastructure/sqlite/migrations/0016_product_images.sql",
    "src-tauri/src/infrastructure/sqlite/migrations/0017_product_image_thumbnails.sql",
    "src-tauri/src/application/catalog/mod.rs",
    "src-tauri/src/infrastructure/sqlite/catalog_repository.rs",
  ];
  for (const changedPath of catalogImagePaths) {
    assert.doesNotThrow(
      () => assertW9ProtectedDiffPolicy([changedPath], currentPackage, baselinePackage, currentLock, baselineLock, ""),
      changedPath,
    );
  }

  for (const changedPath of [
    "src-tauri/src/infrastructure/sqlite/migrations/0018_unrelated.sql",
    "src-tauri/src/infrastructure/sqlite/another_repository.rs",
    "src-tauri/tests/catalog_images.rs",
  ]) {
    assert.throws(
      () => assertW9ProtectedDiffPolicy([changedPath], currentPackage, baselinePackage, currentLock, baselineLock, ""),
      /unexpected protected-path drift/,
      changedPath,
    );
  }
});

test("W9 allows only the exact Dashboard command registration diff", () => {
  const allowedDashboardDiff = [
    "+ dashboard_command,",
    '+#[cfg(feature = "desktop")]',
    "+#[tauri::command]",
    "+fn dashboard_command(",
    "+    state: tauri::State<AppState>,",
    "+    request: commands::dashboard::DashboardRequest,",
    "+) -> commands::dashboard::DashboardResponse {",
    "+    state",
    "+        .with_read(|connection| Ok(commands::dashboard::dashboard(connection, request)))",
    "+        .unwrap_or_else(|_| commands::dashboard::DashboardResponse::Error(commands::dashboard::persistence_failure()))",
    "+}",
    "+    #[test]",
    "+    fn registers_read_only_dashboard_command_at_the_tauri_command_seam() {",
    "+        let (app, window) = test_window();",
    "+        let before = app.state::<AppState>().with_read(|connection| Ok(snapshot(connection))).unwrap();",
    "+        assert!(get_ipc_response(&window, request_with(\"dashboard_command\", serde_json::json!({",
    '+            "today_from_utc": "2024-03-10T05:00:00Z",',
    '+            "today_to_exclusive_utc": "2024-03-11T04:00:00Z",',
    '+            "month_from_utc": "2024-03-01T05:00:00Z",',
    '+            "month_to_exclusive_utc": "2024-04-01T04:00:00Z"',
    "+        }))).is_ok());",
    "+        assert_eq!(app.state::<AppState>().with_read(|connection| Ok(snapshot(connection))).unwrap(), before);",
    "+    }",
  ].join("\n");
  assert.doesNotThrow(() => assertDashboardRegistrationAllowlist(allowedDashboardDiff));
  assert.throws(
    () => assertDashboardRegistrationAllowlist(`${allowedDashboardDiff}\n+ fn dashboard_runtime_change() { native_runtime_drift(); }`),
    /unexpected Dashboard command registration drift/,
  );
});

test("W9 allows only the exact Sales screen path for checkout thumbnail caching", () => {
  assert.doesNotThrow(() => assertW9ProtectedDiffPolicy(
    ["src/ui/sales/sale-screen.ts"],
    currentPackage,
    baselinePackage,
    currentLock,
    baselineLock,
    "",
  ));
  for (const nearPath of [
    "src/ui/sales/sale-screen-extra.ts",
    "src/ui/sales/sale-screen.test.ts",
    "src/ui/sales/sale-screen.mounted.extra.test.ts",
  ]) {
    assert.throws(
      () => assertW9ProtectedDiffPolicy([nearPath], currentPackage, baselinePackage, currentLock, baselineLock, ""),
      /unexpected protected-path drift/,
      nearPath,
    );
  }
});

test("W9 allows only the exact checkout feature paths and rejects nearby paths", () => {
  const checkoutPaths = [
    "odd/tasks/checkout-stock-and-details.md",
    "src/ui/sales/sale-flow.ts",
    "src/ui/sales/sale-flow.test.ts",
  ];
  for (const path of checkoutPaths) {
    assert.doesNotThrow(
      () => assertW9ProtectedDiffPolicy([path], currentPackage, baselinePackage, currentLock, baselineLock, ""),
      path,
    );
  }

  for (const nearPath of [
    "odd/tasks/checkout-stock-and-details-related.md",
    "odd/tasks/unrelated-checkout-stock-and-details.md",
    "src/ui/sales/sale-flow-extra.ts",
    "src/ui/sales/sale-flow.test-extra.ts",
    "src/ui/sales/sale-flow.unrelated.test.ts",
  ]) {
    assert.throws(
      () => assertW9ProtectedDiffPolicy([nearPath], currentPackage, baselinePackage, currentLock, baselineLock, ""),
      /unexpected protected-path drift/,
      nearPath,
    );
  }
});

test("W9 rejects appended text adjacent to an allowed ticket 11 marker", () => {
  const allowedRegistrationDiff = [
    "+ fn command_builder<R: Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {",
    "+ command_builder(builder.plugin(tauri_plugin_dialog::init()))",
    "+ choose_backup_destination_command,",
    "+ choose_restore_source_command,",
    "+ create_backup_command,",
    "+ prepare_restore_command,",
    "+ confirm_restore_command",
  ].join("\n");
  assert.doesNotThrow(() => assertTicket11RegistrationAllowlist(allowedRegistrationDiff));
  assert.throws(
    () => assertTicket11RegistrationAllowlist(
      `${allowedRegistrationDiff}\n+ choose_backup_destination_command, // arbitrary appended production text`,
    ),
    /unexpected src-tauri\/src\/lib\.rs drift/,
  );

  const allowedCatalogImageDiff = [
    "+ choose_product_image_command,",
    "+ remove_product_image_command,",
    "+ catalog_product_image_thumbnail_command,",
  ].join("\n");
  assert.doesNotThrow(() => assertCatalogImageRegistrationAllowlist(allowedCatalogImageDiff));
  assert.throws(
    () => assertCatalogImageRegistrationAllowlist(`${allowedCatalogImageDiff}\n+ fn unrelated_catalog_runtime_change() { native_runtime_drift(); }`),
    /unexpected Catalog image command registration drift/,
  );
});

test("W9 rejects non-allowlisted package metadata drift", () => {
  for (const drift of [
    (packageJson: Record<string, any>) => { packageJson.dependencies.react = "^18.3.2"; },
    (packageJson: Record<string, any>) => { packageJson.scripts.build = "vite --mode unexpected-drift"; },
  ]) {
    const driftedPackage = clone(currentPackage);
    drift(driftedPackage);
    assert.throws(
      () => assertW9ProtectedDiffPolicy(["package.json"], driftedPackage, baselinePackage, currentLock, baselineLock, ""),
      /unexpected package\.json drift/,
    );
  }
});
