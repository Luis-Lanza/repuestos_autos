# Vendor license CLI

This package signs offline licenses and is intentionally outside `src-tauri` and the application build graph. This public repository contains **no secret signing key**. Do not put production private keys, customer records, or issued license files in this repository.

## Key custody

Generate a keypair with `cargo run --manifest-path tools/license-cli/Cargo.toml -- generate-keypair`. The CLI interactively prompts for a private-key file destination outside the repository. It creates a new file containing exactly the 32-byte Ed25519 seed encoded as 64 lowercase hexadecimal bytes (no newline); on Unix the file is created with mode `0600`. It prints only the public verification key and its key ID. The operator is responsible for protecting the private file with vendor-controlled access controls, encrypted storage, and a separate secure backup. Verify backup recovery procedures and restrict access to both the primary and backup copies. Loss of the signing key prevents issuing licenses for its key ID; compromise requires vendor-managed key rotation and application trust updates.

Never pass private-key bytes or a private-key path as arguments or environment variables. The CLI prompts for the external file path. Keep the key on vendor-controlled systems; never ship it in the app, installer, customer machine, or source repository. The CLI reports generic errors and does not print private-key bytes.

## Commands

Generate a keypair (the private destination is prompted interactively):

```sh
cargo run --manifest-path tools/license-cli/Cargo.toml -- generate-keypair
```

Sign a license using a license ID and the customer's installation code. The CLI interactively prompts for the external private-key path and then an explicit `.lic` output path outside the repository:

```sh
cargo run --manifest-path tools/license-cli/Cargo.toml -- sign LICENSE-ID LOWERCASE-64-HEX-INSTALLATION-CODE
```

The installation code is the lowercase hexadecimal SHA-256 machine digest. The signed v1 payload uses the fixed product ID, UTC RFC3339 issue time at second precision, and a key ID equal to the full lowercase-hex SHA-256 of the Ed25519 public key. License ID and key ID are limited to 1–64 ASCII letters, digits, dot, underscore, or hyphen. The `.lic` output is created without overwriting an existing file.
