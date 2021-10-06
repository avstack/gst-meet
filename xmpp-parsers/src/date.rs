// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::util::error::Error;
use chrono::{DateTime as ChronoDateTime, FixedOffset};
use minidom::{IntoAttributeValue, Node};
use std::str::FromStr;

/// Implements the DateTime profile of XEP-0082, which represents a
/// non-recurring moment in time, with an accuracy of seconds or fraction of
/// seconds, and includes a timezone.
#[derive(Debug, Clone, PartialEq)]
pub struct DateTime(pub ChronoDateTime<FixedOffset>);

impl DateTime {
    /// Retrieves the associated timezone.
    pub fn timezone(&self) -> FixedOffset {
        self.0.timezone()
    }

    /// Returns a new `DateTime` with a different timezone.
    pub fn with_timezone(&self, tz: FixedOffset) -> DateTime {
        DateTime(self.0.with_timezone(&tz))
    }

    /// Formats this `DateTime` with the specified format string.
    pub fn format(&self, fmt: &str) -> String {
        format!("{}", self.0.format(fmt))
    }
}

impl FromStr for DateTime {
    type Err = Error;

    fn from_str(s: &str) -> Result<DateTime, Error> {
        Ok(DateTime(ChronoDateTime::parse_from_rfc3339(s)?))
    }
}

impl IntoAttributeValue for DateTime {
    fn into_attribute_value(self) -> Option<String> {
        Some(self.0.to_rfc3339())
    }
}

impl Into<Node> for DateTime {
    fn into(self) -> Node {
        Node::Text(self.0.to_rfc3339())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, Timelike};

    // DateTime’s size doesn’t depend on the architecture.
    #[test]
    fn test_size() {
        assert_size!(DateTime, 16);
    }

    #[test]
    fn test_simple() {
        let date: DateTime = "2002-09-10T23:08:25Z".parse().unwrap();
        assert_eq!(date.0.year(), 2002);
        assert_eq!(date.0.month(), 9);
        assert_eq!(date.0.day(), 10);
        assert_eq!(date.0.hour(), 23);
        assert_eq!(date.0.minute(), 08);
        assert_eq!(date.0.second(), 25);
        assert_eq!(date.0.nanosecond(), 0);
        assert_eq!(date.0.timezone(), FixedOffset::east(0));
    }

    #[test]
    fn test_invalid_date() {
        // There is no thirteenth month.
        let error = DateTime::from_str("2017-13-01T12:23:34Z").unwrap_err();
        let message = match error {
            Error::ChronoParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message.to_string(), "input is out of range");

        // Timezone ≥24:00 aren’t allowed.
        let error = DateTime::from_str("2017-05-27T12:11:02+25:00").unwrap_err();
        let message = match error {
            Error::ChronoParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message.to_string(), "input is out of range");

        // Timezone without the : separator aren’t allowed.
        let error = DateTime::from_str("2017-05-27T12:11:02+0100").unwrap_err();
        let message = match error {
            Error::ChronoParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message.to_string(), "input contains invalid characters");

        // No seconds, error message could be improved.
        let error = DateTime::from_str("2017-05-27T12:11+01:00").unwrap_err();
        let message = match error {
            Error::ChronoParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message.to_string(), "input contains invalid characters");

        // TODO: maybe we’ll want to support this one, as per XEP-0082 §4.
        let error = DateTime::from_str("20170527T12:11:02+01:00").unwrap_err();
        let message = match error {
            Error::ChronoParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message.to_string(), "input contains invalid characters");

        // No timezone.
        let error = DateTime::from_str("2017-05-27T12:11:02").unwrap_err();
        let message = match error {
            Error::ChronoParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message.to_string(), "premature end of input");
    }

    #[test]
    fn test_serialise() {
        let date =
            DateTime(ChronoDateTime::parse_from_rfc3339("2017-05-21T20:19:55+01:00").unwrap());
        let attr = date.into_attribute_value();
        assert_eq!(attr, Some(String::from("2017-05-21T20:19:55+01:00")));
    }
}
