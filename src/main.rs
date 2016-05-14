extern crate select;
extern crate miami_room;

use select::document::Document;
use std::convert::From;
use miami_room::*;

pub fn main() {
    let document = Document::from(include_str!("../example.html"));
    let schedule = Schedule::new(&document);
    //println!("{:?}", schedule.all_available_at_datetime(&Date::new(2016, 05, 08), &Time::new(16, 30)));
    println!("{:?}", schedule.find_available_ranges(60));
}
