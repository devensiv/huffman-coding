use crate::bitutils::Symbol;
use std::collections::HashMap;
use std::fmt::Display;
use std::io::{self, prelude::*};

const HEADER_START: &[u8; 30] = b"----- rxh tree start V2 -----\n";
const HEADER_END: &[u8; 29] = b"\n----- rxh tree end V2 -----\n";
const INVALID_VERSION: &str = r#"file does not contain a valid rxh tree start signature.
If the file contains a valid signature from a prior version you may find a version of this program thats compatible with this file here: https://github.com/devensiv/huffman-coding"#;

pub enum Tree {
    Root(Box<Tree>, Box<Tree>),
    Leaf(u8, usize),
    Node(Box<Tree>, Box<Tree>, usize),
}

impl Tree {
    fn fill_conversion_map(node: &Tree, mut sym: Symbol, map: &mut HashMap<u8, Symbol>) {
        match node {
            Tree::Root(left, right) | Tree::Node(left, right, _) => {
                let mut lsym = sym.clone();
                lsym.append_bit(false);
                Tree::fill_conversion_map(left, lsym, map);
                sym.append_bit(true);
                Tree::fill_conversion_map(right, sym, map);
            }
            Tree::Leaf(key, _) => {
                map.insert(*key, sym);
            }
        }
    }

    /// creates the encoding map from bytes to huffman symbols from the tree contained under `self`
    pub fn make_conversion_map(&self) -> HashMap<u8, Symbol> {
        let mut map = HashMap::new();
        Tree::fill_conversion_map(
            self,
            Symbol {
                bytes: Vec::new(),
                bitpos: 0,
                bytepos: 0,
            },
            &mut map,
        );
        map
    }

    pub fn store(&self, file: &mut impl Write) -> Result<(), io::Error> {
        match self {
            Tree::Leaf(key, _) => {
                file.write_all(&[1, *key])?;
            }
            Tree::Node(left, right, _) => {
                assert_eq!(file.write(&[0])?, 1);
                left.store(file)?;
                right.store(file)?;
            }
            Tree::Root(left, right) => {
                file.write_all(HEADER_START)?;
                assert_eq!(file.write(&[255])?, 1);
                left.store(file)?;
                right.store(file)?;
                file.write_all(HEADER_END)?;
            }
        }
        Ok(())
    }

    pub fn try_load(input: &mut impl Read) -> Result<Tree, io::Error> {
        let mut buffer = [0u8; HEADER_START.len()]; //header start is 29 bytes
        input.read_exact(&mut buffer)?;
        if &buffer != HEADER_START {
            return Err(io::Error::new(io::ErrorKind::InvalidData, INVALID_VERSION));
        }

        let result = Tree::load(input);

        let mut buffer = [0u8; HEADER_END.len()]; //header end is 28 bytes
        input.read_exact(&mut buffer)?;
        if &buffer != HEADER_END {
            return Err(io::Error::new(io::ErrorKind::InvalidData, INVALID_VERSION));
        }
        result
    }

    fn load(input: &mut impl Read) -> Result<Tree, io::Error> {
        let mut buffer = [0u8];
        input.read_exact(&mut buffer)?;
        match buffer[0] {
            0 => Ok(Tree::Node(
                Box::new(Tree::load(input)?),
                Box::new(Tree::load(input)?),
                0,
            )),
            1 => {
                let mut buffer = [0u8];
                input.read_exact(&mut buffer)?;
                Ok(Tree::Leaf(buffer[0], 0))
            }
            255 => Ok(Tree::Root(
                Box::new(Tree::load(input)?),
                Box::new(Tree::load(input)?),
            )),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Tree format broken",
            )),
        }
    }

    pub fn mktree(mut freq: Vec<Tree>) -> Tree {
        loop {
            let mut bigger = (0, usize::MAX);
            let mut smaller = (0, usize::MAX);
            for (num, node) in freq.iter().enumerate() {
                match node {
                    Tree::Leaf(_, value) | Tree::Node(_, _, value) => {
                        if value < &bigger.1 {
                            if value < &smaller.1 {
                                bigger = smaller;
                                smaller = (num, *value);
                            } else {
                                bigger = (num, *value);
                            }
                        }
                    }
                    Tree::Root(_, _) => (),
                }
            }
            let left;
            let right;
            if smaller.0 > bigger.0 {
                left = freq.remove(smaller.0);
                right = freq.remove(bigger.0);
            } else {
                right = freq.remove(bigger.0);
                left = freq.remove(smaller.0);
            }
            if freq.is_empty() {
                return Tree::Root(Box::new(left), Box::new(right));
            }

            freq.push(Tree::Node(
                Box::new(left),
                Box::new(right),
                smaller.1 + bigger.1,
            ));
        }
    }
}

impl Display for Tree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn show(tree: &Tree, depth: usize, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match tree {
                Tree::Leaf(key, val) => {
                    writeln!(f, "{}leaf {} value {}", " ".repeat(depth), key, val)
                }
                Tree::Node(left, right, val) => {
                    writeln!(f, "{}node {}", " ".repeat(depth), val)?;
                    show(left, depth + 1, f)?;
                    show(right, depth + 1, f)
                }
                Tree::Root(left, right) => {
                    show(left, depth + 1, f)?;
                    show(right, depth + 1, f)
                }
            }
        }
        show(self, 0, f)
    }
}
