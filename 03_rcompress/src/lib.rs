use bitmanipulation::BitReader;
use header::Header;
use std::{
    cmp::Ordering,
    fs::File,
    io::{BufReader, BufWriter, Result},
};
use std::{collections::BinaryHeap, io::Read};
use std::{collections::HashMap, io::Write};

use crate::bitmanipulation::BitWriter;

pub mod bitmanipulation;
mod header;

#[derive(Debug, PartialEq, Eq)]
pub enum HuffmanTree {
    Node(usize, Box<HuffmanTree>, Box<HuffmanTree>),
    Leaf(usize, u8),
}

impl HuffmanTree {
    pub fn value(&self) -> usize {
        match self {
            HuffmanTree::Leaf(count, _) => *count,
            HuffmanTree::Node(count, _, _) => *count,
        }
    }

    pub fn encode(&self, value: u8) -> Option<Vec<u8>> {
        match self {
            HuffmanTree::Leaf(_, val) => {
                if *val == value {
                    return Some(vec![]);
                }
                return None;
            }
            HuffmanTree::Node(_, left, right) => {
                if let Some(mut res) = left.encode(value) {
                    res.insert(0, 0);
                    return Some(res);
                }
                if let Some(mut res) = right.encode(value) {
                    res.insert(0, 1);
                    return Some(res);
                }
            }
        }
        None
    }
}

impl Ord for HuffmanTree {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (HuffmanTree::Leaf(a, _), HuffmanTree::Leaf(b, _)) => b.cmp(a),
            (HuffmanTree::Leaf(a, _), HuffmanTree::Node(b, _, _)) => b.cmp(a),
            (HuffmanTree::Node(a, _, _), HuffmanTree::Leaf(b, _)) => b.cmp(a),
            (HuffmanTree::Node(a, _, _), HuffmanTree::Node(b, _, _)) => b.cmp(a),
        }
    }
}

impl PartialOrd for HuffmanTree {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

struct HuffmanDecoder<'a> {
    tree: &'a HuffmanTree,
    current: &'a HuffmanTree,
}

#[derive(Debug, Eq, PartialEq)]
pub enum HuffmanDecodeResult {
    Decoding,
    Decoded(u8),
}

impl<'a> HuffmanDecoder<'_> {
    pub fn new(tree: &HuffmanTree) -> HuffmanDecoder {
        HuffmanDecoder {
            tree: tree,
            current: tree,
        }
    }

    pub fn step(&mut self, bit: u8) -> HuffmanDecodeResult {
        match self.current {
            HuffmanTree::Node(_, left, right) => {
                if bit == 0 {
                    self.current = left;
                } else {
                    self.current = right;
                }
            }
            HuffmanTree::Leaf(_, _) => {
                self.current = self.tree;
            }
        }

        match self.current {
            HuffmanTree::Node(_, _, _) => HuffmanDecodeResult::Decoding,
            HuffmanTree::Leaf(_, value) => {
                self.current = self.tree;
                HuffmanDecodeResult::Decoded(*value)
            }
        }
    }
}

pub fn create_huffman_tree(counts: Vec<(u8, usize)>) -> Box<HuffmanTree> {
    let mut heap = BinaryHeap::new();
    for (elem, count) in counts {
        heap.push(Box::new(HuffmanTree::Leaf(count, elem)));
    }

    while heap.len() > 1 {
        let left = heap.pop().unwrap();
        let right = heap.pop().unwrap();
        heap.push(Box::new(HuffmanTree::Node(
            (*left).value() + (*right).value(),
            left,
            right,
        )));
    }
    return heap.pop().unwrap();
}

fn count_frequencies(s: &str) -> Vec<(u8, usize)> {
    let mut counts_map: HashMap<u8, usize> = HashMap::new();
    let mut counts: Vec<(u8, usize)> = Vec::new();
    for c in s.bytes() {
        if !counts_map.contains_key(&c) {
            counts.push((c, 0));
            counts_map.insert(c, counts.len() - 1);
        }
        let idx = counts_map[&c];
        let (_, count) = counts[idx];
        counts[idx] = (c, count + 1);
    }
    counts
}

pub fn compress(input_path: &str, output_path: &str) -> Result<()> {
    // Read input
    let mut file = BufReader::new(File::open(input_path)?);
    let mut buffer = String::new();
    file.read_to_string(&mut buffer)?;

    // Create tree
    let counts = count_frequencies(&buffer);
    let tree = create_huffman_tree(counts.clone());

    // Create output and encode
    let mut output_file = BufWriter::new(File::create(output_path)?);

    let header = Header {
        counts: counts.clone(),
        filesize: buffer.len(),
    };
    {
        let header = &header;
        header.write(&mut output_file)
    }?;

    let mut writer = BitWriter::new(output_file);

    for c in buffer.bytes() {
        let encoded = tree.encode(c).unwrap();
        writer.write(encoded.as_slice())?;
    }
    writer.flush()?;

    Ok(())
}

pub fn decompress(input_path: &str, output_path: &str) -> Result<()> {
    let mut input_file = BufReader::new(File::open(input_path)?);

    let header = Header::read(&mut input_file)?;
    let tree = create_huffman_tree(header.counts);

    let mut reader = BitReader::new(&mut input_file);
    let mut output_file = BufWriter::new(File::create(output_path)?);

    // Decode file with reader and write to new file - TODO extract?
    let mut decoder = HuffmanDecoder::new(&tree);
    let mut buf = vec![0];
    let mut out_buf = vec![0];
    let mut bytes_written = 0;
    loop {
        match reader.read(&mut buf) {
            Ok(0) => break,
            Err(_) => break,
            Ok(_) => match decoder.step(buf[0]) {
                HuffmanDecodeResult::Decoding => {}
                HuffmanDecodeResult::Decoded(value) => {
                    out_buf[0] = value;
                    output_file.write(&out_buf)?;
                    bytes_written += 1;
                    if bytes_written >= header.filesize {
                        break;
                    }
                }
            },
        }
    }
    output_file.flush()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;

    use crate::{
        count_frequencies, create_huffman_tree, HuffmanDecodeResult, HuffmanDecoder,
        HuffmanTree::Leaf, HuffmanTree::Node,
    };

    #[test]
    fn test_count_frequencies() {
        assert_eq!(count_frequencies("aa"), vec![(b'a', 2)]);
        assert_eq!(
            count_frequencies("abcdbca"),
            vec![(b'a', 2), (b'b', 2), (b'c', 2), (b'd', 1)]
        );
        assert_eq!(
            count_frequencies("hello, I'm testing"),
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

    #[test]
    fn test_create_huffman_tree() {
        let counts = count_frequencies("abacba");
        let tree = create_huffman_tree(counts);
        assert_eq!(
            tree,
            Box::new(Node(
                6,
                Box::new(Leaf(3, b'a')),
                Box::new(Node(3, Box::new(Leaf(1, b'c')), Box::new(Leaf(2, b'b'))))
            ))
        );
    }

    #[test]
    fn test_encode() {
        let counts = count_frequencies("abacba");
        let tree = create_huffman_tree(counts);

        assert_eq!(Some(vec![0]), tree.encode(b'a'));
        assert_eq!(Some(vec![1, 0]), tree.encode(b'c'));
        assert_eq!(Some(vec![1, 1]), tree.encode(b'b'));
    }

    #[test]
    fn test_decode() {
        let counts = count_frequencies("abacba");
        let tree = create_huffman_tree(counts);
        let mut decoder = HuffmanDecoder::new(&(*tree));

        assert_eq!(HuffmanDecodeResult::Decoded(b'a'), decoder.step(0));

        assert_eq!(HuffmanDecodeResult::Decoding, decoder.step(1));
        assert_eq!(HuffmanDecodeResult::Decoded(b'b'), decoder.step(1));

        assert_eq!(HuffmanDecodeResult::Decoding, decoder.step(1));
        assert_eq!(HuffmanDecodeResult::Decoded(b'c'), decoder.step(0));
    }
}
