// Copyright (C) 2020 Guillaume Valadon <guillaume@valadon.net>

extern crate hex;
use hex::FromHex;

extern crate serde;
extern crate serde_json;
use serde::{Deserialize, Serialize};

fn u8_from_hex(input: &str) -> Result<u8, std::num::ParseIntError> {
    u8::from_str_radix(input, 16)
}

fn u16_from_hex(input: &str) -> Result<u16, std::num::ParseIntError> {
    u16::from_str_radix(input, 16)
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

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Record {
    pub length: u8,
    pub load_offset: u16,
    pub r#type: u8,
    pub data: Vec<u8>,
    pub checksum: u8,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RecordParsingError {
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

impl Record {
    pub fn end_of_file() -> Self {
        Record {
            length: 0,
            load_offset: 0,
            r#type: 1,
            data: vec![],
            checksum: 0xFF,
        }
    }

    pub fn parse(input: &str) -> Result<Record, RecordParsingError> {
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

        if !record.verify_checksum() {
            return Err(RecordParsingError::InvalidChecksum);
        }

        Ok(record)
    }

    pub fn verify_checksum(&self) -> bool {
        let sum = self.length as u32
            + (self.load_offset & 0xFF) as u32
            + (self.load_offset >> 8) as u32
            + self.r#type as u32
            + self
                .data
                .iter()
                .fold(0 as u32, |s, value| s as u32 + (*value as u32))
            + self.checksum as u32;
        sum.trailing_zeros() >= 8
    }
}
