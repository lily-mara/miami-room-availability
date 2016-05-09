#[macro_use]
extern crate lazy_static;
extern crate select;
extern crate regex;

use select::document::Document;
use select::predicate::{And, Class, Name};
use std::convert::From;
use std::collections::HashMap;

lazy_static! {
    static ref ROOM_NAME_REGEX: regex::Regex = regex::Regex::new(r"King Study Room (\d+) - (\d+) Person").unwrap();
}

#[derive(Debug, PartialEq)]
pub struct Time {
    hour: u8,
    minute: u8,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Date {
    year: i32,
    month: u8,
    day: u8,
}

pub enum ParseError {
    NameDoesNotMatch,
    NoNumber,
    NoCapacity,
}

#[derive(Debug)]
pub struct KingStudyRoom {
    room_number: u16,
    person_capacity: u8,
    available: HashMap<Date, Vec<TimeRange>>,
}

#[derive(Debug)]
pub struct TimeRange {
    start: Time,
    end: Time,
}

#[derive(Debug)]
pub enum AvailabilityRating {
    Available,
    Unavailable,
    NoInformation,
}

pub struct Schedule {
    rooms: Vec<KingStudyRoom>,
}

impl Schedule {
    pub fn new(document: &Document) -> Schedule {
        let mut rooms = Vec::new();

        for slot_collection in document.find(And(Name("tr"), Class("slots"))).iter() {
            let room_name = slot_collection.find(And(Class("resourceNameSelector"), Name("a"))).first().unwrap().text();
            let mut k = KingStudyRoom::from_str(&room_name).ok().unwrap();

            for slot in slot_collection.find(Class("slot")).iter() {
                k.add_availability_from_node(slot);
            }

            rooms.push(k);
        }

        Schedule{rooms: rooms}
    }

    pub fn all_available_at_datetime(&self, d: &Date, t: &Time) -> Vec<&KingStudyRoom> {
        let mut available = Vec::new();

        for room in &self.rooms {
            if room.is_available(d, t) {
                available.push(room);
            }
        }

        available
    }
}

impl KingStudyRoom {
    pub fn from_str(s: &str) -> Result<KingStudyRoom, ParseError> {
        let captures = match ROOM_NAME_REGEX.captures(s) {
            Some(x) => x,
            None => return Err(ParseError::NameDoesNotMatch),
        };

        let number = match captures.at(1) {
            Some(x) => x,
            None => return Err(ParseError::NoNumber),
        };

        let capacity = match captures.at(2) {
            Some(x) => x,
            None => return Err(ParseError::NoCapacity),
        };

        Ok(KingStudyRoom{
            room_number: number.parse::<u16>().unwrap(),
            person_capacity: capacity.parse::<u8>().unwrap(),
            available: HashMap::new(),
        })
    }

    pub fn add_availability_from_node(&mut self, n: select::node::Node) {
        let (day, start) = match n.attr("ref") {
            Some(x) => {
                match TimeRange::parse_stamp(x) {
                    Some(daystart) => daystart,
                    None => return,
                }
            },
            None => return,
        };

        let end = start.add(&Time::new(0, 30));
        let range = TimeRange::new(start, end);

        if self.available.contains_key(&day) {
            let mut intervals = self.available.get_mut(&day).unwrap();
            intervals.push(range);
        } else {
            let mut intervals = Vec::new();
            intervals.push(range);
            self.available.insert(day, intervals);
        }
    }

    pub fn is_available(&self, d: &Date, t: &Time) -> bool {
        match self.available.get(d) {
            Some(intervals) => for interval in intervals {
                if interval.contains_time(t) {
                    return true;
                }
            },
            None => return false,
        }
        false
    }
}

impl Date {
    pub fn new(year: i32, month: u8, day: u8) -> Date {
        if 1 > month || month > 12 {
            panic!("Month must be in range [1, 12]!");
        }

        if 1 > day || day > 31 {
            panic!("Day must be in range [1, 31]!");
        }

        Date{year: year, month: month, day: day}
    }
}

impl Time {
    pub fn new(hour: u8, minute: u8) -> Time {
        if hour > 23 {
            panic!("Hour must be in range [0, 24)!");
        }

        if minute > 59 {
            panic!("Minute must be in range [0, 60)!");
        }

        Time{hour: hour, minute: minute}
    }

    pub fn add(&self, other: &Time) -> Time {
        let mut t = Time{
            hour: self.hour,
            minute: self.minute,
        };

        t.minute += other.minute;
        t.hour += t.minute / 60;
        t.minute = t.minute % 60;
        t.hour += other.hour;

        t
    }

    pub fn as_minutes(&self) -> u32 {
        (self.hour as u32) * 60 + (self.minute as u32)
    }
}

impl TimeRange {
    pub fn new(start: Time, end: Time) -> TimeRange {
        TimeRange{start: start, end: end}
    }

    pub fn parse_stamp(stamp: &str) -> Option<(Date, Time)> {
        let (year_s, tail) = stamp.split_at(4);
        let (month_s, tail) = tail.split_at(2);
        let (day_s, tail) = tail.split_at(2);
        let (hour_s, tail) = tail.split_at(2);
        let (minute_s, _) = tail.split_at(2);

        let year = match year_s.parse() {
            Ok(s) => s,
            Err(_) => return None,
        };

        let month = match month_s.parse() {
            Ok(s) => s,
            Err(_) => return None,
        };

        let day = match day_s.parse() {
            Ok(s) => s,
            Err(_) => return None,
        };

        let hour = match hour_s.parse() {
            Ok(s) => s,
            Err(_) => return None,
        };

        let minute = match minute_s.parse() {
            Ok(s) => s,
            Err(_) => return None,
        };

        Some((Date::new(year, month, day), Time::new(hour, minute)))
    }

    pub fn contains_time(&self, time: &Time) -> bool {
        let min = time.as_minutes();
        return self.start.as_minutes() <= min && min < self.end.as_minutes()
    }
}

#[cfg(test)]
mod tests {
    use chrono::datetime::DateTime;
    use super::{ Time };

    //#[test]
    //fn test_date_stamp_parsing() {
        //let expected = DateTime::parse_from_rfc3339("2016-05-08T08:30:00-00:00").ok().unwrap();
        //let actual = TimePeriod::parse_stamp("201605080830005").unwrap();
        //assert_eq!(expected, actual);
    //}

    #[test]
    fn test_time_adding() {
        let expected = Time{
            hour: 13,
            minute: 20,
        };

        let actual = Time{hour: 10, minute: 30}.add(&Time{hour: 2, minute: 50});
        assert_eq!(actual, expected);
    }
}
