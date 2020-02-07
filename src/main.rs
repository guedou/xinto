// Guillaume Valadon <guillaume@valadon.net>

use std::fs::File;
use std::io::prelude::*; // used to get the BufRead trait
use std::io::BufReader;

extern crate hex;
use hex::FromHex;

extern crate serde;
extern crate serde_json;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Record {
    length: u8,
    load_offset: u16,
    r#type: u8,
    data: Vec<u8>,
    checksum: u8,
}

fn u8_from_hex(input: &str) -> Result<u8, std::num::ParseIntError> {
    u8::from_str_radix(input, 16)
}

fn u16_from_hex(input: &str) -> Result<u16, std::num::ParseIntError> {
    u16::from_str_radix(input, 16)
}

fn verify_checksum(record: &Record) -> bool {
    let sum = record.length as u32
        + (record.load_offset & 0xFF) as u32
        + (record.load_offset >> 8) as u32
        + record.r#type as u32
        + record
            .data
            .iter()
            .fold(0 as u32, |s, value| s as u32 + (*value as u32))
        + record.checksum as u32;
    sum.trailing_zeros() >= 8
}

#[derive(Clone, Debug, PartialEq)]
enum RecordParsingError {
    TooSmall,
    MissingTag,
    InvalidLengthFormat,
    InvalidLength,
    InvalidLoadOffsetFormat,
    InvalidType,
    InvalidTypeFormat,
    InvalidDataFormat,
    InvalidChecksum,
    InvalidChecksumFormat,
    TooLarge,
    ParseIntError,
}

fn parse_u8(input: &str) -> Result<(&str, u8), RecordParsingError> {
    let size = std::mem::size_of::<u8>() * 2;
    if input.len() < size {
        return Err(RecordParsingError::TooSmall);
    }
    Ok((
        &input[size..],
        u8_from_hex(&input[0..size]).or_else(|_| Err(RecordParsingError::ParseIntError))?,
    ))
}

fn parse_u16(input: &str) -> Result<(&str, u16), RecordParsingError> {
    let size = std::mem::size_of::<u16>() * 2;
    if input.len() < size {
        return Err(RecordParsingError::TooSmall);
    }
    Ok((
        &input[size..],
        u16_from_hex(&input[0..size]).or_else(|_| Err(RecordParsingError::ParseIntError))?,
    ))
}

fn parse_record(input: &str) -> Result<Record, RecordParsingError> {
    if input.len() < 11 {
        return Err(RecordParsingError::TooSmall);
    }

    if !input.starts_with(':') {
        return Err(RecordParsingError::MissingTag);
    }
    let input = &input[1..];

    let (input, length) =
        parse_u8(input).or_else(|_| Err(RecordParsingError::InvalidLengthFormat))?;

    let (input, load_offset) =
        parse_u16(input).or_else(|_| Err(RecordParsingError::InvalidLoadOffsetFormat))?;

    let (input, r#type) =
        parse_u8(input).or_else(|_| Err(RecordParsingError::InvalidTypeFormat))?;
    if r#type > 5 {
        return Err(RecordParsingError::InvalidType);
    }

    let char_count: usize = length as usize * 2;
    if char_count > (input.len() - 2) {
        return Err(RecordParsingError::InvalidLength);
    }

    let (input, data) = (&input[char_count..], &input[0..char_count]);

    let (input, checksum) =
        parse_u8(input).or_else(|_| Err(RecordParsingError::InvalidChecksumFormat))?;

    let record = Record {
        length,
        load_offset,
        r#type,
        data: Vec::from_hex(data).or_else(|_| Err(RecordParsingError::InvalidDataFormat))?,
        checksum,
    };

    if !input.is_empty() {
        return Err(RecordParsingError::TooLarge);
    }

    if !verify_checksum(&record) {
        return Err(RecordParsingError::InvalidChecksum);
    }

    Ok(record)
}

fn main() -> Result<(), u8> {
    let file = File::open("data/wikipedia.hex").unwrap();
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
    println!("{}", serde_json::to_string(&v).or_else(|_| Err(3))?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{parse_record, RecordParsingError};

    #[test]
    fn test_format() {
        // Record too small
        let result = parse_record("");
        assert!(result.err() == Some(RecordParsingError::TooSmall));

        // Missing record mark
        let result = parse_record("00000000000");
        assert!(result.err() == Some(RecordParsingError::MissingTag));

        // Invalid length format
        let result = parse_record(":xy00000000");
        assert!(result.err() == Some(RecordParsingError::InvalidLengthFormat));

        // Invalid length
        let result = parse_record(":ff00000000");
        assert!(result.err() == Some(RecordParsingError::InvalidLength));

        // Invalid load offset format
        let result = parse_record(":00wxyz0000");
        assert!(result.err() == Some(RecordParsingError::InvalidLoadOffsetFormat));

        // Invalid type format
        let result = parse_record(":0000000x00");
        assert!(result.err() == Some(RecordParsingError::InvalidTypeFormat));

        // Invalid type
        let result = parse_record(":0000000f00");
        assert!(result.err() == Some(RecordParsingError::InvalidType));

        // Invalid checksum format
        let result = parse_record(":00000000xx");
        assert!(result.err() == Some(RecordParsingError::InvalidChecksumFormat));

        // Invalid checksum
        let result = parse_record(":00000000ff");
        assert!(result.err() == Some(RecordParsingError::InvalidChecksum));

        // Record too large
        let result = parse_record(":0000000000aa");
        assert!(result.err() == Some(RecordParsingError::TooLarge));

        // Valid Record
        let result = parse_record(":10010000214601360121470136007EFE09D2190140");
        if result.is_err() {
            assert!(false);
        }
        let record = result.unwrap();
        assert!(record.length == 0x10);
        assert!(record.load_offset == 0x100);
        assert!(record.r#type == 0x00);
        assert!(record.checksum == 0x40);
        assert!(record.data == [33, 70, 1, 54, 1, 33, 71, 1, 54, 0, 126, 254, 9, 210, 25, 1]);
    }
}
