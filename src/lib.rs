//! This is a library to parse dates in unknown format.
//! It diligently tries to apply known patterns and returns
//! best found match.
//!
//! # Examples
//!
//! ```rust
//! use diligent_date_parser::parse_date;
//! use diligent_date_parser::chrono::prelude::*;
//! use diligent_date_parser::chrono::offset::FixedOffset;
//!
//! assert_eq!(
//!     parse_date("Mon, 2 Jan 2006 15:04:05 MST"),
//!     Some(FixedOffset::west(7 * 3600).ymd(2006, 1, 2).and_hms(15, 4, 5)),
//! );
//! assert_eq!(
//!     parse_date("Apr 21 2016"),
//!     Some(Utc.ymd(2016, 4, 21).and_hms(0, 0, 0).into()),
//! );
//! assert_eq!(
//!     parse_date("Sun Dec 24 13:19:25 +0200 2017"),
//!     Some(Utc.ymd(2017, 12, 24).and_hms(11, 19, 25).into()),
//! );
//! assert_eq!(
//!     parse_date("Yesterday"),
//!     None,
//! );
//! ```

pub use chrono;
use chrono::prelude::*;
pub use chrono::{offset::FixedOffset, DateTime};
use std::convert::AsRef;

fn cut(string: &str, len: usize) -> Option<&str> {
    if string.len() >= len && string.is_char_boundary(len) {
        Some(&string[..len])
    } else {
        None
    }
}

fn suffix(string: &str, suffix: &'static str) -> String {
    format!("{}{}", string, suffix)
}

fn rfc3339<T: AsRef<str>>(string: T) -> Option<DateTime<FixedOffset>> {
    DateTime::parse_from_rfc3339(string.as_ref()).ok()
}

fn rfc2822<T: AsRef<str>>(string: T) -> Option<DateTime<FixedOffset>> {
    DateTime::parse_from_rfc2822(string.as_ref()).ok()
}

fn utc_datetime(string: &str, format: &str) -> Option<DateTime<FixedOffset>> {
    NaiveDateTime::parse_from_str(string, format)
        .map(|d| Utc.from_utc_datetime(&d))
        .ok()
        .map(|d: DateTime<Utc>| d.into())
}

fn utc_date(string: &str, format: &str) -> Option<DateTime<FixedOffset>> {
    let date = NaiveDate::parse_from_str(string, format).ok()?;
    let time = NaiveTime::from_hms_opt(0, 0, 0)?;
    let datetime = NaiveDateTime::new(date, time);
    Some(Utc.from_utc_datetime(&datetime).into())
}

/// Parses a string using multiple formats
///
/// # Example
///
/// ```rust
/// # use diligent_date_parser::parse_date;
/// # use diligent_date_parser::chrono::prelude::*;
/// # use diligent_date_parser::chrono::offset::FixedOffset;
/// let datetime = parse_date("Mon, 2 Jan 2006 15:04:05 MST");
/// let expected = FixedOffset::west(7 * 3600).ymd(2006, 1, 2).and_hms(15, 4, 5);
/// assert_eq!(datetime, Some(expected));
/// ```
pub fn parse_date(string: &str) -> Option<DateTime<FixedOffset>> {
    let trimmed = string.trim();
    None.or_else(|| rfc3339(trimmed))
        .or_else(|| cut(trimmed, 20).and_then(rfc3339))
        .or_else(|| cut(trimmed, 19).map(|s| suffix(s, "Z")).and_then(rfc3339))
        .or_else(|| DateTime::parse_from_str(trimmed, "%Y-%m-%d %H:%M:%S%.3f %z").ok())
        .or_else(|| utc_datetime(trimmed, "%Y-%m-%d %H:%M:%S%.3f"))
        .or_else(|| {
            cut(trimmed, 16)
                .map(|s| suffix(s, ":00Z"))
                .and_then(rfc3339)
        })
        .or_else(|| {
            cut(trimmed, 13)
                .map(|s| suffix(s, ":00:00Z"))
                .and_then(rfc3339)
        })
        .or_else(|| {
            cut(trimmed, 10)
                .map(|s| suffix(s, "T00:00:00Z"))
                .and_then(rfc3339)
        })
        .or_else(|| rfc2822(trimmed))
        .or_else(|| rfc2822(suffix(trimmed, " +0000")))
        .or_else(|| rfc2822(suffix(trimmed, ":00 +0000")))
        .or_else(|| rfc2822(suffix(trimmed, ":00:00 +0000")))
        .or_else(|| rfc2822(suffix(trimmed, " 00:00:00 +0000")))
        .or_else(|| DateTime::parse_from_str(trimmed, "%a %b %d %H:%M:%S %z %Y").ok()) // twitter's format
        .or_else(|| utc_date(trimmed, "%b %d %Y"))
        .or_else(|| utc_date(trimmed, "%b %e %Y"))
        .or_else(|| utc_date(trimmed, "%B %d %Y"))
        .or_else(|| utc_date(trimmed, "%B %e %Y"))
        .or_else(|| utc_date(trimmed, "%b %d, %Y"))
        .or_else(|| utc_date(trimmed, "%b %e, %Y"))
        .or_else(|| utc_date(trimmed, "%B %d, %Y"))
        .or_else(|| utc_date(trimmed, "%B %e, %Y"))
        .or_else(|| utc_date(trimmed, "%m/%d/%Y"))
        .or_else(|| utc_date(trimmed, "%d.%m.%Y"))
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::Duration;

    fn utc(year: i32, mon: u32, day: u32, hour: u32, min: u32, sec: u32) -> DateTime<FixedOffset> {
        Utc.with_ymd_and_hms(year, mon, day, hour, min, sec)
            .unwrap()
            .into()
    }

    #[test]
    fn test_parse_date() {
        assert_eq!(
            parse_date("2011-11-17T08:00:00-08:00"),
            Some(utc(2011, 11, 17, 16, 0, 0))
        );
        assert_eq!(
            parse_date("2011-11-17T08:00:00-08:00"),
            Some(
                FixedOffset::west_opt(8 * 3600)
                    .unwrap()
                    .with_ymd_and_hms(2011, 11, 17, 8, 0, 0)
                    .unwrap()
            )
        );
        assert_eq!(
            parse_date("2011-11-23T18:12:20Z"),
            Some(utc(2011, 11, 23, 18, 12, 20))
        );
        assert_eq!(
            parse_date("2011-12-10T14:32:42+00:00"),
            Some(utc(2011, 12, 10, 14, 32, 42))
        );
        assert_eq!(
            parse_date("2010-02-17T00:00:00ZT00:00:00-08:00"),
            Some(utc(2010, 2, 17, 0, 0, 0))
        );
        assert_eq!(
            parse_date("2010-12-21T19:57:37+00:00"),
            Some(utc(2010, 12, 21, 19, 57, 37))
        );
        assert_eq!(
            parse_date("2012-02-14T17:58:00-08:00"),
            Some(utc(2012, 2, 15, 1, 58, 0))
        );
        assert_eq!(
            parse_date("2012-02-15T12:24:00+02:00"),
            Some(utc(2012, 2, 15, 10, 24, 0))
        );
        assert_eq!(
            parse_date("2013-03-20T10:46:37.600732+02:00"),
            Some(utc(2013, 3, 20, 8, 46, 37) + Duration::microseconds(600732))
        );
        assert_eq!(
            parse_date("2013-03-20T14:00:00.000000+02:00"),
            Some(utc(2013, 3, 20, 12, 0, 0))
        );
        assert_eq!(
            parse_date("2013-10-21T18:23:10.394069+03:00"),
            Some(utc(2013, 10, 21, 15, 23, 10) + Duration::microseconds(394069))
        );
        assert_eq!(
            parse_date("2014-01-08T01:18:21"),
            Some(utc(2014, 1, 8, 1, 18, 21))
        );
        assert_eq!(
            parse_date("2014-01-07T20:45"),
            Some(utc(2014, 1, 7, 20, 45, 0))
        );
        assert_eq!(parse_date("2014-01-08T13"), Some(utc(2014, 1, 8, 13, 0, 0)));
        assert_eq!(parse_date("2014-01-11"), Some(utc(2014, 1, 11, 0, 0, 0)));

        assert_eq!(
            parse_date("2014-01-11 01:18:21 +0000"),
            Some(utc(2014, 01, 11, 1, 18, 21))
        );
        assert_eq!(
            parse_date("2014-01-11 01:18:21 +0100"),
            Some(
                FixedOffset::east_opt(3600)
                    .unwrap()
                    .with_ymd_and_hms(2014, 01, 11, 1, 18, 21)
                    .unwrap()
            )
        );
        assert_eq!(
            parse_date(" 2014-01-11 01:18:21 "),
            Some(utc(2014, 01, 11, 1, 18, 21))
        );
        assert_eq!(
            parse_date(" 2014-01-11 01:18:21.125 "),
            Some(utc(2014, 01, 11, 1, 18, 21) + Duration::milliseconds(125))
        );
        assert_eq!(
            parse_date("Fri, 12 Feb 2016 14:08:24 +0000"),
            Some(utc(2016, 2, 12, 14, 8, 24))
        );
        assert_eq!(
            parse_date("Fri, 13 Aug 2010 00:49:00 +0700"),
            Some(utc(2010, 8, 12, 17, 49, 0))
        );
        assert_eq!(
            parse_date("Fri, 13 Jul 2012 07:13:31 -0600"),
            Some(utc(2012, 7, 13, 13, 13, 31))
        );
        assert_eq!(
            parse_date("Fri, 14 Dec 2012 04:00:00 -0800"),
            Some(utc(2012, 12, 14, 12, 0, 0))
        );
        assert_eq!(
            parse_date("Fri, 14 Jun 2013 05:00:00 -0700"),
            Some(utc(2013, 6, 14, 12, 0, 0))
        );
        assert_eq!(
            parse_date("Fri, 14 Nov 2014 17:16:12 PST"),
            Some(utc(2014, 11, 15, 1, 16, 12))
        );
        assert_eq!(
            parse_date("Fri, 14 Oct 2011 04:01:47 +0000"),
            Some(utc(2011, 10, 14, 4, 1, 47))
        );
        assert_eq!(
            parse_date("Fri, 15 Apr 2016 00:00:00 +0200"),
            Some(utc(2016, 4, 14, 22, 0, 0))
        );
        assert_eq!(
            parse_date("Fri, 15 Apr 2016 23:02:22 GMT"),
            Some(utc(2016, 4, 15, 23, 2, 22))
        );
        assert_eq!(
            parse_date("Fri, 15 Mar 2013 07:27:18 +0000"),
            Some(utc(2013, 3, 15, 7, 27, 18))
        );
        assert_eq!(
            parse_date("Fri, 16 May 2014 02:13:00 PDT"),
            Some(utc(2014, 5, 16, 9, 13, 0))
        );
        assert_eq!(
            parse_date("Tue, 3 Jul 2012 23:02:36 +0400"),
            Some(utc(2012, 7, 3, 19, 2, 36))
        );
        assert_eq!(
            parse_date("Tue,  3  Jul 2012 23:02:36 +0400"),
            Some(utc(2012, 7, 3, 19, 2, 36))
        );
        assert_eq!(
            parse_date("Tue, 3 Jul 2012 23:02:36"),
            Some(utc(2012, 7, 3, 23, 2, 36))
        );
        assert_eq!(
            parse_date("Tue, 3 Jul 2012 23:02"),
            Some(utc(2012, 7, 3, 23, 2, 0))
        );
        assert_eq!(
            parse_date("Tue, 3 Jul 2012 23"),
            Some(utc(2012, 7, 3, 23, 0, 0))
        );
        assert_eq!(
            parse_date("Tue, 3 Jul 2012"),
            Some(utc(2012, 7, 3, 0, 0, 0))
        );
        assert_eq!(
            parse_date("3 Jul 2012 23:02:36"),
            Some(utc(2012, 7, 3, 23, 2, 36))
        );

        assert_eq!(parse_date("14 Apr 2016"), Some(utc(2016, 4, 14, 0, 0, 0)));
        assert_eq!(parse_date("21 Apr 2016"), Some(utc(2016, 4, 21, 0, 0, 0)));
        assert_eq!(parse_date("28 Apr 2016"), Some(utc(2016, 4, 28, 0, 0, 0)));
        assert_eq!(parse_date(" 7 Apr 2016"), Some(utc(2016, 4, 7, 0, 0, 0)));

        assert_eq!(parse_date("Apr 21 2016"), Some(utc(2016, 4, 21, 0, 0, 0)));
        assert_eq!(parse_date(" Apr  1, 2016"), Some(utc(2016, 4, 1, 0, 0, 0)));
        assert_eq!(
            parse_date("  April 01, 2016"),
            Some(utc(2016, 4, 1, 0, 0, 0))
        );

        // twitter
        assert_eq!(
            parse_date("Sun Dec 24 13:19:25 +0000 2017"),
            Some(utc(2017, 12, 24, 13, 19, 25))
        );
        assert_eq!(
            parse_date("Sun Dec 24 13:19:25 -0000 2017"),
            Some(utc(2017, 12, 24, 13, 19, 25))
        );
        assert_eq!(
            parse_date("Sun Dec 24 13:19:25 +0200 2017"),
            Some(utc(2017, 12, 24, 11, 19, 25))
        );
        assert_eq!(
            parse_date("Sun Dec 24 13:19:25 -0200 2017"),
            Some(utc(2017, 12, 24, 15, 19, 25))
        );
    }
}
