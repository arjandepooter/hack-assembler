use clap::{App, Arg};
use std::fs::{metadata, read_to_string};

mod parser;
mod symbols;
mod types;

fn main() {
    let matches = App::new("Hack assembler")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .arg(
            Arg::with_name("input")
                .index(1)
                .required(true)
                .help("Path to the input file")
                .validator(|f| match metadata(f) {
                    Ok(m) => {
                        if m.is_file() {
                            Ok(())
                        } else {
                            Err("Input is not a valid file".to_string())
                        }
                    }
                    Err(e) => Err(format!("{}", e)),
                }),
        )
        .get_matches();

    let input_file = matches.value_of("input").unwrap();
    let file_contents = read_to_string(input_file).unwrap();

    match parser::parse_file(&file_contents).and_then(symbols::process_symbols) {
        Ok(symbols) => println!("{:?}", symbols),
        Err(e) => println!("{}", e),
    }
}
