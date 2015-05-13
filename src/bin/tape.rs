//
// MSX CAS Packager
// Copyright (c) 2015 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io::Read;
use std::io::Error as IoError;

#[derive(Debug)]
pub enum File {
	Bin(String, usize, usize, usize, Vec<u8>),
    Custom(Vec<u8>)
}

#[derive(Debug)]
pub struct Tape {
	files: Vec<File>,
}

#[derive(Debug)]
pub enum LoadError {
	Io(IoError),
}

impl From<IoError> for LoadError {
	fn from(e: IoError) -> LoadError { LoadError::Io(e) }
}

struct Block { data: Vec<u8>, }

impl Block {

    pub fn is_bin_header(&self) -> bool {
        self.data.len() >= 16 && 
            self.data[..10] == [0xd0, 0xd0, 0xd0, 0xd0, 0xd0, 0xd0, 0xd0, 0xd0, 0xd0, 0xd0]
    }
}

impl Tape {

	#[allow(dead_code)]
	pub fn read(input: &mut Read) -> Result<Tape, LoadError> {
		let mut bytes: Vec<u8> = vec![];
		try!(input.read_to_end(&mut bytes));
		Tape::from_bytes(&bytes[..])
	}

	pub fn from_bytes(bytes: &[u8]) -> Result<Tape, LoadError> {
        let blocks = Tape::parse_blocks(bytes);
		let mut tape = Tape { files: vec![] };
        let mut i = 0;
        while i < blocks.len() {
            let block = &blocks[i];
            if block.is_bin_header() {
                let name = String::from_utf8(Vec::from(&block.data[10..16])).unwrap();
                let content = &blocks[i+1].data;
                let begin = (content[0] as usize) | (content[1] as usize) << 8;
                let end = (content[2] as usize) | (content[3] as usize) << 8;
                let start = (content[4] as usize) | (content[5] as usize) << 8;
                let data = &content[6..];
                tape.files.push(File::Bin(name, begin, end, start, Vec::from(data)));
                i = i + 2;
            } else {
                tape.files.push(File::Custom(blocks[i].data.clone()));
                i = i + 1;
            }
        }
		Ok(tape)
	}

    fn parse_blocks(bytes: &[u8]) -> Vec<Block> {
        let mut blocks: Vec<Block> = vec![];
        let mut hindex: Vec<usize> = vec![];
        let mut i = 0;

        // First of all, we compute the indices of all block headers.
        for win in bytes.windows(8) {
            if win == [0x1F, 0xA6, 0xDE, 0xBA, 0xCC, 0x13, 0x7D, 0x74] {
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

    #[test]
    fn should_load_empty_tape() {
    	let bytes: Vec<u8> = vec![];
    	let tape = Tape::from_bytes(&bytes);
        assert!(tape.is_ok());
        assert_eq!(0, tape.unwrap().files.len());
    }

    #[test]
    fn should_load_bin_file() {
        let bytes: Vec<u8> = vec![
            0x1F, 0xA6, 0xDE, 0xBA, 0xCC, 0x13, 0x7D, 0x74,
            0xd0, 0xd0, 0xd0, 0xd0, 0xd0, 0xd0, 0xd0, 0xd0, 0xd0, 0xd0,
            0x46, 0x4f, 0x4f, 0x42, 0x41, 0x52,
            0x1F, 0xA6, 0xDE, 0xBA, 0xCC, 0x13, 0x7D, 0x74,
            0x00, 0x80, 0x08, 0x80, 0x00, 0x00,
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07
        ];
        let tape = Tape::from_bytes(&bytes);
        assert!(tape.is_ok());
        let files = tape.unwrap().files;
        assert_eq!(1, files.len());
        match &files[0] {
            &File::Bin(ref name, begin, end, start, ref data) => {
                assert_eq!("FOOBAR", name);
                assert_eq!(0x8000, begin);
                assert_eq!(0x8008, end);
                assert_eq!(0x0000, start);
                assert_eq!(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07], &data[..]);
            },
            _ => panic!("unexpected file"),
        };
    }
}
