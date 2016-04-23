//
// MSX CAS Packager
// Copyright (c) 2015 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

extern crate byteorder;
extern crate docopt;
extern crate rustc_serialize;

#[cfg(test)]
extern crate quickcheck;

mod args;
mod tape;
mod wav;

use std::convert::From;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::io::Error as IoError;
use std::path::Path;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[derive(Debug)]
enum Error {
    InvalidInputFile(String),
    Io(IoError)
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Error { Error::Io(e) }
}

type Result<T> = std::result::Result<T, Error>;

#[allow(dead_code)]
fn main() {
    let cmd = args::parse();
    let result = match cmd {
        args::Command::Version => print_version(),
        args::Command::List(path) => list_files(&path[..]),
        args::Command::Add(path, files) =>  add_files(&path[..], &files[..]),
        args::Command::Extract(path) => extract_all(&path[..]),
        args::Command::Export(path, output) => export(&*path, &*output),
    };
    if result.is_err() {
        match result.unwrap_err() {
            Error::InvalidInputFile(file) => 
                println!("Error: {} is not a valid input file", file),
            Error::Io(e) => 
                println!("Error: IO operation failed: {}", e),
        }
    }
}

fn print_version() -> Result<()> {
    println!("MSX CAS Packager (MCP) v{}", VERSION);
    println!("Copyright (C) 2015 Alvaro Polo");
    println!("");
    println!("This program is subject to the terms of the Mozilla Public License v2.0.");
    println!("");
    Ok(())
}

macro_rules! read_file {
    ($name: expr) => ({
        let mut file = try!(File::open($name));
        let mut data: Vec<u8> = vec![];
        try!(file.read_to_end(&mut data).map(|_| data))
    });
}

macro_rules! open_tape {
    ($path: expr) => ({
        let mut tape_file = try!(File::open($path));
        try!(tape::Tape::read(&mut tape_file))
    })
}

macro_rules! open_or_create_tape {
    ($path: expr) => ({
        match File::open($path) {
            Ok(mut f) => try!(tape::Tape::read(&mut f).map(|t| (t, true))),
            Err(_) => (tape::Tape::new(), false),
        }
    })
}

macro_rules! file_name {
    ($file: expr) => ({
        let src_name = Path::new($file).file_name().unwrap().to_str().unwrap();
        let fname = &src_name[..src_name.len() - 4];
        if fname.len() > 6 {
            println!("Warning: filename {} is too long, truncating", src_name);
        }
        tape::file_name(fname)
    })
}

fn file_name(path: &str) -> Result<[u8;6]> {
    let src_name = try!(Path::new(path)
        .file_name()
        .map_or(None, |f| f.to_str())
        .ok_or(Error::InvalidInputFile(path.to_string())));
    let fname = &src_name[..src_name.len() - 4];
    if fname.len() > 6 {
        println!("Warning: filename {} is too long, truncating", src_name);
    }
    Ok(tape::file_name(fname))
}

fn list_files(path: &str) -> Result<()> {
    let tape = open_tape!(path);
    for file in tape.files() {
        match file {
            tape::File::Bin(name, begin, end, start, data) => {
                println!("bin    | {:6} | {:5} bytes | [0x{:x},0x{:x}]:0x{:x}",
                    name, data.len(), begin, end, start);
            },
            tape::File::Basic(name, data) => {
                println!("basic  | {:6} | {:5} bytes |", name, data.len());
            },
            tape::File::Ascii(name, data) => {
                let nbytes = data.iter().fold(0, |size, chunk| size + chunk.len());
                println!("ascii  | {:6} | {:5} bytes |", name, nbytes);
            },
            tape::File::Custom(data) => {
                println!("custom |        | {:5} bytes |", data.len());
            }
        };
    }
    Ok(())
}

fn extract_all(path: &str) -> Result<()> {
    let tape = open_tape!(path);
    let mut next_custom = 0;
    for file in tape.files() {
        let out_path = file.name()
            .map(|n| n.to_string())
            .unwrap_or_else(|| format!("custom.{:03}", { next_custom += 1; next_custom }));
        print!("Extracting {}... ", out_path);
        try!(extract_file(&file, &out_path[..]));
        println!("Done");
    }
    Ok(())
}

fn extract_file(file: &tape::File, out_path: &str) -> Result<()> {
    let mut ofile = try!(File::create(out_path));
    match file {
        &tape::File::Bin(_, _, _, _, data) => {
            // First, write the BIN file ID byte not present in cassete
            try!(ofile.write_all(&[0xfe]));
            try!(ofile.write_all(data));
        },
        &tape::File::Basic(_, data) => {
            try!(ofile.write_all(data));
        },
        &tape::File::Ascii(_, ref chunks) => {
            for chunk in chunks {
                let last = chunk.iter().position(|b| *b == 0x1a).unwrap_or(chunk.len());
                try!(ofile.write_all(&chunk[..last]));
            }
        },
        &tape::File::Custom(ref data) => {
            try!(ofile.write_all(data));
        },
    }
    Ok(())
}

fn add_files(path: &str, files: &[String]) -> Result<()> {
    let (mut tape, tape_exist) = open_or_create_tape!(path);
    for fname in files {
        print!("Adding {}... ", fname);
        if fname.ends_with(".bin") {
            try!(add_bin_file(&mut tape, &fname[..]));
        } else if fname.ends_with(".asc") {
            try!(add_ascii_file(&mut tape, &fname[..]));
        } else if fname.ends_with(".bas") {
            try!(add_basic_file(&mut tape, &fname[..]));
        } else {
            try!(add_custom_file(&mut tape, &fname[..]));
        }
        println!("Done");
    }
    let otmp = format!("{}.tmp", path);
    try!(save_tape(&tape, &otmp[..]));
    if tape_exist {
        try!(fs::remove_file(path));
    }
    try!(fs::rename(otmp, path));
    Ok(())
}

fn add_bin_file(tape: &mut tape::Tape, file: &str) -> Result<()> {
    let data = &read_file!(file)[..];
    // Skip bin file ID byte if present
    let bytes = if data[0] == 0xfe { &data[1..] } else { data };
    let fname = try!(file_name(file));
    tape.append_bin(&fname, bytes);
    Ok(())
}

fn add_basic_file(tape: &mut tape::Tape, file: &str) -> Result<()> {
    let data = &read_file!(file)[..];
    let fname = try!(file_name(file));
    tape.append_basic(&fname, data);
    Ok(())
}

fn add_ascii_file(tape: &mut tape::Tape, file: &str) -> Result<()> {
    let data = &read_file!(file)[..];
    let fname = try!(file_name(file));
    tape.append_ascii(&fname, data);
    Ok(())
}

fn add_custom_file(tape: &mut tape::Tape, file: &str) -> Result<()> {
    let data = &read_file!(file)[..];
    tape.append_custom(data);
    Ok(())
}

fn save_tape(tape: &tape::Tape, file: &str) -> Result<()> {
    let mut ofile = try!(File::create(&file));
    for block in tape.blocks() {
        try!(ofile.write_all(block.data()));
    }
    Ok(())
}

fn export(cas_path: &str, wav_path: &str) -> Result<()> {
    let tape = open_tape!(cas_path);
    let mut exporter = wav::Exporter::new();
    let mut wav_file = try!(File::create(wav_path));

    for (block, i) in tape.blocks().iter().zip(0..tape.blocks().len()) {
        print!("Encoding block {}... ", i);
        let mut nbytes = 0;
        if block.is_file_header() {
            nbytes += exporter.write_long_silence().ok().unwrap();
            nbytes += exporter.write_long_header().ok().unwrap();
        } else {
            nbytes += exporter.write_short_silence().ok().unwrap();
            nbytes += exporter.write_short_header().ok().unwrap();
        }
        nbytes += exporter.write_data(block.data_without_prefix()).ok().unwrap();
        println!("{} KiB", nbytes / 1024);
    }
    exporter.export(&mut wav_file).ok();
    Ok(())
}

#[cfg(test)]
mod test {

    use super::file_name;

    #[test]
    fn should_extract_correct_filename() {
        assert_eq!("foo   ".as_bytes(), file_name("foo.bin").ok().unwrap());
        assert_eq!("foo   ".as_bytes(), file_name("foo.asc").ok().unwrap());
        assert_eq!("foo   ".as_bytes(), file_name("foo.bas").ok().unwrap());
        assert_eq!("foo   ".as_bytes(), file_name("../foo.bin").ok().unwrap());
        assert_eq!("foo   ".as_bytes(), file_name("../foo.asc").ok().unwrap());
        assert_eq!("foo   ".as_bytes(), file_name("../foo.bas").ok().unwrap());
    }

    #[test]
    fn should_fail_extract_invalid_filename() {
        assert!(file_name(".").is_err());
        assert!(file_name("..").is_err());
        assert!(file_name("./").is_err());
        assert!(file_name("../").is_err());
        assert!(file_name("/.").is_err());
        assert!(file_name("/..").is_err());
    }
}
