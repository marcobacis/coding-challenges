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

    pub fn read(&mut self) -> io::Result<u8> {
        if self.index > 7 {
            // Read next byte
            let mut buf = [0];
            self.source.read(&mut buf).unwrap();
            self.current = buf[0];
            self.index = 0;
        }
        let val = (self.current >> self.index) & 0x01;
        self.index += 1;
        Ok(val)
    }
}

#[cfg(test)]
mod tests {
    use super::BitReader;

    #[test]
    fn can_read_from_one_byte() {
        let s: [u8; 1] = [0b10101101];
        let mut reader = BitReader::new(&s[..]);

        assert_eq!(reader.read().unwrap(), 0x01);
        assert_eq!(reader.read().unwrap(), 0x00);
        assert_eq!(reader.read().unwrap(), 0x01);
        assert_eq!(reader.read().unwrap(), 0x01);
        assert_eq!(reader.read().unwrap(), 0x00);
        assert_eq!(reader.read().unwrap(), 0x01);
        assert_eq!(reader.read().unwrap(), 0x00);
        assert_eq!(reader.read().unwrap(), 0x01);
    }

    #[test]
    fn can_read_from_more_bytes() {
        let s: [u8; 2] = [0b10101101, 0b10101010];
        let mut reader = BitReader::new(&s[..]);

        // read first byte
        for _ in 0..8 {
            reader.read().unwrap();
        }

        assert_eq!(reader.read().unwrap(), 0x00);
        assert_eq!(reader.read().unwrap(), 0x01);
        assert_eq!(reader.read().unwrap(), 0x00);
        assert_eq!(reader.read().unwrap(), 0x01);
        assert_eq!(reader.read().unwrap(), 0x00);
        assert_eq!(reader.read().unwrap(), 0x01);
        assert_eq!(reader.read().unwrap(), 0x00);
        assert_eq!(reader.read().unwrap(), 0x01);
    }
    /*
    #[test]
    fn read_empty() {
        let s: [u8; 0] = [];
        let mut reader = BitReader::new(&s[..]);
        assert_eq!(reader.read().is_err(), true);
    }*/
}
