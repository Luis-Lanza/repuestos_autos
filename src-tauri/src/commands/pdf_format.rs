//! Presentation-only date and timestamp formatting for PDF reports.

use chrono::{DateTime, FixedOffset, NaiveDate};
use chrono_tz::Tz;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormatError {
    InvalidValue,
    LocalTimezoneUnavailable,
}

/// Format a stored ISO date without converting it through a timezone.
pub fn format_date(value: &str) -> Result<String, FormatError> {
    let date =
        NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|_| FormatError::InvalidValue)?;
    if date.format("%Y-%m-%d").to_string() != value {
        return Err(FormatError::InvalidValue);
    }
    Ok(date.format("%d/%m/%Y").to_string())
}

/// Format an RFC 3339 instant at an explicitly supplied offset.
pub fn format_timestamp_at_offset(value: &str, offset: FixedOffset) -> Result<String, FormatError> {
    let instant = DateTime::parse_from_rfc3339(value).map_err(|_| FormatError::InvalidValue)?;
    Ok(instant
        .with_timezone(&offset)
        .format("%d/%m/%Y %H:%M")
        .to_string())
}

/// Format an RFC 3339 instant using checked IANA rules for the effective computer timezone.
pub fn format_timestamp_local(value: &str) -> Result<String, FormatError> {
    let instant = DateTime::parse_from_rfc3339(value).map_err(|_| FormatError::InvalidValue)?;
    let zone = effective_iana_timezone()?;
    Ok(instant
        .with_timezone(&zone)
        .format("%d/%m/%Y %H:%M")
        .to_string())
}

fn effective_iana_timezone() -> Result<Tz, FormatError> {
    // Honor only IANA process overrides; POSIX TZ strings and custom zone files are unsupported.
    let zone_name = match std::env::var_os("TZ") {
        Some(value) => value
            .into_string()
            .map_err(|_| FormatError::LocalTimezoneUnavailable)?,
        None => {
            iana_time_zone::get_timezone().map_err(|_| FormatError::LocalTimezoneUnavailable)?
        }
    };
    zone_name
        .parse()
        .map_err(|_| FormatError::LocalTimezoneUnavailable)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::FixedOffset;

    #[test]
    fn date_only_keeps_its_calendar_date_and_formats_leap_day() {
        assert_eq!(format_date("2024-02-29"), Ok("29/02/2024".to_owned()));
    }

    #[test]
    fn date_only_rejects_invalid_or_timestamp_values() {
        assert_eq!(format_date("2023-02-29"), Err(FormatError::InvalidValue));
        assert_eq!(
            format_date("2024-01-01T00:00:00Z"),
            Err(FormatError::InvalidValue)
        );
    }

    #[test]
    fn positive_offset_crosses_calendar_boundary_and_omits_seconds() {
        let offset = FixedOffset::east_opt(5 * 60 * 60 + 30 * 60).unwrap();
        assert_eq!(
            format_timestamp_at_offset("2024-01-01T20:45:59Z", offset),
            Ok("02/01/2024 02:15".to_owned())
        );
    }

    #[test]
    fn negative_offset_uses_the_offset_at_the_instant() {
        let offset = FixedOffset::west_opt(8 * 60 * 60).unwrap();
        assert_eq!(
            format_timestamp_at_offset("2024-01-01T02:05:59Z", offset),
            Ok("31/12/2023 18:05".to_owned())
        );
    }

    #[test]
    fn invalid_timestamps_fail_without_fallback() {
        let offset = FixedOffset::east_opt(0).unwrap();
        assert_eq!(
            format_timestamp_at_offset("not a timestamp", offset),
            Err(FormatError::InvalidValue)
        );
        assert_eq!(
            format_timestamp_local("not a timestamp"),
            Err(FormatError::InvalidValue)
        );
    }

    #[test]
    fn local_adapter_formats_an_instant_without_assuming_a_host_timezone() {
        let formatted = format_timestamp_local("2024-06-15T12:34:56Z").unwrap();
        assert_eq!(formatted.len(), 16);
        assert_eq!(&formatted[2..3], "/");
        assert_eq!(&formatted[5..6], "/");
        assert_eq!(&formatted[10..11], " ");
        assert_eq!(&formatted[13..14], ":");
    }
}
