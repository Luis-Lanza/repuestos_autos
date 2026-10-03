use repuestos_autos::commands::pdf_format::{format_timestamp_local, FormatError};
use std::process::Command;

const CHILD_ZONE: &str = "PDF_FORMAT_CHILD_ZONE";

#[test]
fn timezone_child_checks_production_adapter() {
    let Ok(zone) = std::env::var(CHILD_ZONE) else {
        return;
    };

    let cases = match zone.as_str() {
        "America/Los_Angeles" => [
            ("2024-01-15T12:30:00Z", "15/01/2024 04:30"),
            ("2024-07-15T12:30:00Z", "15/07/2024 05:30"),
            ("2024-01-01T02:30:00+05:30", "31/12/2023 13:00"),
        ],
        "Asia/Kolkata" => [
            ("2024-01-15T12:30:00Z", "15/01/2024 18:00"),
            ("2024-07-15T12:30:00Z", "15/07/2024 18:00"),
            ("2024-01-01T02:30:00+05:30", "01/01/2024 02:30"),
        ],
        "Not/AZone" => {
            assert_eq!(
                format_timestamp_local("2024-01-15T12:30:00Z"),
                Err(FormatError::LocalTimezoneUnavailable)
            );
            return;
        }
        _ => panic!("unexpected test zone: {zone}"),
    };

    for (input, expected) in cases {
        assert_eq!(format_timestamp_local(input).as_deref(), Ok(expected));
    }
}

#[cfg(unix)]
fn run_in_zone(zone: &str) {
    let output = Command::new(std::env::current_exe().expect("test executable path"))
        .args([
            "--exact",
            "timezone_child_checks_production_adapter",
            "--nocapture",
        ])
        .env("TZ", zone)
        .env(CHILD_ZONE, zone)
        .output()
        .expect("timezone child process");
    assert!(
        output.status.success(),
        "child for {zone} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[cfg(unix)]
#[test]
fn production_adapter_uses_los_angeles_historical_and_dst_rules() {
    run_in_zone("America/Los_Angeles");
}

#[cfg(unix)]
#[test]
fn production_adapter_uses_kolkata_rules() {
    run_in_zone("Asia/Kolkata");
}

#[cfg(unix)]
#[test]
fn production_adapter_rejects_unavailable_effective_zone() {
    run_in_zone("Not/AZone");
}

#[test]
fn production_adapter_preserves_invalid_timestamp_error() {
    assert_eq!(
        format_timestamp_local("not a timestamp"),
        Err(FormatError::InvalidValue)
    );
}
