# MSX CAS Packager

**M**SX **C**AS **P**ackager is a command line interface (CLI) tool to manage
MSX CAS files. A CAS file is a very common file format used to represent
the contents of a MSX cassete tape.

Using MCP you can list the contents of a CAS file, add new files, extract
them, etc.

## Build from sources

MCP is written in [Rust programming language](http://www.rust-lang.org).
You will need Rust to be installed in the system to build MCP. The simplest
method to do that is to go to the project website and download the latest
distribution installer for your platform.

After installing Rust, just go to the top directory of your MCP's working
copy and type:

	cargo build

MCP will be build in the `target/` directory.
