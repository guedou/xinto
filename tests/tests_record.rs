// Copyright (C) 2020 Guillaume Valadon <guillaume@valadon.net>

#[cfg(test)]
mod tests {
    use xinto::{parse_record, RecordParsingError};

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

        // Valid record
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
