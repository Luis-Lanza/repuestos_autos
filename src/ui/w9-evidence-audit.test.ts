import { strict as assert } from "node:assert";
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

function assertW9ProtectedDiffPolicy(
  changedPaths: string[],
  currentPackageValue: Record<string, any>,
  baselinePackageValue: Record<string, any>,
  currentLockValue: Record<string, any>,
  baselineLockValue: Record<string, any>,
  libDiff: string,
) {
  const allowedPaths = new Set([
    "package.json",
    "package-lock.json",
    "src-tauri/src/lib.rs",
    "src/commands/catalog.ts",
    "src/commands/catalog.test.ts",
    "src-tauri/src/application/catalog/mod.rs",
    "src-tauri/src/commands/catalog.rs",
    "src-tauri/src/application/mod.rs",
    "src-tauri/src/commands/mod.rs",
    "src-tauri/src/infrastructure/sqlite/mod.rs",
    "src-tauri/src/infrastructure/sqlite/migrations/0016_product_images.sql",
    "src-tauri/src/infrastructure/sqlite/migrations/0017_product_image_thumbnails.sql",
    "src-tauri/src/infrastructure/sqlite/catalog_repository.rs",
    "src-tauri/Cargo.lock",
    "src-tauri/Cargo.toml",
    "src-tauri/src/application/catalog/repository.rs",
    "src-tauri/tests/backup_restore.rs",
    "src-tauri/tests/catalog_maintenance_commands.rs",
    "src-tauri/tests/catalog_maintenance_sqlite.rs",
    "src-tauri/tests/post_sale_lifecycle.rs",
    "src-tauri/tests/sqlite_migrations.rs",
    "src/ui/app-shell.mounted.test.ts",
    "src/ui/catalog/catalog-maintenance-flow.test.ts",
    "src/ui/catalog/catalog-maintenance-flow.ts",
    "src/ui/catalog/catalog-maintenance-screen.mounted.test.ts",
    "src/ui/catalog/catalog-maintenance-screen.ts",
    "src/ui/catalog/product-browser.test.ts",
    "src/ui/catalog/product-browser.ts",
    "src/ui/styles.css",
    "src/ui/visual-system/catalog-edit-dialog.ts",
    "src/ui/visual-system/confirmation-dialog.ts",
    "src/ui/w9-evidence-audit.test.ts",
  ]);
  const unexpectedPaths = changedPaths.filter((path) => !allowedPaths.has(path));
  assert.deepEqual(unexpectedPaths, [], "unexpected protected-path drift");

  if (changedPaths.includes("package.json") || changedPaths.includes("package-lock.json")) {
    assertTicket14PackageAllowlist(
      currentPackageValue,
      baselinePackageValue,
      currentLockValue,
      baselineLockValue,
    );
  }
  if (changedPaths.includes("src-tauri/src/lib.rs")) {
    if (libDiff.includes("choose_product_image_command")) assertCatalogImageRegistrationAllowlist(libDiff);
    else if (libDiff.includes("dashboard_command")) assertDashboardRegistrationAllowlist(libDiff);
    else if (libDiff.includes("browse_products_command") || libDiff.includes("list_catalog_categories_command")) assertCatalogRegistrationAllowlist(libDiff);
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

test("W9 allows clean trees, ticket 14 metadata, and the bounded ticket 11 seam", () => {
  const changedProtectedPaths = execFileSync("git", ["diff", "--name-only", "HEAD"], { cwd: root, encoding: "utf8" }).trim();
  const changedPaths = changedProtectedPaths ? changedProtectedPaths.split("\n").sort() : [];
  const libDiff = execFileSync("git", ["diff", "--unified=0", "HEAD", "--", "src-tauri/src/lib.rs"], { cwd: root, encoding: "utf8" });
  assertW9ProtectedDiffPolicy(
    changedPaths,
    currentPackage,
    baselinePackage,
    currentLock,
    baselineLock,
    libDiff,
  );
  const uiSources = mountedSuites.map((suite) => read(suite)).join("\n");
  assert.doesNotMatch(uiSources, /https?:\/\/|cdn\.|innerHTML/);
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

test("W9 allows only the exact Dashboard module-registration paths", () => {
  const dashboardRegistrationPaths = [
    "src-tauri/src/application/mod.rs",
    "src-tauri/src/commands/mod.rs",
    "src-tauri/src/infrastructure/sqlite/mod.rs",
  ];
  for (const changedPath of dashboardRegistrationPaths) {
    assert.doesNotThrow(
      () => assertW9ProtectedDiffPolicy([changedPath], currentPackage, baselinePackage, currentLock, baselineLock, ""),
      changedPath,
    );
  }

  for (const changedPath of [
    "src-tauri/src/application/reporting/mod.rs",
    "src-tauri/src/commands/dashboard.rs",
    "src-tauri/src/infrastructure/sqlite/dashboard_repository.rs",
    "src-tauri/src/infrastructure/sqlite/migrations/0018_unrelated.sql",
    "src-tauri/src/infrastructure/sqlite/another_repository.rs",
    "src-tauri/tests/catalog_images.rs",
    "src-tauri/tests/dashboard_reporting.rs",
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

  const catalogImageDiff = execFileSync("git", ["diff", "--unified=0", "HEAD", "--", "src-tauri/src/lib.rs"], { cwd: root, encoding: "utf8" });
  assert.throws(
    () => assertCatalogImageRegistrationAllowlist(`${catalogImageDiff}+ fn unrelated_catalog_runtime_change() { native_runtime_drift(); }`),
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
