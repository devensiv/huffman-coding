use crate::bitutils::Symbol;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io::prelude::*;

pub enum Tree {
    Root(Box<Tree>, Box<Tree>),
    Leaf(u8, usize),
    Node(Box<Tree>, Box<Tree>, usize),
}

pub enum Walker<'a> {
    Next(&'a Box<Tree>),
    End(u8),
    No,
}

impl Tree {
    pub fn walk(&self, one: bool) -> Walker {
        match self {
            Tree::Root(left, right) | Tree::Node(left, right, _) => {
                if one {
                    match **right {
                        Tree::Leaf(key, _) => Walker::End(key),
                        _ => Walker::Next(right),
                    }
                } else {
                    match **left {
                        Tree::Leaf(key, _) => Walker::End(key),
                        _ => Walker::Next(left),
                    }
                }
            }
            Tree::Leaf(key, _) => Walker::End(*key),
        }
    }

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

    pub fn make_conversion_map(&self) -> Option<HashMap<u8, Symbol>> {
        if let Tree::Root(_, _) = self {
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
            Some(map)
        } else {
            None
        }
    }

    pub fn store(&self, file: &mut fs::File) -> Result<(), Box<dyn Error>> {
        match self {
            Tree::Leaf(key, _) => {
                file.write(&[1, *key])?;
            }
            Tree::Node(left, right, _) => {
                file.write(&[0])?;
                left.store(file)?;
                right.store(file)?;
            }
            Tree::Root(left, right) => {
                file.write(b"----- rxh tree start V1-----\n")?;
                file.write(&[255])?;
                left.store(file)?;
                right.store(file)?;
                file.write(b"\n----- rxh tree end V1-----\n")?;
            }
        }
        Ok(())
    }

    pub fn try_load(mut file: &mut fs::File) -> Result<Tree, Box<dyn Error>> {
        let mut buffer = [0u8; 29]; //header start is 29 bytes
        file.read_exact(&mut buffer)?;
        if &buffer != b"----- rxh tree start V1-----\n" {
            return Err("file does not contain a V1 rxh tree start signature")?;
        }

        let result = Tree::load(&mut file);

        let mut buffer = [0u8; 28]; //header end is 28 bytes
        file.read_exact(&mut buffer)?;
        if &buffer != b"\n----- rxh tree end V1-----\n" {
            return Err("file does not contain a V1 rxh tree end signature")?;
        }
        result
    }

    fn load(file: &mut fs::File) -> Result<Tree, Box<dyn Error>> {
        let mut buffer = [0u8];
        file.read_exact(&mut buffer)?;
        match buffer[0] {
            0 => Ok(Tree::Node(
                Box::new(Tree::load(file)?),
                Box::new(Tree::load(file)?),
                0,
            )),
            1 => {
                let mut buffer = [0u8];
                file.read_exact(&mut buffer)?;
                Ok(Tree::Leaf(buffer[0], 0))
            }
            255 => Ok(Tree::Root(
                Box::new(Tree::load(file)?),
                Box::new(Tree::load(file)?),
            )),
            _ => Err("invalid key")?,
        }
    }

    pub fn show(&self, depth: usize) {
        match self {
            Tree::Leaf(key, val) => {
                println!("{}leaf {} value {}", " ".repeat(depth), key, val)
            }
            Tree::Node(left, right, val) => {
                println!("{}node {}", " ".repeat(depth), val);
                left.show(depth + 1);
                right.show(depth + 1);
            }
            Tree::Root(left, right) => {
                left.show(depth + 1);
                right.show(depth + 1);
            }
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
            if freq.len() == 0 {
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
