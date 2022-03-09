use std::env;
use std::error::Error;
use std::process::exit;
mod day01;
mod day02;

struct Config {
    day: u8,
    filename: String,
}

impl Config {
    fn new(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 3 {
            return Err("Invalid number of arguments");
        }
        let day = match args[1].parse::<u8>() {
            Ok(val) => val,
            Err(_err) => return Err("Error parsing day number"),
        };
        let filename = args[2].clone();

        Ok(Config { day, filename })
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::new(&args).unwrap_or_else(|err| {
        println!("Coulnd't parse arguments: {}", err);
        exit(1);
    });

    println!("Day: {}\nFilename: {}", config.day, config.filename);

    if let Err(err) = run(config) {
        println!("{}", err);
        exit(2);
    }
}

fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let result = match config.day {
        1 => day01::day01(&config.filename),
        2 => day02::day02(&config.filename),
        _ => Err("Invalid day number")?,
    };

    result
}
