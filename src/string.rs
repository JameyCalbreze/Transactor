//! Basic utilities for working with Strings. This was written to assist with
//! writing unit tests within other modules. Especially tests which require parsing of CSV data.

use std::io::Read;

/// Utility for testing CSV input for this binary
pub(crate) struct StringReader {
    inner: String,
    cursor: usize,
}

impl From<String> for StringReader {
    fn from(value: String) -> Self {
        StringReader {
            inner: value,
            cursor: 0,
        }
    }
}

impl From<&str> for StringReader {
    fn from(value: &str) -> Self {
        StringReader::from(value.to_string())
    }
}

impl Read for StringReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let bytes = self.inner.as_bytes();

        // Write to the start of the buffer
        let mut i = 0;

        // Proceed either to the end of the buffer or the end of the string
        while i < buf.len() && self.cursor + i < bytes.len() {
            buf[i] = bytes[self.cursor + i];
            i += 1;
        }

        // Move the cursor forward
        self.cursor += i;

        // Return the number of bytes read
        Ok(i)
    }
}

#[cfg(test)]
mod test {
    use std::io::{BufReader, Read};

    use anyhow::Result;

    use crate::string::StringReader;

    #[test]
    fn read_a_string() -> Result<()> {
        let s = "Hello World".to_string();

        let r = StringReader::from(s.clone());

        // New bytes
        let mut read_bytes = Vec::new();
        for _ in 0..s.len() {
            read_bytes.push(0);
        }

        let mut reader = BufReader::new(r);
        reader.read(read_bytes.as_mut_slice())?;

        assert_eq!(read_bytes.as_slice(), s.as_bytes());

        Ok(())
    }

    #[test]
    fn sanity_check_partial_reads() -> Result<()> {
        let s = "Hello World";

        let r = StringReader::from(s);

        // Only one byte will be present in this buffer. We'll read one byte at a time
        let mut read_bytes = Vec::new();
        read_bytes.push(0);

        let mut reader = BufReader::new(r);

        for _ in 0..s.len() {
            let count = reader.read(read_bytes.as_mut_slice())?;
            assert_eq!(count, 1);
        }

        // There should be no more bytes to read
        let count = reader.read(read_bytes.as_mut_slice())?;
        assert_eq!(count, 0);

        Ok(())
    }
}
