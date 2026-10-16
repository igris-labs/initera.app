use chrono::{
    Duration, LocalResult, NaiveDate, NaiveDateTime, NaiveTime, SecondsFormat, TimeZone, Utc,
};
use chrono_tz::Tz;

pub(crate) fn valid_zone(value: &str) -> bool {
    value.parse::<Tz>().is_ok()
}

pub(crate) fn valid_time(value: &str) -> bool {
    NaiveTime::parse_from_str(value, "%H:%M").is_ok()
}

// Resolve wall-clock boundaries consistently across DST. A repeated time takes
// its earlier instant; a skipped time advances to the first valid minute.
pub(crate) fn boundary(date: &str, time: &str, zone: &str) -> Result<String, String> {
    let tz: Tz = zone.parse().map_err(|_| "Choose a valid IANA time zone.")?;
    let date =
        NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(|_| "Choose a valid cycle date.")?;
    let time =
        NaiveTime::parse_from_str(time, "%H:%M").map_err(|_| "Choose a valid cycle time.")?;
    let mut local = NaiveDateTime::new(date, time);

    for _ in 0..=180 {
        let resolved = match tz.from_local_datetime(&local) {
            LocalResult::Single(value) => Some(value),
            LocalResult::Ambiguous(first, second) => Some(first.min(second)),
            LocalResult::None => None,
        };
        if let Some(value) = resolved {
            return Ok(value
                .with_timezone(&Utc)
                .to_rfc3339_opts(SecondsFormat::Secs, true));
        }
        local += Duration::minutes(1);
    }

    Err("This local cycle time cannot be resolved.".into())
}

pub(crate) fn local_date_now(zone: &str) -> Result<String, String> {
    let tz: Tz = zone.parse().map_err(|_| "Choose a valid IANA time zone.")?;
    Ok(Utc::now()
        .with_timezone(&tz)
        .date_naive()
        .format("%Y-%m-%d")
        .to_string())
}

pub(crate) fn utc_now() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_boundaries_follow_dst_policy() {
        assert_eq!(
            boundary("2026-03-08", "02:30", "America/New_York").unwrap(),
            "2026-03-08T07:00:00Z"
        );
        assert_eq!(
            boundary("2026-11-01", "01:30", "America/New_York").unwrap(),
            "2026-11-01T05:30:00Z"
        );
        assert_eq!(
            boundary("2026-03-01", "09:00", "America/New_York").unwrap(),
            "2026-03-01T14:00:00Z"
        );
        assert_eq!(
            boundary("2026-03-15", "09:00", "America/New_York").unwrap(),
            "2026-03-15T13:00:00Z"
        );
    }
}
