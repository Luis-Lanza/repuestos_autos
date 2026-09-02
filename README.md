# Repuestos Autos

Desktop point-of-sale and inventory application built with React, Tauri 2, Rust, and SQLite.

## Run on Fedora 43

### 1. Install system dependencies

Tauri needs a C toolchain, WebKitGTK 4.1, and the Linux desktop development libraries:

```bash
sudo dnf check-update
sudo dnf install \
  webkit2gtk4.1-devel \
  openssl-devel \
  curl \
  wget \
  file \
  libappindicator-gtk3-devel \
  librsvg2-devel
sudo dnf group install "c-development"
```

These package names follow the [Tauri 2 prerequisites for Fedora](https://v2.tauri.app/start/prerequisites/). Fedora 43 provides both `webkit2gtk4.1-devel` and `libappindicator-gtk3-devel`.

### 2. Install Node.js and Rust

Install Node.js with npm. Fedora 43's standard `nodejs` package provides Node.js 22.22.2, which satisfies this repository's lockfile (`^22.22.2`, `^24.15.0`, or `>=26`):

```bash
sudo dnf install nodejs npm
node --version
npm --version
```

Install the stable Rust toolchain with [rustup](https://rustup.rs/) if `cargo` is not already available:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustup default stable
rustc --version
cargo --version
```

### 3. Install project dependencies

From the repository root:

```bash
npm ci
```

### 4. Start the desktop application

```bash
npm run tauri:dev
```

This command starts Vite automatically, builds the Rust desktop target with the required `desktop` feature, and opens the **Repuestos Autos** window. The first Rust build can take several minutes.

Do not start `npm run dev` separately. That command serves only the web frontend; native Tauri commands such as catalog, sales, inventory, and backup are unavailable there.

## Disposable manual smoke test

Use an isolated XDG data directory so the smoke test cannot modify your normal local database:

```bash
SMOKE_DATA_DIR="$(mktemp -d)"
XDG_DATA_HOME="$SMOKE_DATA_DIR" npm run tauri:dev
```

After the application exits, remove only the temporary directory created above:

```bash
rm -rf -- "$SMOKE_DATA_DIR"
unset SMOKE_DATA_DIR
```

While the application is running, verify this minimum path:

- [ ] The window opens at the desktop layout and can be resized down to 960×640.
- [ ] The sidebar exposes `Ventas`, `Inventario`, `Catálogo`, `Alta de productos`, `Historial de ventas`, and `Copia y restauración`.
- [ ] Keyboard navigation shows a visible focus indicator and activates each destination.
- [ ] Create a category and product, then confirm that the product is discoverable from Sales and Inventory.
- [ ] Record an inventory operation and confirm that the displayed stock changes.
- [ ] Add the product to a sale, enter a valid `Bs` payment, and confirm the sale once.
- [ ] Confirm that the persisted sale summary shows the sale-time facts and that `Nueva venta` returns to a clean draft.
- [ ] Confirm that the sale appears in Sales History.
- [ ] Create a backup only after test data exists. Skip destructive restore unless the disposable data directory is still active.

## Normal local data

Running `npm run tauri:dev` without the temporary `XDG_DATA_HOME` uses the normal Tauri application-data directory. On a default Fedora installation, the SQLite database is expected at:

```text
~/.local/share/com.repuestosautos.app/repuestos-autos.sqlite3
```

If `XDG_DATA_HOME` is configured, replace `~/.local/share` with that value. Back up this directory before manually testing restore behavior.

## Useful checks

```bash
# Show the detected Tauri, Rust, Node.js, and Linux dependencies.
npx tauri info

# Verify the frontend and TypeScript build.
npm test
npx tsc --noEmit
npm run build
```

If Tauri reports a missing WebKitGTK package, verify Fedora can resolve the required native library:

```bash
pkg-config --modversion webkit2gtk-4.1
```
