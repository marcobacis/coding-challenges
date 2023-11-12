use std::io::{Read, Result, Write};

use crate::{bitmanipulation::BitReader, header::Header, tree::HuffmanTree};

pub(crate) struct HuffmanDecoder {
    tree: HuffmanTree,
    filesize: usize,
}

impl HuffmanDecoder {
    pub fn new(header: &Header) -> HuffmanDecoder {
        HuffmanDecoder {
            tree: HuffmanTree::create(&header.counts),
            filesize: header.filesize,
        }
    }

    pub fn decode<R: Read, W: Write>(&self, source: &mut R, writer: &mut W) -> Result<()> {
        let mut reader = BitReader::new(source);

        let mut buf = vec![0];
        let mut out_buf = vec![0];
        let mut bytes_written = 0;
        let mut current = &self.tree;

        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Err(_) => break,
                Ok(_) => {
                    let bit = buf[0];

                    // Move on the tree based on bit value
                    match current {
                        HuffmanTree::Node(_, left, right) => {
                            if bit == 0 {
                                current = left;
                            } else {
                                current = right;
                            }
                        }
                        HuffmanTree::Leaf(_, _) => current = &self.tree,
                    }

                    // Write to output buffer if we catched a value
                    if let HuffmanTree::Leaf(_, value) = current {
                        current = &self.tree;
                        out_buf[0] = *value;
                        writer.write(&out_buf)?;
                        bytes_written += 1;
                        if bytes_written >= self.filesize {
                            break;
                        }
                    }
                }
            }
        }
        writer.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{count_frequencies, decoder::HuffmanDecoder, header::Header};

    #[test]
    fn test_decode() {
        let mut buf = "abacba".as_bytes();
        let header = Header {
            counts: count_frequencies(&mut buf),
            filesize: 3,
        };
        let decoder = HuffmanDecoder::new(&header);

        let input: [u8; 1] = [0b00001110];
        let mut output: Vec<u8> = Vec::new();

        decoder.decode(&mut input.as_ref(), &mut output).unwrap();

        assert_eq!("abc", std::str::from_utf8(&output).unwrap());
    }
}
