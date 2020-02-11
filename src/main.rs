// Copyright (C) 2020 Guillaume Valadon <guillaume@valadon.net>

use std::fs::File;
use std::io::prelude::*; // used to get the BufRead trait
use std::io::BufReader;
use std::path::Path;

extern crate clap;
use clap::{App, Arg};

use xinto::{parse_record, Record, RecordParsingError};

fn main() -> Result<(), u8> {
    // Parse command line arguments
    let matches = App::new("xinto - parse & convert Intel hexadecimal object file format")
        .arg(
            Arg::with_name("HEX_FILENAME")
                .help("hex file to convert")
                .required(true))
        .get_matches();

    // Check if the file exists
    let filename = matches.value_of("HEX_FILENAME").unwrap();
    if !Path::new(filename).is_file() {
        eprintln!("Error: '{}' is not a valid file!", filename);
        return Err(1);
    }

    let file = File::open(filename).unwrap();
    let buf_reader = BufReader::new(file);

    let mut v = vec![];
    for (line_number, line) in buf_reader.lines().enumerate().map(|(ln, l)| (ln + 1, l)) {
        let record = match parse_record(&line.unwrap()) {
            Ok(r) => r,
            Err(RecordParsingError::MissingTag) => {
                eprintln!("Error at line {}: missing record mark!", line_number);
                return Err(1);
            }
            Err(e) => {
                eprintln!("Error at line {}: {:?}", line_number, e);
                return Err(1);
            }
        };
        v.push(record);
    }

  
    if !v.is_empty() {
        if *v.last().unwrap() != Record::end_of_file() {
            eprintln!("Error: last record is not a \"End of File Record\"!");
            return Err(1);
	}
    }

    println!("{}", serde_json::to_string(&v).or_else(|_| Err(3))?);

    Ok(())
}
