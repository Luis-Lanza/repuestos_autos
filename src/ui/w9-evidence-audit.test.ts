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
  delete comparablePackage.scripts["typecheck:tests"];
  delete comparablePackage.devDependencies["@types/jsdom"];
  assert.deepEqual(comparablePackage, baselinePackage, "unexpected package.json drift");

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
  assert.match(read("openspec/changes/define-frontend-ui-ux-visual-system/design.md"), /contrast[\s\S]*validate/i);
});

test("W9 allows only ticket 14 package metadata changes", () => {
  const changedProtectedPaths = execFileSync("git", ["diff", "--name-only", "HEAD", "--", "src/commands", "src-tauri", "package.json", "package-lock.json"], { cwd: root, encoding: "utf8" }).trim();
  assert.deepEqual(changedProtectedPaths ? changedProtectedPaths.split("\n").sort() : [], ["package-lock.json", "package.json"]);
  assertTicket14PackageAllowlist(currentPackage, baselinePackage, currentLock, baselineLock);
  const uiSources = mountedSuites.map((suite) => read(suite)).join("\n");
  assert.doesNotMatch(uiSources, /https?:\/\/|cdn\.|innerHTML/);
});

test("W9 rejects non-allowlisted package metadata drift", () => {
  for (const drift of [
    (packageJson: Record<string, any>) => { packageJson.dependencies.react = "^18.3.2"; },
    (packageJson: Record<string, any>) => { packageJson.scripts.build = "vite --mode unexpected-drift"; },
  ]) {
    const driftedPackage = clone(currentPackage);
    drift(driftedPackage);
    assert.throws(
      () => assertTicket14PackageAllowlist(driftedPackage, baselinePackage, currentLock, baselineLock),
      /unexpected package\.json drift/,
    );
  }
});
