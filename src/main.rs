// Copyright (C) 2020 Guillaume Valadon <guillaume@valadon.net>

extern crate clap;
use clap::{crate_version, App, Arg};

use xinto::Record;

fn main() -> Result<(), String> {
    // Parse command line arguments
    let matches = App::new("xinto - parse & convert Intel hexadecimal object file format")
        .version(&crate_version!()[..])
        .arg(
            Arg::with_name("HEX_FILENAME")
                .help("hex file to convert")
                .required(true),
        )
        .arg(
            Arg::with_name("pretty")
                .short("p")
                .long("pretty")
                .help("Pretty print the JSON output"),
        )
        .get_matches();

    // Parse the file to extract hex records
    let filename = matches.value_of("HEX_FILENAME").unwrap();
    let records = Record::from_file(filename)?;

    // Print parsed records as JSON
    let struct_to_json = if matches.is_present("pretty") {
        serde_json::to_string_pretty
    } else {
        serde_json::to_string
    };
    let json_document = struct_to_json(&records).or_else(|_| Err("cannot convert to JSON!"))?;
    println!("{}", json_document);

    Ok(())
}
