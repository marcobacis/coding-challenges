use std::io::{self, BufReader, Read};

struct BitReader<R: Read> {
    index: usize,
    current: u8,
    source: BufReader<R>,
}

impl<R: Read> BitReader<R> {
    pub fn new(source: R) -> Self {
        let mut reader = BitReader {
            index: 8,
            current: 0,
            source: BufReader::new(source),
        };
        reader
    }

    pub fn read(&mut self, out: &mut [u8]) -> io::Result<u8> {
        let mut bits_read: u8 = 0;
        for _ in 0..out.len() {
            if self.index > 7 {
                // Read next byte
                let mut buf = [0];
                match self.source.read(&mut buf) {
                    Ok(0) => return Ok(bits_read),
                    Ok(_) => {
                        self.current = buf[0];
                        self.index = 0;
                    }
                    Err(e) => return Err(e),
                }
            }
            let val = (self.current >> self.index) & 0x01;
            self.index += 1;
            out[bits_read as usize] = val;
            bits_read += 1;
        }
        Ok(bits_read)
    }
}

#[cfg(test)]
mod tests {
    use super::BitReader;

    #[test]
    fn can_read_from_one_byte() {
        let s: [u8; 1] = [0b10101101];
        let mut reader = BitReader::new(&s[..]);

        let mut buf: [u8; 8] = [0; 8];
        let expected = [0x01, 0x00, 0x01, 0x01, 0x00, 0x01, 0x00, 0x01];

        let bits_read = reader.read(&mut buf).unwrap();
        assert_eq!(bits_read, 8);
        assert!(buf.iter().eq(expected.iter()));
    }

    #[test]
    fn can_read_from_more_bytes() {
        let s: [u8; 2] = [0b10101101, 0b10101010];
        let mut reader = BitReader::new(&s[..]);

        let mut buf: [u8; 16] = [0; 16];
        let expected: [u8; 16] = [1, 0, 1, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 1];

        let bits_read = reader.read(&mut buf).unwrap();
        assert_eq!(bits_read, 16);
        assert!(buf.iter().eq(expected.iter()));
    }

    #[test]
    fn read_empty() {
        let s: [u8; 0] = [];
        let mut buf: [u8; 1] = [0];
        let mut reader = BitReader::new(&s[..]);
        assert_eq!(reader.read(&mut buf).unwrap(), 0);
    }
}
