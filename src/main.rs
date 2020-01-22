// Guillaume Valadon <guillaume@valadon.net>

use std::fs::File;
use std::io::prelude::*; // used to get the BufRead trait
use std::io::BufReader;

extern crate nom;
use nom::bytes::complete::{tag, take, take_while_m_n};
use nom::combinator::map_res;
use nom::error::ErrorKind;
use nom::sequence::tuple;
use nom::IResult;

extern crate hex;
use hex::FromHex;

#[derive(Debug)]
struct Record {
    length: u8,
    load_offset: u16,
    r#type: u8,
    data: Vec<u8>,
    checksum: u8,
}

fn is_hex_digit(c: char) -> bool {
    c.is_digit(16)
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

fn parse_record(input: &str) -> IResult<&str, Record> {
    let (input, (_record_mark, length, load_offset, r#type)) = tuple((
        tag(":"),
        map_res(take_while_m_n(2, 2, is_hex_digit), u8_from_hex),
        map_res(take_while_m_n(4, 4, is_hex_digit), u16_from_hex),
        map_res(take_while_m_n(2, 2, is_hex_digit), u8_from_hex),
    ))(input)?;

    let char_count: usize = length as usize * 2;

    let (input, (data, checksum)) = tuple((
        take_while_m_n(char_count, char_count, is_hex_digit),
        map_res(take(2usize), u8_from_hex),
    ))(input)?;

    let record = Record {
        length,
        load_offset,
        r#type,
        data: Vec::from_hex(data)
            .or_else(|_| Err(nom::Err::Failure((data, ErrorKind::HexDigit))))?,
        checksum,
    };

    println!("{:?}", verify_checksum(&record));

    Ok((input, record))
}

fn main() {
    let file = File::open("data/wikipedia.hex").unwrap();
    let buf_reader = BufReader::new(file);
    for line in buf_reader.lines() {
        println!("line: {:?}", line);
        println!("record: {:?}", parse_record(&line.unwrap()));
        println!("====");
    }
}
