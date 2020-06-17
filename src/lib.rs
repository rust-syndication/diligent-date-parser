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

fn strptime(string: &str, format: &str) -> Option<DateTime<FixedOffset>> {
    NaiveDate::parse_from_str(string, format)
        .map(|d| DateTime::from_utc(d.and_hms(0, 0, 0), Utc))
        .ok()
        .map(|d: DateTime<Utc>| d.into())
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
        .or_else(|| strptime(trimmed, "%b %d %Y"))
        .or_else(|| strptime(trimmed, "%b %e %Y"))
        .or_else(|| strptime(trimmed, "%B %d %Y"))
        .or_else(|| strptime(trimmed, "%B %e %Y"))
        .or_else(|| strptime(trimmed, "%b %d, %Y"))
        .or_else(|| strptime(trimmed, "%b %e, %Y"))
        .or_else(|| strptime(trimmed, "%B %d, %Y"))
        .or_else(|| strptime(trimmed, "%B %e, %Y"))
        .or_else(|| strptime(trimmed, "%m/%d/%Y"))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_date() {
        assert_eq!(
            parse_date("2011-11-17T08:00:00-08:00"),
            Some(Utc.ymd(2011, 11, 17).and_hms(16, 0, 0).into())
        );
        assert_eq!(
            parse_date("2011-11-23T18:12:20Z"),
            Some(Utc.ymd(2011, 11, 23).and_hms(18, 12, 20).into())
        );
        assert_eq!(
            parse_date("2011-12-10T14:32:42+00:00"),
            Some(Utc.ymd(2011, 12, 10).and_hms(14, 32, 42).into())
        );
        assert_eq!(
            parse_date("2010-02-17T00:00:00ZT00:00:00-08:00"),
            Some(Utc.ymd(2010, 2, 17).and_hms(0, 0, 0).into())
        );
        assert_eq!(
            parse_date("2010-12-21T19:57:37+00:00"),
            Some(Utc.ymd(2010, 12, 21).and_hms(19, 57, 37).into())
        );
        assert_eq!(
            parse_date("2012-02-14T17:58:00-08:00"),
            Some(Utc.ymd(2012, 2, 15).and_hms(1, 58, 0).into())
        );
        assert_eq!(
            parse_date("2012-02-15T12:24:00+02:00"),
            Some(Utc.ymd(2012, 2, 15).and_hms(10, 24, 0).into())
        );
        assert_eq!(
            parse_date("2013-03-20T10:46:37.600732+02:00"),
            Some(Utc.ymd(2013, 3, 20).and_hms_micro(8, 46, 37, 600732).into())
        );
        assert_eq!(
            parse_date("2013-03-20T14:00:00.000000+02:00"),
            Some(Utc.ymd(2013, 3, 20).and_hms(12, 0, 0).into())
        );
        assert_eq!(
            parse_date("2013-10-21T18:23:10.394069+03:00"),
            Some(
                Utc.ymd(2013, 10, 21)
                    .and_hms_micro(15, 23, 10, 394069)
                    .into()
            )
        );
        assert_eq!(
            parse_date("2014-01-08T01:18:21"),
            Some(Utc.ymd(2014, 1, 8).and_hms(1, 18, 21).into())
        );
        assert_eq!(
            parse_date("2014-01-07T20:45"),
            Some(Utc.ymd(2014, 1, 7).and_hms(20, 45, 0).into())
        );
        assert_eq!(
            parse_date("2014-01-08T13"),
            Some(Utc.ymd(2014, 1, 8).and_hms(13, 0, 0).into())
        );
        assert_eq!(
            parse_date("2014-01-11"),
            Some(Utc.ymd(2014, 1, 11).and_hms(0, 0, 0).into())
        );

        assert_eq!(
            parse_date("2014-01-11 01:18:21 +0000"),
            Some(Utc.ymd(2014, 01, 11).and_hms(1, 18, 21).into())
        );

        assert_eq!(
            parse_date("Fri, 12 Feb 2016 14:08:24 +0000"),
            Some(Utc.ymd(2016, 2, 12).and_hms(14, 8, 24).into())
        );
        assert_eq!(
            parse_date("Fri, 13 Aug 2010 00:49:00 +0700"),
            Some(Utc.ymd(2010, 8, 12).and_hms(17, 49, 0).into())
        );
        assert_eq!(
            parse_date("Fri, 13 Jul 2012 07:13:31 -0600"),
            Some(Utc.ymd(2012, 7, 13).and_hms(13, 13, 31).into())
        );
        assert_eq!(
            parse_date("Fri, 14 Dec 2012 04:00:00 -0800"),
            Some(Utc.ymd(2012, 12, 14).and_hms(12, 0, 0).into())
        );
        assert_eq!(
            parse_date("Fri, 14 Jun 2013 05:00:00 -0700"),
            Some(Utc.ymd(2013, 6, 14).and_hms(12, 0, 0).into())
        );
        assert_eq!(
            parse_date("Fri, 14 Nov 2014 17:16:12 PST"),
            Some(Utc.ymd(2014, 11, 15).and_hms(1, 16, 12).into())
        );
        assert_eq!(
            parse_date("Fri, 14 Oct 2011 04:01:47 +0000"),
            Some(Utc.ymd(2011, 10, 14).and_hms(4, 1, 47).into())
        );
        assert_eq!(
            parse_date("Fri, 15 Apr 2016 00:00:00 +0200"),
            Some(Utc.ymd(2016, 4, 14).and_hms(22, 0, 0).into())
        );
        assert_eq!(
            parse_date("Fri, 15 Apr 2016 23:02:22 GMT"),
            Some(Utc.ymd(2016, 4, 15).and_hms(23, 2, 22).into())
        );
        assert_eq!(
            parse_date("Fri, 15 Mar 2013 07:27:18 +0000"),
            Some(Utc.ymd(2013, 3, 15).and_hms(7, 27, 18).into())
        );
        assert_eq!(
            parse_date("Fri, 16 May 2014 02:13:00 PDT"),
            Some(Utc.ymd(2014, 5, 16).and_hms(9, 13, 0).into())
        );
        assert_eq!(
            parse_date("Tue, 3 Jul 2012 23:02:36 +0400"),
            Some(Utc.ymd(2012, 7, 3).and_hms(19, 2, 36).into())
        );
        assert_eq!(
            parse_date("Tue,  3  Jul 2012 23:02:36 +0400"),
            Some(Utc.ymd(2012, 7, 3).and_hms(19, 2, 36).into())
        );
        assert_eq!(
            parse_date("Tue, 3 Jul 2012 23:02:36"),
            Some(Utc.ymd(2012, 7, 3).and_hms(23, 2, 36).into())
        );
        assert_eq!(
            parse_date("Tue, 3 Jul 2012 23:02"),
            Some(Utc.ymd(2012, 7, 3).and_hms(23, 2, 0).into())
        );
        assert_eq!(
            parse_date("Tue, 3 Jul 2012 23"),
            Some(Utc.ymd(2012, 7, 3).and_hms(23, 0, 0).into())
        );
        assert_eq!(
            parse_date("Tue, 3 Jul 2012"),
            Some(Utc.ymd(2012, 7, 3).and_hms(0, 0, 0).into())
        );
        assert_eq!(
            parse_date("3 Jul 2012 23:02:36"),
            Some(Utc.ymd(2012, 7, 3).and_hms(23, 2, 36).into())
        );

        assert_eq!(
            parse_date("14 Apr 2016"),
            Some(Utc.ymd(2016, 4, 14).and_hms(0, 0, 0).into())
        );
        assert_eq!(
            parse_date("21 Apr 2016"),
            Some(Utc.ymd(2016, 4, 21).and_hms(0, 0, 0).into())
        );
        assert_eq!(
            parse_date("28 Apr 2016"),
            Some(Utc.ymd(2016, 4, 28).and_hms(0, 0, 0).into())
        );
        assert_eq!(
            parse_date(" 7 Apr 2016"),
            Some(Utc.ymd(2016, 4, 7).and_hms(0, 0, 0).into())
        );

        assert_eq!(
            parse_date("Apr 21 2016"),
            Some(Utc.ymd(2016, 4, 21).and_hms(0, 0, 0).into())
        );
        assert_eq!(
            parse_date(" Apr  1, 2016"),
            Some(Utc.ymd(2016, 4, 1).and_hms(0, 0, 0).into())
        );
        assert_eq!(
            parse_date("  April 01, 2016"),
            Some(Utc.ymd(2016, 4, 1).and_hms(0, 0, 0).into())
        );

        // twitter
        assert_eq!(
            parse_date("Sun Dec 24 13:19:25 +0000 2017"),
            Some(Utc.ymd(2017, 12, 24).and_hms(13, 19, 25).into())
        );
        assert_eq!(
            parse_date("Sun Dec 24 13:19:25 -0000 2017"),
            Some(Utc.ymd(2017, 12, 24).and_hms(13, 19, 25).into())
        );
        assert_eq!(
            parse_date("Sun Dec 24 13:19:25 +0200 2017"),
            Some(Utc.ymd(2017, 12, 24).and_hms(11, 19, 25).into())
        );
        assert_eq!(
            parse_date("Sun Dec 24 13:19:25 -0200 2017"),
            Some(Utc.ymd(2017, 12, 24).and_hms(15, 19, 25).into())
        );
    }
}
