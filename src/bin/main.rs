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

#[allow(dead_code)]
fn main() {
    let action = args::parse_args().action();
    match action {
        args::Action::Version => print_version(),
        _ => {},
    };
}

fn print_version() {
    println!("MSX CAS Packager (MCP) v0.1.0");
    println!("Copyright (C) 2015 Alvaro Polo");
    println!("");
    println!("This program is subject to the terms of the Mozilla Public License v2.0.");
    println!("");
}
