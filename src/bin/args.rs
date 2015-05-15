//
// MSX CAS Packager
// Copyright (c) 2015 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::env::args;

use docopt::Docopt;

static USAGE: &'static str = "
Usage: mcp -l <cas-file>
       mcp -x <cas-file>
       mcp --help
       mcp --version

Options:
    -h, --help                  Print this message
    -v, --version               Print the mcp version
    -l, --list                  Lists the contents of the given CAS file
    -x, --extract               Extracts the contents from the given CAS file
";

/// A command introduced through the command line interface
///
/// An enumeration of the commands accepted by `mcp`.
///
/// * `Version`, prints the `mcp` version
/// * `List(path: String)`, lists the contents of the given CAS file
/// * `Extract(path: String, item: String)`, extract the given item from the given CAS file
///
#[derive(Debug, PartialEq)]
pub enum Command {
    Version,
    List(String),
    Extract(String)
}

/// A raw description of the arguments processed by DCOPT
///
/// This is not public. Use `Command` instead.
///
#[derive(RustcDecodable)]
struct Args {
    // flag_help: bool,
    flag_version: bool,
    flag_list: bool,
    flag_extract: bool,
    arg_cas_file: String,
}

impl Args {

    /// Parse the
    pub fn cmd(&self) -> Command {
        if self.flag_version {
            Command::Version
        } else if self.flag_list {
            Command::List(self.arg_cas_file.clone())
        } else if self.flag_extract {
            Command::Extract(self.arg_cas_file.clone())
        } else {
            panic!("args are parsed in a inconsistent state")
        }
    }
}

/// Parse the arguments passed to `mcp`
///
/// Same as `parse_args(std::env::args())`.
///
#[allow(dead_code)]
pub fn parse() -> Command {
    parse_args(args())
}

/// Parse the given arguments and return the corresponding `Command` object
pub fn parse_args<I, S>(args: I) -> Command where I: Iterator<Item=S>, S: Into<String> {
    let parsed: Args = Docopt::new(USAGE)
        .and_then(|d| d.argv(args).decode())
        .unwrap_or_else(|e| e.exit());
    parsed.cmd()
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn should_parse_version() {
        let argv = ["mcp", "--version"];
        let cmd = parse_args(argv.iter().map(|a| a.to_string()));
        assert_eq!(Command::Version, cmd);
    }

    #[test]
    fn should_parse_list() {
        let argv = ["mcp", "--list", "foobar.cas"];
        let cmd = parse_args(argv.iter().map(|a| a.to_string()));
        assert_eq!(Command::List("foobar.cas".to_string()), cmd);
    }

    #[test]
    fn should_parse_extract() {
        let argv = ["mcp", "--extract", "foobar.cas"];
        let cmd = parse_args(argv.iter().map(|a| a.to_string()));
        assert_eq!(Command::Extract("foobar.cas".to_string()), cmd);
    }
}
