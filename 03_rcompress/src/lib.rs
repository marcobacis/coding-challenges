use itertools::Itertools;
use std::{
    cmp::Ordering,
    io::{self},
};
use std::{collections::BinaryHeap, io::Read};
use std::{collections::HashMap, io::Write};

pub mod bitmanipulation;

#[derive(Debug, PartialEq, Eq)]
pub enum HuffmanTree {
    Node(usize, Box<HuffmanTree>, Box<HuffmanTree>),
    Leaf(usize, u8),
}

impl HuffmanTree {
    fn value(&self) -> usize {
        match self {
            HuffmanTree::Leaf(count, _) => *count,
            HuffmanTree::Node(count, _, _) => *count,
        }
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

fn create_huffman_tree(counts: HashMap<u8, usize>) -> Box<HuffmanTree> {
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

fn count_frequencies(s: &str) -> HashMap<u8, usize> {
    let mut counts: HashMap<u8, usize> = HashMap::new();
    for c in s.bytes() {
        *counts.entry(c).or_insert(0) += 1;
    }
    return counts;
}

fn encode_header<W: Write>(counts: HashMap<u8, usize>, writer: &mut W) -> io::Result<()> {
    writer.write(&[counts.len() as u8])?;
    for (c, count) in counts.iter().sorted() {
        writer.write(&[*c as u8])?;
        let count_bytes = (*count as u32).to_le_bytes();
        writer.write(&count_bytes)?;
    }
    Ok(())
}

fn decode_header<R: Read>(reader: &mut R) -> HashMap<u8, usize> {
    let mut counts: HashMap<u8, usize> = HashMap::new();

    // Read array size
    let mut sizebuf: [u8; 1] = [0];
    reader.read(&mut sizebuf).unwrap();
    let size = sizebuf[0];

    let mut buf = [0; 5];
    for _ in 0..size {
        // Read symbol and count
        reader.read(&mut buf).unwrap();
        let symbol = buf[0];
        let count = u32::from_le_bytes([buf[1], buf[2], buf[3], buf[4]]) as usize;
        counts.insert(symbol, count);
    }
    counts
}

#[cfg(test)]
mod tests {
    use crate::{
        count_frequencies, create_huffman_tree, decode_header, encode_header, HuffmanTree::Leaf,
        HuffmanTree::Node,
    };
    use std::{
        collections::HashMap,
        io::{BufReader, Write},
    };

    fn assert_equal_freq(counts: HashMap<u8, usize>, expected: Vec<(u8, usize)>) {
        let expected_map: HashMap<u8, usize> = expected.into_iter().collect();
        assert_eq!(counts, expected_map);
    }

    #[test]
    fn test_count_frequencies() {
        assert_equal_freq(count_frequencies("aa"), vec![(b'a', 2)]);
        assert_equal_freq(
            count_frequencies("hello, I'm testing"),
            vec![
                (b'h', 1),
                (b'e', 2),
                (b'l', 2),
                (b'o', 1),
                (b',', 1),
                (b'I', 1),
                (b'\'', 1),
                (b'm', 1),
                (b't', 2),
                (b'i', 1),
                (b' ', 2),
                (b's', 1),
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
    fn test_encode_header() {
        let counts = count_frequencies("abacba");
        let mut bytes = Vec::new();
        encode_header(counts, &mut bytes).unwrap();
        assert_eq!(
            bytes,
            vec![
                0x03, b'a', 0x03, 0x00, 0x00, 0x00, b'b', 0x02, 0x00, 0x00, 0x00, b'c', 0x01, 0x00,
                0x00, 0x00
            ]
        )
    }

    #[test]
    fn test_decode_header() {
        let encoded = vec![
            0x03, b'a', 0x03, 0x00, 0x00, 0x00, b'b', 0x02, 0x00, 0x00, 0x00, b'c', 0x01, 0x00,
            0x00, 0x00,
        ];

        let mut reader = BufReader::new(&encoded[..]);

        let counts = decode_header(&mut reader);
        assert_eq!(counts, HashMap::from([(b'a', 3), (b'b', 2), (b'c', 1)]));
    }
}
