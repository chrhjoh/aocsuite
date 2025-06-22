use aocsuite::{
    parse_args,
    utils::{is_puzzle_released, is_year_released, today},
};
use chrono::Datelike;

fn main() {
    let args = parse_args();
    let day = match args.day {
        Some(day) => day,
        None => today().day(),
    };
    let year = match args.year {
        Some(year) => year,
        None => today().year(),
    };
    println!("year:{:?}, day:{:?}", year, day);

    if is_puzzle_released(day, year) {
        println!("Puzzle is released")
    }
    if is_year_released(day, year) {
        println!("Year is released")
    }
    println!("{:?}", args)
}
