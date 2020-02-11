// Copyright (C) 2020 Guillaume Valadon <guillaume@valadon.net>

use std::fs::File;
use std::io::prelude::*; // used to get the BufRead trait
use std::io::BufReader;
use std::path::Path;

extern crate clap;
use clap::{App, Arg};

use xinto::{Record, RecordParsingError};

fn main() -> Result<(), String> {
    // Parse command line arguments
    let matches = App::new("xinto - parse & convert Intel hexadecimal object file format")
        .arg(
            Arg::with_name("HEX_FILENAME")
                .help("hex file to convert")
                .required(true),
        )
        .get_matches();

    // Check if the file exists
    let filename = matches.value_of("HEX_FILENAME").unwrap();
    if !Path::new(filename).is_file() {
        return Err(format!("'{}' is not a valid file!", filename));
    }

    // Check if the file can be opened
    let file = File::open(filename);
    if file.is_err() {
        return Err(format!("cannot open '{}'", filename));
    }

    // Parse the file to extract hex records
    let mut records = vec![];
    let buf_reader = BufReader::new(file.unwrap());

    for (line_number, line) in buf_reader.lines().enumerate().map(|(ln, l)| (ln + 1, l)) {
        if line.is_err() {
            return Err(format!("IO error at line {}: '{:?}'!", line_number, line));
        }

        let record = match Record::parse(&line.unwrap()) {
            Ok(r) => r,
            Err(RecordParsingError::MissingTag) => {
                eprintln!("Error at line {}: missing record mark!", line_number);
                break;
            }
            Err(e) => {
                eprintln!("Error at line {}: {:?}", line_number, e);
                break;
            }
        };

        records.push(record);
    }

    // Warn if the file format is incorrect
    if !records.is_empty() && *records.last().unwrap() != Record::end_of_file() {
        eprintln!("Error: last record is not a \"End of File Record\"!");
    }

    // Print parsed records as JSON
    let json_document =
        serde_json::to_string(&records).or_else(|_| Err("cannot convert to JSON!"))?;
    println!("{}", json_document);

    Ok(())
}
