//! Date/time helpers implemented without external dependencies.
//!
//! Git stores author/committer timestamps as Unix seconds. These helpers
//! convert them to ISO-8601 strings and "YYYY-MM" month keys using a
//! civil-from-days algorithm (Howard Hinnant's epoch algorithm).

/// Days since 1970-01-01 from a Unix timestamp.
fn days_from_epoch(secs: i64) -> i64 {
    secs.div_euclid(86400)
}

/// Convert days since 1970-01-01 to a (year, month, day) civil date.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = z - era * 146097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32; // [1, 12]
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// "YYYY-MM" month key for a Unix timestamp.
pub fn month_key(secs: i64) -> String {
    let (y, m, _) = civil_from_days(days_from_epoch(secs));
    format!("{y:04}-{m:02}")
}

/// "YYYY-MM-DD" date for a Unix timestamp.
pub fn iso_date(secs: i64) -> String {
    let (y, m, d) = civil_from_days(days_from_epoch(secs));
    format!("{y:04}-{m:02}-{d:02}")
}

/// Full ISO-8601 datetime for a Unix timestamp.
pub fn iso_datetime(secs: i64) -> String {
    let (y, m, d) = civil_from_days(days_from_epoch(secs));
    let rem = secs.rem_euclid(86400);
    let h = rem / 3600;
    let mi = (rem % 3600) / 60;
    let s = rem % 60;
    format!("{y:04}-{m:02}-{d:02}T{h:02}:{mi:02}:{s:02}")
}

/// Current UTC time as an ISO-8601 string.
pub fn now_iso() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    iso_datetime(secs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_is_1970() {
        assert_eq!(iso_date(0), "1970-01-01");
    }

    #[test]
    fn known_date() {
        // 2021-06-01 12:00:00 UTC
        let secs = 1622548800;
        assert_eq!(iso_date(secs), "2021-06-01");
        assert_eq!(iso_datetime(secs), "2021-06-01T12:00:00");
        assert_eq!(month_key(secs), "2021-06");
    }

    #[test]
    fn leap_year() {
        // 2000-02-29 is a valid date only in leap years.
        let secs = 951782400;
        assert_eq!(iso_date(secs), "2000-02-29");
    }

    #[test]
    fn negative_and_offsets() {
        // 1969-12-31T23:59:59Z
        assert_eq!(iso_datetime(-1), "1969-12-31T23:59:59");
    }
}
