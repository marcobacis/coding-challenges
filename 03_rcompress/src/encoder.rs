use std::io::{Read, Result, Write};

use crate::{bitmanipulation::BitWriter, header::Header, tree::HuffmanTree};

pub(crate) struct HuffmanEncoder {
    tree: HuffmanTree,
}

impl HuffmanEncoder {
    pub fn new(header: &Header) -> Self {
        HuffmanEncoder {
            tree: HuffmanTree::create(&header.counts),
        }
    }

    pub fn encode<R: Read, W: Write>(&self, source: &mut R, sink: &mut W) -> Result<()> {
        let mut writer = BitWriter::new(sink);
        let mut buf = vec![0];

        loop {
            match source.read(&mut buf) {
                Ok(0) => break,
                Err(_) => break,
                Ok(_) => {
                    let encoded = self.tree.encode(buf[0]).unwrap();
                    writer.write(encoded.as_slice())?;
                }
            }
        }
        writer.flush()?;

        Ok(())
    }
}
