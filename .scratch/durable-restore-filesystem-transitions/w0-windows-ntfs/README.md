# W0 Windows/local-NTFS feasibility harness

This ignored standalone crate collects **external feasibility evidence** for
`durable-restore-filesystem-transitions`. It is not product code, does not run W2, and does not
prove crash durability. In particular, `ReplaceFileW` is invoked with flags `0` because Microsoft
documents `REPLACEFILE_WRITE_THROUGH` as unsupported; the harness does not treat replacement alone
as a durability barrier.

## Run (ordinary user, local Windows PowerShell)

Requirements: a real Windows machine, ordinary-user `%LOCALAPPDATA%` on a local fixed NTFS volume
(not WSL, a network share, a subst/virtual/removable volume, or a cloud/reparse-backed location), and
the Rust MSVC toolchain. Run PowerShell **without** “Run as administrator”:

```powershell
cd <checkout>\.scratch\durable-restore-filesystem-transitions\w0-windows-ntfs
powershell -NoProfile -ExecutionPolicy Bypass -File .\run.ps1
```

The script refuses an elevated shell. For each run it keeps evidence under ignored
`evidence/<run-id>/` but creates the disposable runtime root at
`%LOCALAPPDATA%\RepuestosAutos-W0\<run-id>`. The runtime layout is
`<TEST_ROOT>/backup-restore/staging`, with the backup-restore and root directories surrounding it.
After runtime output capture finishes, the script recursively removes the disposable root and
records the sanitized result in `cleanup.json`. Cleanup failure makes the overall exit status
non-zero.

The evidence directory contains sanitized environment facts, `cargo tree`, build output, runtime
stderr, JSONL operation evidence, cleanup and step exit codes, and SHA-256 checksums. Every text
artifact is sanitized before checksums are generated. Evidence may use `<TEST_ROOT>`, `<ROOT>`, and
`<VOLUME_MOUNT>` labels, but must not contain a raw profile path, username, Windows identity, or SID.
The required directory-barrier evidence separately covers staging, backup-restore, and root.
Runtime exits non-zero unless every required W0 case passes. Unsupported optional storage layouts
are explicit `SKIP`; unsupported required local NTFS operations (including directory flush or
extended no-replace/write-through rename) are `UNPROVEN`/`FAIL`, never `PASS`.

Inspect `exit-codes.json`, `cleanup.json`, and the final `summary` JSONL record. Return the evidence
by zipping the whole run directory (do not rename or selectively copy files):

```powershell
$run = Get-ChildItem .\evidence -Directory | Sort-Object LastWriteTime -Descending | Select-Object -First 1
Compress-Archive -LiteralPath $run.FullName -DestinationPath ("$($run.FullName).zip")
```

Before sending, inspect text files for unexpected personal paths or usernames. The maintainer must
review and approve the sanitized W0 evidence before any W2 work can proceed.
