//
// MSX CAS Packager
// Copyright (c) 2015 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

extern crate docopt;
extern crate rustc_serialize;

mod args;
mod tape;

use std::fs::File;

#[allow(dead_code)]
fn main() {
    let action = args::parse_args().action();
    match action {
        args::Action::Version => print_version(),
        args::Action::List(path) => list_files(&path[..]),
    };
}

fn print_version() {
    println!("MSX CAS Packager (MCP) v0.1.0");
    println!("Copyright (C) 2015 Alvaro Polo");
    println!("");
    println!("This program is subject to the terms of the Mozilla Public License v2.0.");
    println!("");
}

fn list_files(path: &str) {
    let mut tape_file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            println!("Cannot open file '{}': {}", path, e);
            return
        }
    };
    let tape = match tape::Tape::read(&mut tape_file) {
        Ok(f) => f,
        Err(e) => {
            println!("Cannot read file '{}': {}", path, e);
            return
        }
    };
    for file in tape.files() {
        match file {
            tape::File::Bin(name, begin, end, start, data) => {
                println!("bin    | {} | {:5} bytes | [0x{:x},0x{:x}]:0x{:x}", 
                    name, data.len(), begin, end, start);
            },
            tape::File::Basic(name, data) => {
                println!("basic  | {} | {:5} bytes", name, data.len());
            },
            tape::File::Ascii(name, data) => {
                let nbytes = data.iter().fold(0, |size, chunk| size + chunk.len());
                println!("ascii  | {} | {:5} bytes", name, nbytes);
            },
            tape::File::Custom(data) => {
                println!("custom |        | {:5} bytes", data.len());
            }
        };
    }
}
