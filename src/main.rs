use clap::{App, Arg};
use std::fs::{metadata, read_to_string};
use std::io::{stdout, Write};

mod output;
mod parser;
mod symbols;
mod types;

fn write_output<W>(buffer: &mut W, lines: Vec<String>)
where
    W: Write,
{
    for line in lines.iter() {
        buffer
            .write(line.as_bytes())
            .expect("Error while writing to output");
        buffer.write("\n".as_bytes());
    }
    buffer.flush().unwrap();
}

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

    match parser::parse_file(&file_contents) {
        Err(err) => println!("{}", err),
        Ok(instructions) => match symbols::process_symbols(&instructions) {
            Err(err) => println!("{}", err),
            Ok(symbols) => {
                let out = stdout();
                let mut handle = out.lock();
                let lines = output::to_machine_instructions(&instructions, symbols);
                write_output(&mut handle, lines);
            }
        },
    }
}
