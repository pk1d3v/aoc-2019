use std::process::exit;
mod day01;

fn main() {
    if let Err(err) = day01::day01() {
        println!("{}", err);
        exit(-1);
    }
}


