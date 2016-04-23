//
// MSX CAS Packager
// Copyright (c) 2015 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::env::args;
use std::path::PathBuf;

use docopt::Docopt;

static USAGE: &'static str = "
Usage: mcp -l <cas-file>
       mcp -a <cas-file> <file>...
       mcp -x <cas-file>
       mcp -e <cas-file> <wav-file>
       mcp --help
       mcp --version

Options:
    -h, --help                  Print this message
    -v, --version               Print the mcp version
    -l, --list                  Lists the contents of the given CAS file
    -a, --add                   Add new files to a given CAS file. If the CAS
                                file does not exist, it is created.
    -x, --extract               Extracts the contents from the given CAS file
    -e, --export                Exports the CAS file into a WAV file
";

/// A command introduced through the command line interface
///
/// An enumeration of the commands accepted by `mcp`.
///
/// * `Version`, prints the `mcp` version
/// * `List(path: PathBuf)`, lists the contents of the given CAS file
/// * `Add(path: PathBuf, files: Vec<PathBuf>)`, adds files to the given CAS file
/// * `Extract(path: PathBuf, item: PathBuf)`, extract the given item from the given CAS file
/// * `Export(path: PathBuf, output: PathBuf)`, export the given CAS file into given output WAV file
///
#[derive(Debug, PartialEq)]
pub enum Command {
    Version,
    List(PathBuf),
    Add(PathBuf, Vec<PathBuf>),
    Extract(PathBuf),
    Export(PathBuf, PathBuf),
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
    flag_add: bool,
    flag_extract: bool,
    flag_export: bool,
    arg_cas_file: String,
    arg_file: Vec<String>,
    arg_wav_file: String,
}

impl Args {

    /// Parse the
    pub fn cmd(self) -> Command {
        if self.flag_version {
            Command::Version
        } else if self.flag_list {
            Command::List(PathBuf::from(self.arg_cas_file))
        } else if self.flag_add {
            Command::Add(
                PathBuf::from(self.arg_cas_file),
                self.arg_file.iter().map(|f| PathBuf::from(f)).collect())
        } else if self.flag_extract {
            Command::Extract(PathBuf::from(self.arg_cas_file))
        } else if self.flag_export {
            Command::Export(PathBuf::from(self.arg_cas_file), PathBuf::from(self.arg_wav_file))
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
pub fn parse_args<I, S>(args: I) -> Command
where S: AsRef<str>, I: Iterator<Item=S>, S: Into<String> {
    let parsed: Args = Docopt::new(USAGE)
        .and_then(|d| d.argv(args).decode())
        .unwrap_or_else(|e| e.exit());
    parsed.cmd()
}

#[cfg(test)]
mod test {

    use std::path::PathBuf;

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
        assert_eq!(Command::List(PathBuf::from("foobar.cas")), cmd);
    }

    #[test]
    fn should_parse_add() {
        let argv = ["mcp", "--add", "foobar.cas", "f1.bin"];
        let cmd = parse_args(argv.iter().map(|a| a.to_string()));
        assert_eq!(Command::Add(PathBuf::from("foobar.cas"), vec![ PathBuf::from("f1.bin")]), cmd);
    }

    #[test]
    fn should_parse_extract() {
        let argv = ["mcp", "--extract", "foobar.cas"];
        let cmd = parse_args(argv.iter().map(|a| a.to_string()));
        assert_eq!(Command::Extract(PathBuf::from("foobar.cas")), cmd);
    }

    #[test]
    fn should_parse_export() {
        let argv = ["mcp", "--export", "foobar.cas", "foobar.wav"];
        let cmd = parse_args(argv.iter().map(|a| a.to_string()));
        assert_eq!(Command::Export(PathBuf::from("foobar.cas"), PathBuf::from("foobar.wav")), cmd);
    }
}
