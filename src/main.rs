//
// MSX CAS Packager
// Copyright (c) 2015 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.


extern crate byteorder;
extern crate docopt;
#[macro_use] extern crate serde_derive;

#[cfg(test)] extern crate quickcheck;
#[cfg(test)] extern crate tempdir;

mod args;
mod file;
mod tape;
mod wav;

use std::convert::From;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::Path;

use tape::Tape;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[derive(Debug)]
enum Error {
    Io(io::Error)
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error { Error::Io(e) }
}

type Result<T> = std::result::Result<T, Error>;

#[allow(dead_code)]
fn main() {
    let cmd = args::parse();
    let result = match cmd {
        args::Command::Version => print_version(),
        args::Command::List(path) => list_files(&path),
        args::Command::Add(path, files) =>  {
            let input_files: Vec<&Path> = files.iter().map(|f| f.as_path()).collect();
            add_files(&path, &input_files)
        },
        args::Command::Extract(path) => extract_all(&path),
        args::Command::Export(path, output) => export(&*path, &*output),
    };
    if result.is_err() {
        match result.unwrap_err() {
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

fn list_files(path: &Path) -> Result<()> {
    let tape = tape::Tape::from_file(path)?;
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

fn extract_all(path: &Path) -> Result<()> {
    let tape = tape::Tape::from_file(path)?;
    let mut next_custom = 0;
    for file in tape.files() {
        let out_path = file.name()
            .map(|n| n.to_string())
            .unwrap_or_else(|| format!("custom.{:03}", { next_custom += 1; next_custom }));
        print!("Extracting {}... ", out_path);
        extract_file(&file, Path::new(&out_path))?;
        println!("Done");
    }
    Ok(())
}

fn extract_file(file: &tape::File, out_path: &Path) -> Result<()> {
    let (out_filename, clash) = file::unique_filename(out_path)?;
    if clash {
        print!("Warning: filename {:?} already exists, writing output to {:?}... ",
            out_path, out_filename);
    }
    let mut ofile = File::create(&out_filename)?;
    match file {
        &tape::File::Bin(_, _, _, _, data) => {
            // First, write the BIN file ID byte not present in cassete
            ofile.write_all(&[0xfe])?;
            ofile.write_all(data)?;
        },
        &tape::File::Basic(_, data) => {
            ofile.write_all(data)?;
        },
        &tape::File::Ascii(_, ref chunks) => {
            for chunk in chunks {
                let last = chunk.iter().position(|b| *b == 0x1a).unwrap_or(chunk.len());
                ofile.write_all(&chunk[..last])?;
            }
        },
        &tape::File::Custom(ref data) => {
            ofile.write_all(data)?;
        },
    }
    Ok(())
}

fn add_files(path: &Path, files: &[&Path]) -> Result<()> {
    let mut tape = Tape::from_file(path).unwrap_or_else(|_| Tape::new());
    for file in files {
        if file::is_bin_file(file) {
            print!("Adding binary file {:?}... ", file.as_os_str());
            add_bin_file(&mut tape, &file)?;
        } else if file::is_ascii_file(file) {
            print!("Adding ascii file {:?}... ", file.as_os_str());
            add_ascii_file(&mut tape, &file)?;
        } else if file::is_basic_file(file) {
            print!("Adding basic file {:?}... ", file.as_os_str());
            add_basic_file(&mut tape, &file)?;
        } else {
            print!("Adding custom file {:?}... ", file.as_os_str());
            add_custom_file(&mut tape, &file)?;
        }
        println!("Done");
    }
    save_tape(&tape, &path)?;
    Ok(())
}

fn add_bin_file(tape: &mut tape::Tape, file: &Path) -> Result<()> {
    let data = file::read_content(file)?;
    // Skip bin file ID byte if present
    let bytes = if data[0] == 0xfe { &data[1..] } else { &data[..] };
    let (fname, truncated) = file::file_name_of(file)?;
    if truncated {
        print!("Warning: file name truncated to {}... ", String::from_utf8_lossy(&fname));
    }
    tape.append_bin(&fname, bytes);
    Ok(())
}

fn add_basic_file(tape: &mut tape::Tape, file: &Path) -> Result<()> {
    let data = file::read_content(file)?;
    let (fname, truncated) = file::file_name_of(file)?;
    if truncated {
        print!("Warning: file name truncated to {}... ", String::from_utf8_lossy(&fname));
    }
    tape.append_basic(&fname, &data);
    Ok(())
}

fn add_ascii_file(tape: &mut tape::Tape, file: &Path) -> Result<()> {
    let data = file::read_content(file)?;
    let (fname, truncated) = file::file_name_of(file)?;
    if truncated {
        print!("Warning: file name truncated to {}... ", String::from_utf8_lossy(&fname));
    }
    tape.append_ascii(&fname, &data);
    Ok(())
}

fn add_custom_file(tape: &mut tape::Tape, file: &Path) -> Result<()> {
    let data = file::read_content(file)?;
    tape.append_custom(&data);
    Ok(())
}

fn save_tape(tape: &tape::Tape, file: &Path) -> Result<()> {
    let mut buff = Vec::with_capacity(64*1024);
    for block in tape.blocks() {
        buff.write_all(block.data())?;
    }
    file::write_content(file, &buff)?;
    Ok(())
}

fn export(cas_path: &Path, wav_path: &Path) -> Result<()> {
    let tape = Tape::from_file(cas_path)?;
    let mut exporter = wav::Exporter::new();
    let mut wav_file = File::create(wav_path)?;

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
