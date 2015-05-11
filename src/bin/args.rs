//
// MSX CAS Packager
// Copyright (c) 2015 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use docopt::Docopt;

static USAGE: &'static str = "
Usage: mcp --list <input-file>
       mcp --help
       mcp --version

Options:
    -h, --help                  Print this message
    -v, --version               Print the mcp version
";

#[derive(RustcDecodable)]
pub struct Args {
    // flag_help: bool,
    flag_version: bool,
    flag_list: bool,
    arg_input_file: String,
}

#[derive(Debug, PartialEq)]
pub enum Action { Version, List(String) }

impl Args {

    pub fn action(&self) -> Action {
        if self.flag_version { Action::Version }
        else if self.flag_list { Action::List(self.arg_input_file.clone()) }
        else { panic!("args are parsed in a inconsistent state") }
    }
}

#[allow(dead_code)]
pub fn parse_args() -> Args {
    Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit())
}

#[cfg(test)]
mod test {

    use super::*;

    use docopt::Docopt;

    fn args_for(tokens: &Vec<&str>) -> Args {
        let argv = tokens.iter().map(|t| t.to_string());
        Docopt::new(super::USAGE)
            .and_then(|d| d.argv(argv.into_iter()).decode())
            .unwrap_or_else(|e| e.exit())
    }

    #[test]
    fn should_parse_version() {
        let args = args_for(&vec!["mcp", "--version"]);
        assert_eq!(Action::Version, args.action());
    }

    #[test]
    fn should_parse_list() {
        let args = args_for(&vec!["mcp", "--list", "foobar.cas"]);
        assert_eq!(Action::List("foobar.cas".to_string()), args.action());
    }
}