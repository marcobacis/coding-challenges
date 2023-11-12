use encoder::HuffmanEncoder;
use header::Header;
use std::{collections::HashMap, fs::File, os::unix::prelude::MetadataExt};
use std::{
    fs,
    io::{BufReader, BufWriter, Read, Result, Seek, SeekFrom},
};

use crate::decoder::HuffmanDecoder;

mod bitmanipulation;
mod decoder;
mod encoder;
mod header;
mod tree;

fn count_frequencies<R: Read>(source: &mut R) -> Vec<(u8, usize)> {
    let mut counts_map: HashMap<u8, usize> = HashMap::new();
    let mut counts: Vec<(u8, usize)> = Vec::new();

    let mut buf = vec![0];
    loop {
        match source.read(&mut buf) {
            Ok(0) => break,
            Err(_) => break,
            Ok(_) => {
                let c = buf[0];
                if !counts_map.contains_key(&c) {
                    counts.push((c, 0));
                    counts_map.insert(c, counts.len() - 1);
                }
                let idx = counts_map[&c];
                let (_, count) = counts[idx];
                counts[idx] = (c, count + 1);
            }
        }
    }

    counts
}

pub fn compress(input_path: &str, output_path: &str) -> Result<()> {
    // Read input
    let filedata = fs::metadata(input_path)?;
    let mut input_file = BufReader::new(File::open(input_path)?);

    // Create header
    let counts = count_frequencies(&mut input_file);
    let header = Header {
        counts: counts.clone(),
        filesize: filedata.size() as usize,
    };

    // Seek back to start, as the encoder needs to read the file again
    input_file.seek(SeekFrom::Start(0));

    // Write output
    let mut output_file = BufWriter::new(File::create(output_path)?);
    header.write(&mut output_file)?;

    let encoder = HuffmanEncoder::new(&header);
    encoder.encode(&mut input_file, &mut output_file)?;

    Ok(())
}

pub fn decompress(input_path: &str, output_path: &str) -> Result<()> {
    let mut input_file = BufReader::new(File::open(input_path)?);
    let mut output_file = BufWriter::new(File::create(output_path)?);

    let header = Header::read(&mut input_file)?;

    let decoder = HuffmanDecoder::new(&header);
    decoder.decode(&mut input_file, &mut output_file)?;

    Ok(())
}

#[cfg(test)]
mod tests {

    use crate::{
        count_frequencies,
        HuffmanTree::{self},
    };

    fn test_frequencies(s: &str, expected: Vec<(u8, usize)>) {
        let mut buf = s.as_bytes();
        let counts = count_frequencies(&mut buf);
        assert_eq!(counts, expected);
    }

    #[test]
    fn test_count_frequencies() {
        test_frequencies("aa", vec![(b'a', 2)]);
        test_frequencies("abcdbca", vec![(b'a', 2), (b'b', 2), (b'c', 2), (b'd', 1)]);
        test_frequencies(
            "hello, I'm testing",
            vec![
                (b'h', 1),
                (b'e', 2),
                (b'l', 2),
                (b'o', 1),
                (b',', 1),
                (b' ', 2),
                (b'I', 1),
                (b'\'', 1),
                (b'm', 1),
                (b't', 2),
                (b's', 1),
                (b'i', 1),
                (b'n', 1),
                (b'g', 1),
            ],
        );
    }
}
