//
// MSX CAS Packager
// Copyright (c) 2015 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io::Read;
use std::io::Error as IoError;
use std::str::from_utf8;

#[derive(Debug)]
pub struct Block { data: Vec<u8>, }

impl Block {
 
    pub fn is_bin_header(&self) -> bool {
        self.data.len() >= 16 && 
            self.data[..10] == [0xd0, 0xd0, 0xd0, 0xd0, 0xd0, 0xd0, 0xd0, 0xd0, 0xd0, 0xd0]
    }

    pub fn file_name(&self) -> Option<&str> {
        if self.is_bin_header() { from_utf8(&self.data[10..16]).ok() } 
        else { None }
    }
}

#[derive(Debug, PartialEq)]
pub enum File<'a> {
	Bin(String, usize, usize, usize, &'a[u8]),
    Custom(&'a[u8])
}

pub struct Files<'a> {
    tape: &'a Tape,
    i: usize,
}

impl<'a> Iterator for Files<'a> {
    
    type Item = File<'a>;
    
    fn next(&mut self) -> Option<File<'a>> {
        while self.i < self.tape.blocks.len() {
            let block = &self.tape.blocks[self.i];
            if block.is_bin_header() {
                let name = block.file_name().unwrap().to_string();
                let content = &self.tape.blocks[self.i+1].data;
                let begin = (content[0] as usize) | (content[1] as usize) << 8;
                let end = (content[2] as usize) | (content[3] as usize) << 8;
                let start = (content[4] as usize) | (content[5] as usize) << 8;
                let data = &content[6..];
                self.i = self.i + 2;
                return Some(File::Bin(name, begin, end, start, data));
            } else {
                self.i = self.i + 1;
                return Some(File::Custom(&block.data[..]));
            }
        }
        None
    }
}

#[derive(Debug)]
pub struct Tape {
    blocks: Vec<Block>,
}

#[derive(Debug)]
pub enum LoadError {
	Io(IoError),
}

impl From<IoError> for LoadError {
	fn from(e: IoError) -> LoadError { LoadError::Io(e) }
}

impl Tape {

	#[allow(dead_code)]
	pub fn read(input: &mut Read) -> Result<Tape, LoadError> {
		let mut bytes: Vec<u8> = vec![];
		try!(input.read_to_end(&mut bytes));
		Tape::from_bytes(&bytes[..])
	}

	pub fn from_bytes(bytes: &[u8]) -> Result<Tape, LoadError> {
		Ok(Tape { blocks: Tape::parse_blocks(bytes) })
	}

    pub fn files(&self) -> Files { Files { tape: self, i: 0 } }

    fn parse_blocks(bytes: &[u8]) -> Vec<Block> {
        let mut blocks: Vec<Block> = vec![];
        let mut hindex: Vec<usize> = vec![];
        let mut i = 0;

        // First of all, we compute the indices of all block headers.
        for win in bytes.windows(8) {
            if win == [0x1f, 0xa6, 0xde, 0xba, 0xcc, 0x13, 0x7d, 0x74] {
                hindex.push(i);
            }
            i = i + 1;
        }

        // Now we use the block header indices to generate the blocks
        for i in 0..hindex.len() {
            let from = hindex[i] + 8;
            let to = if i == hindex.len() - 1 { bytes.len() } else { hindex[i+1] };
            let block = Block { data: Vec::from(&bytes[from..to]) };
            blocks.push(block);
        }
        blocks
    }
}

#[cfg(test)]
mod test {

    use super::*;

    macro_rules! assert_bin {
        ($f:expr, $n:expr, $b:expr, $e:expr, $s:expr, $d:expr) => {
            match $f {
            &File::Bin(ref name, begin, end, start, data) => {
                assert_eq!($n, name);
                assert_eq!($b, begin);
                assert_eq!($e, end);
                assert_eq!($s, start);
                assert_eq!($d, &data[..]);
            },
            _ => panic!("unexpected file"),
        }
        }
    }

    #[test]
    fn should_load_empty_tape() {
    	let bytes: Vec<u8> = vec![];
    	let tape = Tape::from_bytes(&bytes);
        assert!(tape.is_ok());
        assert_eq!(None, tape.unwrap().files().next());
    }

    #[test]
    fn should_load_bin_file() {
        let bytes: Vec<u8> = vec![
            0x1f, 0xa6, 0xde, 0xba, 0xcc, 0x13, 0x7d, 0x74,
            0xd0, 0xd0, 0xd0, 0xd0, 0xd0, 0xd0, 0xd0, 0xd0, 0xd0, 0xd0,
            0x46, 0x4f, 0x4f, 0x42, 0x41, 0x52,
            0x1f, 0xa6, 0xde, 0xba, 0xcc, 0x13, 0x7d, 0x74,
            0x00, 0x80, 0x08, 0x80, 0x00, 0x00,
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07
        ];
        let tape = Tape::from_bytes(&bytes).unwrap();        
        for f in tape.files() {
            assert_bin!(&f, 
                "FOOBAR", 0x8000, 0x8008, 0x0000, &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07]);
        } 
    } 
}
