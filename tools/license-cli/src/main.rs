use ed25519_dalek::SigningKey;
use license_protocol::{
    key_id, sign_payload, validate_identifier, validate_machine_hash, LicensePayload, PRODUCT_ID,
};
use rand_core::OsRng;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

const HEX: &[u8; 16] = b"0123456789abcdef";

fn main() {
    if let Err(error) = run() {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), &'static str> {
    let mut args = env::args().skip(1);
    let command = args.next().ok_or("expected generate-keypair or sign")?;
    match command.as_str() {
        "generate-keypair" if args.next().is_none() => generate_keypair(),
        "sign" => {
            let license_id = args.next().ok_or("expected license ID and installation code")?;
            let installation_code = args.next().ok_or("expected license ID and installation code")?;
            if args.next().is_some() {
                return Err("unexpected arguments");
            }
            sign_license(&license_id, &installation_code)
        }
        _ => Err("invalid command or arguments"),
    }
}

fn generate_keypair() -> Result<(), &'static str> {
    println!("Private key destination (outside this repository):");
    let repository_root = repository_root()?;
    let path = prompt_new_external_file(&prompt_path()?, &repository_root)?;
    let signing_key = SigningKey::generate(&mut OsRng);
    let private_seed = encode_hex(signing_key.to_bytes().as_ref());
    create_private_file(&path, private_seed.as_bytes())?;
    let public_key = signing_key.verifying_key();
    println!("Public verification key: {}", encode_hex(public_key.as_bytes()));
    println!("Key ID: {}", key_id(&public_key));
    Ok(())
}

fn sign_license(license_id: &str, installation_code: &str) -> Result<(), &'static str> {
    validate_identifier(license_id).map_err(|_| "invalid license ID")?;
    validate_machine_hash(installation_code).map_err(|_| "invalid installation code")?;
    println!("External private-key file (outside this repository):");
    let key_path = prompt_path()?;
    let repository_root = repository_root()?;
    ensure_outside_repository(&key_path, &repository_root)?;
    let private_seed = read_private_seed(&key_path, &repository_root)?;
    let signing_key = SigningKey::from_bytes(&private_seed);
    let machine_hash = installation_code.to_owned();
    let issued_at = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|_| "could not format issuance time")?;
    let issued_at = issued_at
        .split_once('.')
        .map(|(seconds, _)| format!("{seconds}Z"))
        .unwrap_or(issued_at);
    let payload = LicensePayload {
        version: 1,
        license_id: license_id.to_owned(),
        key_id: key_id(&signing_key.verifying_key()),
        product_id: PRODUCT_ID.to_owned(),
        machine_hash,
        issued_at,
    };
    let envelope = sign_payload(payload, &signing_key).map_err(|_| "could not create license")?;
    println!("License output path (.lic, outside this repository):");
    let output_path = prompt_path()?;
    ensure_lic_output_outside_repository(&output_path, &repository_root)?;
    create_new_file(&output_path, &envelope)?;
    println!("License written.");
    Ok(())
}

fn prompt_path() -> Result<PathBuf, &'static str> {
    print!("> ");
    io::stdout().flush().map_err(|_| "could not prompt for path")?;
    let mut value = String::new();
    io::stdin().read_line(&mut value).map_err(|_| "could not read path")?;
    let value = value.trim_end_matches(['\r', '\n']);
    if value.is_empty() {
        return Err("path is required");
    }
    Ok(PathBuf::from(value))
}

fn repository_root() -> Result<PathBuf, &'static str> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest.join("../..").canonicalize().map_err(|_| "could not resolve repository boundary")
}

pub(crate) fn ensure_outside_repository(path: &Path, repository_root: &Path) -> Result<PathBuf, &'static str> {
    let canonical = path.canonicalize().map_err(|_| "selected file does not exist")?;
    if canonical.starts_with(repository_root) || !canonical.is_file() {
        return Err("selected key file must be outside the repository");
    }
    Ok(canonical)
}

pub(crate) fn prompt_new_external_file(path: &Path, repository_root: &Path) -> Result<PathBuf, &'static str> {
    let name = path.file_name().ok_or("invalid private-key path")?;
    let parent = path.parent().ok_or("invalid private-key path")?.canonicalize().map_err(|_| "key directory does not exist")?;
    if parent.starts_with(repository_root) {
        return Err("private-key file must be outside the repository");
    }
    let destination = parent.join(name);
    if destination.exists() {
        return Err("private-key file already exists");
    }
    Ok(destination)
}

pub(crate) fn ensure_lic_output_outside_repository(path: &Path, repository_root: &Path) -> Result<(), &'static str> {
    if path.extension().and_then(|value| value.to_str()) != Some("lic") {
        return Err("output file must use the .lic extension");
    }
    let name = path.file_name().ok_or("invalid output path")?;
    let parent = path.parent().ok_or("invalid output path")?.canonicalize().map_err(|_| "output directory does not exist")?;
    if parent.starts_with(repository_root) {
        return Err("output file must be outside the repository");
    }
    let full_path = parent.join(name);
    if full_path.exists() {
        return Err("output file already exists");
    }
    Ok(())
}

pub(crate) fn read_private_seed(path: &Path, repository_root: &Path) -> Result<[u8; 32], &'static str> {
    let path = ensure_outside_repository(path, repository_root)?;
    let mut bytes = Vec::with_capacity(65);
    fs::File::open(path)
        .map_err(|_| "could not read private-key file")?
        .take(65)
        .read_to_end(&mut bytes)
        .map_err(|_| "could not read private-key file")?;
    if bytes.len() != 64 || !bytes.iter().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(byte)) {
        return Err("private-key file must contain exactly 64 lowercase hexadecimal bytes");
    }
    let mut seed = [0_u8; 32];
    for (index, pair) in bytes.chunks_exact(2).enumerate() {
        seed[index] = (hex_value(pair[0])? << 4) | hex_value(pair[1])?;
    }
    Ok(seed)
}

fn create_private_file(path: &Path, contents: &[u8]) -> Result<(), &'static str> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path).map_err(|_| "could not create private-key file")?;
    file.write_all(contents).map_err(|_| "could not write private-key file")?;
    file.sync_all().map_err(|_| "could not secure private-key file")?;
    Ok(())
}

pub(crate) fn create_new_file(path: &Path, contents: &[u8]) -> Result<(), &'static str> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|_| "could not create license output file")?;
    file.write_all(contents).map_err(|_| "could not write license output file")?;
    file.sync_all().map_err(|_| "could not finish license output file")?;
    Ok(())
}

fn encode_hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

fn hex_value(byte: u8) -> Result<u8, &'static str> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        _ => Err("private-key file must contain exactly 64 lowercase hexadecimal bytes"),
    }
}
