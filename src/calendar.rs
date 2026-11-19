use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Date {
    year: u16,
    month: u8,
    day: u8,
}

fn month_len(year: u16, month: u8) -> u8 {
    match month {
        4 | 6 | 9 | 11 => 30,
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        _ => 31,
    }
}

impl FromStr for Date {
    type Err = &'static str;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let bytes = input.as_bytes();
        if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-'
            || ![&bytes[..4], &bytes[5..7], &bytes[8..]].into_iter().flatten().all(u8::is_ascii_digit)
        {
            return Err("expected a date in YYYY-MM-DD format");
        }
        let year = input[..4].parse().unwrap();
        let month = input[5..7].parse().unwrap();
        let day = input[8..].parse().unwrap();
        if year == 0 || !(1..=12).contains(&month) || !(1..=month_len(year, month)).contains(&day) {
            return Err("date does not exist");
        }
        Ok(Self { year, month, day })
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

impl Date {
    fn next(self) -> Option<Self> {
        if self.day < month_len(self.year, self.month) {
            Some(Self { day: self.day + 1, ..self })
        } else if self.month < 12 {
            Some(Self { month: self.month + 1, day: 1, ..self })
        } else if self.year < 9999 {
            Some(Self { year: self.year + 1, month: 1, day: 1 })
        } else {
            None
        }
    }

    fn previous(self) -> Option<Self> {
        if self.day > 1 {
            Some(Self { day: self.day - 1, ..self })
        } else if self.month > 1 {
            let month = self.month - 1;
            Some(Self { month, day: month_len(self.year, month), ..self })
        } else if self.year > 1 {
            Some(Self { year: self.year - 1, month: 12, day: 31 })
        } else {
            None
        }
    }

    fn weekday(self) -> u8 {
        let year = u32::from(self.year - 1);
        let preceding_years = year * 365 + year / 4 - year / 100 + year / 400;
        let preceding_months: u32 = (1..self.month).map(|m| u32::from(month_len(self.year, m))).sum();
        ((preceding_years + preceding_months + u32::from(self.day - 1)) % 7) as u8
    }
}

#[derive(Clone, Copy)]
pub enum View {
    Day,
    Week,
    Month,
}

impl FromStr for View {
    type Err = &'static str;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "day" => Ok(Self::Day),
            "week" => Ok(Self::Week),
            "month" => Ok(Self::Month),
            _ => Err("view must be day, week, or month"),
        }
    }
}

impl View {
    pub fn period(self, date: Date) -> Result<(String, Vec<Date>), &'static str> {
        let (title, start, count) = match self {
            Self::Day => (format!("Daily timeline · {date}"), date, 1),
            Self::Week => {
                let mut monday = date;
                for _ in 0..date.weekday() {
                    monday = monday.previous().ok_or("week starts outside the supported date range")?;
                }
                (format!("Weekly timeline · week of {monday}"), monday, 7)
            }
            Self::Month => (
                format!("Monthly timeline · {:04}-{:02}", date.year, date.month),
                Date { day: 1, ..date },
                month_len(date.year, date.month),
            ),
        };
        let days = std::iter::successors(Some(start), |date| date.next()).take(count.into()).collect::<Vec<_>>();
        if days.len() != usize::from(count) {
            return Err("period ends outside the supported date range");
        }
        Ok((title, days))
    }
}
