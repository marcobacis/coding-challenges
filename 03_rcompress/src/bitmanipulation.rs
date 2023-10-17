use std::io::{self, BufReader, BufWriter, Read, Write};

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

struct BitWriter<W: Write> {
    sink: BufWriter<W>,
    current: u8,
    index: usize,
}

impl<W: Write> BitWriter<W> {
    pub fn new(sink: W) -> Self {
        BitWriter {
            sink: BufWriter::new(sink),
            current: 0,
            index: 0,
        }
    }

    pub fn write(&mut self, bits: &[u8]) -> io::Result<usize> {
        for bit in bits {
            if self.index > 7 {
                self.sink.write(&[self.current])?;
                self.current = 0;
                self.index = 0;
            }
            self.current = self.current | ((bit & 0x01) << self.index);
            self.index += 1;
        }
        Ok(bits.len())
    }

    pub fn flush(&mut self) -> io::Result<()> {
        if self.index > 0 {
            self.sink.write(&[self.current])?;
            return self.sink.flush();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use crate::bitmanipulation::BitWriter;

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

    #[test]
    fn write_empty() {
        let mut output = Vec::new();
        {
            let mut writer = BitWriter::new(&mut output);
            writer.flush().unwrap();
        }

        assert_eq!(0, output.len());
    }

    #[test]
    fn write_some_bits() {
        let mut output = Vec::new();
        {
            let mut writer = BitWriter::new(&mut output);
            writer.write(&[0x01]).unwrap();
            writer.write(&[0x00]).unwrap();
            writer.write(&[0x01]).unwrap();
            writer.flush().unwrap();
        }

        assert_eq!(1, output.len());
        assert_eq!([0x05], output[0..1]);
    }

    #[test]
    fn write_more_than_one_bit_at_a_time() {
        let mut output = Vec::new();
        {
            let mut writer = BitWriter::new(&mut output);
            writer.write(&[0x01, 0x01, 0x01]).unwrap();
            writer.flush().unwrap();
        }

        assert_eq!(1, output.len());
        assert_eq!([0x07], output[0..1]);
    }

    #[test]
    fn write_more_than_one_byte() {
        let mut output = Vec::new();
        {
            let mut writer = BitWriter::new(&mut output);
            let written = writer
                .write(&[
                    0x01, 0x01, 0x01, 0x00, 0x00, 0x01, 0x00, 0x01, //first  byte
                    0x00, 0x00, 0x00, 0x01, // second byte
                ])
                .unwrap();
            writer.flush().unwrap();

            assert_eq!(12, written);
        }

        assert_eq!(2, output.len());
        assert_eq!([0xA7, 0x08], output[0..2]);
    }

    #[test]
    fn write_and_read() {
        let input = [
            0x01, 0x01, 0x01, 0x00, 0x00, 0x01, 0x00, 0x01, //first  byte
            0x00, 0x00, 0x00, 0x01, // second byte
        ];
        let mut buffer = Vec::new();
        {
            let mut writer = BitWriter::new(&mut buffer);
            writer.write(&input).unwrap();
            writer.flush().unwrap();
        }

        let mut reader = BitReader::new(&buffer[..]);

        let mut output = [0; 12];
        reader.read(&mut output).unwrap();

        assert_eq!(output, input);
    }
}
