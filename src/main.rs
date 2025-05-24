use jlox::scanner::Scanner;
use anyhow::Result;

fn run(source: Vec<char>) {
    let mut scanner = Scanner::new(source);
    scanner.scan();
    scanner.log();
}

fn read_file(filename: &str) -> Result<String> {
    Ok(std::fs::read_to_string(filename)?)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args.len() {
        2 => {
            let filename = args[1].as_str();
            let source = read_file(filename);
            match source {
                Ok(source) => run(source.chars().collect()),
                Err(e) => panic!("Failed to open file {}: {}", filename, e)
            }
        },
        _ => panic!("Wrong number of arguments")
    }
}
