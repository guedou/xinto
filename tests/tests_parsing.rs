// Copyright (C) 2020 Guillaume Valadon <guillaume@valadon.net>

#![feature(proc_macro_hygiene)]

#[cfg(test)]
mod tests {
    use xinto::{FileParsingError, Record, RecordParsingError};

    #[test]
    fn test_invalid_records() {
        // Record too small
        let result = Record::parse("");
        assert!(result.err() == Some(RecordParsingError::TooSmall));

        // Missing record mark
        let result = Record::parse("00000000000");
        assert!(result.err() == Some(RecordParsingError::MissingTag));

        // Invalid length format
        let result = Record::parse(":xy00000000");
        assert!(result.err() == Some(RecordParsingError::InvalidLengthFormat));

        // Invalid length
        let result = Record::parse(":ff00000000");
        assert!(result.err() == Some(RecordParsingError::InvalidLength));

        // Invalid load offset format
        let result = Record::parse(":00wxyz0000");
        assert!(result.err() == Some(RecordParsingError::InvalidLoadOffsetFormat));

        // Invalid type format
        let result = Record::parse(":0000000x00");
        assert!(result.err() == Some(RecordParsingError::InvalidTypeFormat));

        // Invalid type
        let result = Record::parse(":0000000f00");
        assert!(result.err() == Some(RecordParsingError::InvalidType));

        // Invalid checksum format
        let result = Record::parse(":00000000xx");
        assert!(result.err() == Some(RecordParsingError::InvalidChecksumFormat));

        // Invalid checksum
        let result = Record::parse(":00000000ff");
        assert!(result.err() == Some(RecordParsingError::InvalidChecksum));

        // Record too large
        let result = Record::parse(":0000000000aa");
        assert!(result.err() == Some(RecordParsingError::TooLarge));
    }

    #[test]
    fn test_valid_record() {
        // Valid record
        let result = Record::parse(":10010000214601360121470136007EFE09D2190140");
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

    #[test]
    fn test_from_file() {
        use std::fs;
        use std::fs::File;
        use std::io::prelude::*;
        use std::os::unix::fs::PermissionsExt;

        use tempfile::tempdir;

        // Create a temporary directory
        let dir = tempdir().unwrap();

        // Not a regular file
        let dirname = match dir.path().as_os_str().to_str() {
            Some(d) => d,
            None => {
                assert!(false);
                return ();
            }
        };

        let record = Record::from_file(dirname);
        assert_eq!(record, Err(FileParsingError::InvalidFile(dirname)));

        // Non readable file (UNIX test only)
        let file_path = dir.path().join("unreadable.hex");

        let file = match File::create(&file_path) {
            Ok(f) => f,
            Err(_) => {
                assert!(false);
                return ();
            }
        };

        let metadata = match file.metadata() {
            Ok(m) => m,
            Err(_) => {
                assert!(false);
                return ();
            }
        };

        let mut permissions = metadata.permissions();
        permissions.set_mode(0o000);

        match fs::set_permissions(&file_path, permissions) {
            Ok(_) => assert!(true),
            Err(_) => assert!(false),
        };

        let filename = match file_path.as_os_str().to_str() {
            Some(f) => f,
            None => {
                assert!(false);
                return ();
            }
        };

        match Record::from_file(&filename) {
            Err(FileParsingError::ReadFileError(_)) => assert!(true),
            _ => assert!(false),
        };

        // Catch a RecordParsingError
        let file_path = dir.path().join("test.hex");

        let mut file = match File::create(&file_path) {
            Ok(f) => f,
            Err(_) => {
                assert!(false);
                return ();
            }
        };
        match file.write_all(b"bad record") {
            Ok(_) => assert!(true),
            Err(_) => assert!(false),
        };

        let filename = match file_path.as_os_str().to_str() {
            Some(f) => f,
            None => {
                assert!(false);
                return ();
            }
        };

        println!("{:?}", Record::from_file(filename));
        match Record::from_file(&filename) {
            Err(FileParsingError::RecordError { .. }) => assert!(true),
            _ => assert!(false),
        };
    }
}
