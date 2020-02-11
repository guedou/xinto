// Copyright (C) 2020 Guillaume Valadon <guillaume@valadon.net>

extern crate clap;
use clap::{App, Arg};

use xinto::Record;

fn main() -> Result<(), String> {
    // Parse command line arguments
    let matches = App::new("xinto - parse & convert Intel hexadecimal object file format")
        .arg(
            Arg::with_name("HEX_FILENAME")
                .help("hex file to convert")
                .required(true),
        )
        .get_matches();

    // Parse the file to extract hex records
    let filename = matches.value_of("HEX_FILENAME").unwrap();
    let records = Record::from_file(filename)?;

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
