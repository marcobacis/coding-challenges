use std::{cmp::Ordering, collections::BinaryHeap};

#[derive(Debug, PartialEq, Eq)]
pub enum HuffmanTree {
    Node(usize, Box<HuffmanTree>, Box<HuffmanTree>),
    Leaf(usize, u8),
}

impl HuffmanTree {
    pub fn create(counts: &Vec<(u8, usize)>) -> HuffmanTree {
        let mut heap = BinaryHeap::new();
        for (elem, count) in counts.clone() {
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
        return *(heap.pop().unwrap());
    }

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

mod tests {
    use crate::*;

    #[test]
    fn test_create_huffman_tree() {
        let mut buf = "abacba".as_bytes();
        let counts = count_frequencies(&mut buf);
        let tree = HuffmanTree::create(&counts);
        assert_eq!(
            tree,
            HuffmanTree::Node(
                6,
                Box::new(HuffmanTree::Leaf(3, b'a')),
                Box::new(HuffmanTree::Node(
                    3,
                    Box::new(HuffmanTree::Leaf(1, b'c')),
                    Box::new(HuffmanTree::Leaf(2, b'b'))
                ))
            )
        );
    }

    #[test]
    fn test_encode() {
        let mut buf = "abacba".as_bytes();
        let counts = count_frequencies(&mut buf);
        let tree = HuffmanTree::create(&counts);

        assert_eq!(Some(vec![0]), tree.encode(b'a'));
        assert_eq!(Some(vec![1, 0]), tree.encode(b'c'));
        assert_eq!(Some(vec![1, 1]), tree.encode(b'b'));
    }
}
