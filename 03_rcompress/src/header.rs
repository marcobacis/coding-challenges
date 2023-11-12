use std::io::{self, Read, Write};

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Header {
    pub counts: Vec<(u8, usize)>,
    pub filesize: usize,
}

impl Header {
    pub(crate) fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        let size_bytes = self.filesize.to_le_bytes();
        writer.write(&size_bytes)?;

        writer.write(&[self.counts.len() as u8])?;
        for (c, count) in self.counts.iter() {
            writer.write(&[*c as u8])?;
            let count_bytes = (*count as u32).to_le_bytes();
            writer.write(&count_bytes)?;
        }
        Ok(())
    }

    pub(crate) fn read<R: Read>(reader: &mut R) -> io::Result<Header> {
        let mut counts: Vec<(u8, usize)> = Vec::new();

        // Read file size
        let mut filesizebuf: [u8; 8] = [0; 8];
        reader.read(&mut filesizebuf)?;
        let filesize = usize::from_le_bytes(filesizebuf);

        // Read array size
        let mut sizebuf: [u8; 1] = [0];
        reader.read(&mut sizebuf)?;
        let size = sizebuf[0];

        let mut buf = [0; 5];
        for _ in 0..size {
            // Read symbol and count
            reader.read(&mut buf)?;
            let symbol = buf[0];
            let count = u32::from_le_bytes([buf[1], buf[2], buf[3], buf[4]]) as usize;
            counts.push((symbol, count));
        }

        Ok(Self { filesize, counts })
    }
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;

    use crate::{count_frequencies, header::Header};

    #[test]
    fn test_encode_header() {
        let mut buf = "abacba".as_bytes();
        let header = Header {
            counts: count_frequencies(&mut buf),
            filesize: 10,
        };

        let mut bytes = Vec::new();

        header.write(&mut bytes).unwrap();

        assert_eq!(
            bytes,
            vec![
                0x0A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // File Size
                0x03, // Number of frequencies
                b'a', 0x03, 0x00, 0x00, 0x00, b'b', 0x02, 0x00, 0x00, 0x00, b'c', 0x01, 0x00, 0x00,
                0x00
            ]
        )
    }

    #[test]
    fn test_decode_header() {
        let encoded = vec![
            0x75, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // File size
            0x03, // Number of frequencies
            b'a', 0x03, 0x00, 0x00, 0x00, b'b', 0x02, 0x00, 0x00, 0x00, b'c', 0x01, 0x00, 0x00,
            0x00,
        ];

        let mut reader = BufReader::new(&encoded[..]);
        let counts = Header::read(&mut reader).unwrap();
        assert_eq!(
            counts,
            Header {
                counts: vec![(b'a', 3), (b'b', 2), (b'c', 1)],
                filesize: 2677
            }
        );
    }
}
