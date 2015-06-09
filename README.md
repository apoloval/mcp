[![Build Status](https://travis-ci.org/apoloval/mcp.svg?branch=master)](https://travis-ci.org/apoloval/mcp)

# MSX CAS Packager

**M**SX **C**AS **P**ackager is a command line interface (CLI) tool to manage
MSX CAS files. A CAS file is a very common file format used to represent
the contents of a MSX cassete tape.

Using MCP you can perform the following actions on a CAS file.

* List CAS files. You can obtain a list of files contained in the CAS file.
* Add files to a new or existing CAS file. This is one the most powerful
features of MCP. You can create your own CAS files with your files. Basic
programs, assembler programs, data, etc, can be stored into a CAS file.
* Extract contents from existing CAS file. Did you ever want to make some
reverse engineering of your favorite MSX games? Now that's possible thanks
to MCP! You can extract the files of a CAS file, read their Basic programs,
study its binary code, etc.
* Export CAS files to WAV format. This feature, combined with the ability
to create new CAS files with your own programs, makes possible to convert
the CAS file into WAV that can be played to your MSX hardware cassette
interface. Code your own games and play them in your real MSX!

By the way, MCP is free and open source. You only have to obey the terms of
the [Mozilla Public License](https://www.mozilla.org/MPL/), which is basically:

* Use the software for anything you want, as long as you want, wherever you
want.
* If you want to modify the software, read the terms carefully.

## Download binaries

You will find installation binaries for the latest version of MCP in the
following links.

* [MCP for Mac OS X 64-bits](https://bintray.com/artifact/download/apoloval/msx/mcp/v0.1.0/mcp-0.1.0.pkg)
* [MCP for Windows 32-bits](https://bintray.com/artifact/download/apoloval/msx/mcp/v0.1.0/mcp-0.1.0_x86.exe)
* [MCP for Windows 64-bits](https://bintray.com/artifact/download/apoloval/msx/mcp/v0.1.0/mcp-0.1.0_x64.exe)

If your operating system is not listed above, please try to build from sources
as described below.

## Build from sources

MCP is written in [Rust programming language](http://www.rust-lang.org).
You will need Rust to be installed in the system to build MCP. The simplest
method to do that is to go to the project website and download the latest
distribution installer for your platform.

After installing Rust, just go to the top directory of your MCP's working
copy and type:

    $ cargo build --release

MCP will be build in the `target/release` directory.

## How it works

MCP is a command line utility (CLI), and therefore must be used from a console.
If you don't know how to use a console have a look to the thousand of tutorials
available on the Internet and come back later.

In you have installed from binaries, take into account that.

* In Windows platform, you will find an app launcher named `MCP Console`. This is
just a Windows shell with the apprpriate `%PATH%` definition to find the `mcp`
program. Run this launcher and test it by typing `mcp --help`.
* In Mac OS X, the installer will put `mcp` command in `/usr/local/bin`. In a
typicall installation, this directory should be listed in `PATH` environment
variable, so `mcp` can be executed in your terminal. Please open `Terminal` and
type `mcp --help` to check it works.

If you have installed from sources, you are responsible to put `mcp` program
somewhere in your hard drive and (suggestion) include that directory in your
`PATH` environment variable. If you do so you will be able to execute `mcp`
easily.

As mentioned above, executing `mcp --help` will let you to know if `mcp` is
working fine. But it also presents you the information needed to familiarize
yourself with the command options.

    $ mcp --help
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

Let's have a look to each of the commands to see how they work.

### List package contents

With `mcp -l arkanoid.cas` (or longer version `mcp --list arkanoid.cas`), we
can see the contents of the `arkanoid.cas` file.

    $ mcp -l arkanoid.cas
    ascii  | ark    |   256 bytes |
    bin    | ARK    |    96 bytes | [0xc000,0xc057]:0xc000
    custom |        | 32768 bytes |

As you can see, the contents of the file are shown. In this example we have
three files in the CAS tape. The first column indicates the file type, which
can be one of the following:

* `binary`: a binary file, probably containing a mix of binary code and data.
It is typically loaded with `BLOAD` command from the Basic interpreter.
* `ascii`: an ASCII file, probably containing a non-tokenized Basic program.
It is typically loaded with `LOAD` command from the Basic interpreter.
* `basic`: a Basic file, containing a tokenized Basic program. It is typically
loaded with `CLOAD` command from the Basic interpreter.
* `custom`: custom data, aimed to be loaded by some program in a custom way.

The second column shows the name of the file. The third column shows the length
of the file in bytes. The forth column is only shown for binary files, and it
contains the memory addresses where the binary data will be placed: start
address, end address and begin address.

### Add contents to package

With `mcp -a myprogram.cas myprog.bin`, you can create a new CAS file
`myprogram.cas` that contains the file `myprog.bin`.

    $ mcp -a myprogram.cas myprog.bin
    Adding myprog.bin... Done

You can check the contents of the new CAS file with `mcp -l`.

    $ mcp -l myprogram.cas
    bin    | myprog |   100 bytes | [0x8000,0x803e]:0x8000

The bin filename is intentionally shorten than the CAS file. Tape filenames
are limited to six bytes. If your bin file would be `myprogram.bin` its name
would be truncated.

    $ mcp -a myprogram.cas myprogram.bin
    Adding myprogram.bin... Warning: filename myprogram.bin is too long, truncating
    Done

    $ mcp -l myprogram.cas
    bin    | myprog |   100 bytes | [0x8000,0x803e]:0x8000

MCP is able to determine the file type by the file extension with the following
criteria:

* `file.bin` is interpreted and stored as binary file. The binary file ID byte
(`0xfe`) is automatically removed when dumped into the CAS file. Please remember
that binary files in cassette do not include it.
* `file.asc` is interpreted and stored as ASCII file. Its contents are automatically
padded by MCP with EOF (end-of-file) bytes to have 256-byte aligned blocks required
by MSX systems to load the file successfully.
* `file.bas` is interpreted and stored as Basic file
* Any other file extension is interpreted as and stored as a custom file

It is possible to add new files to an existing CAS file.

    $ mcp -l myprogram.cas
    bin    | myprog |   100 bytes | [0x8000,0x803e]:0x8000

    $ mcp -a myprogram.cas foobar.dat
    Adding foobar.dat... Done

    $ mcp -l myprogram.cas
    bin    | myprog |   100 bytes | [0x8000,0x803e]:0x8000
    custom |        |  6920 bytes |

Nevertheless, you don't have to add files one by one. You can specify several
files and all them will be added to the CAS file.

    $ mcp -l myprogram.cas
    bin    | myprog |   100 bytes | [0x8000,0x803e]:0x8000
    custom |        |  6920 bytes |

    $ mcp -a myprogram.cas foobar2.dat foobar3.dat
    Adding foobar2.dat... Done
    Adding foobar3.dat... Done

    $ mcp -l myprogram.cas
    bin    | myprog |   100 bytes | [0x8000,0x803e]:0x8000
    custom |        |  6920 bytes |
    custom |        | 29648 bytes |
    custom |        | 49272 bytes |


### Extract package contents

Using `mcp -x arkanoid.cas`, you can extract the contents of `arkanoid.cas`
into the working directory.

    $ mcp -x arkanoid.cas
    Extracting ark.asc... Done
    Extracting ARK.bin... Done
    Extracting custom.001... Done

    $ ls
    ARK.bin		ark.asc		arkanoid.cas	custom.001

The files are extracted using the following criteria:

* Binary files are extracted with the original name plus `.bin` extension.
* ASCII files are extracted with the original name plus `.asc` extension.
* Basic files are extracted with the original name plus `.bas` extension.
* Custom files are extracted as `custom.XXX`, where `XXX` is a sequence number
indicating the relative position of the custom file in the tape.

In case of ASCII files, the trailing EOF bytes are not copied to the target
file so you can read the Basic source code as text.

In case of binary files, the leading ID byte (`0xfe`) is automatically
prepended to the target file.

    $ file ark.asc
    ark.asc: ASCII text, with CRLF line terminators

    $ cat ark.asc
    10 BLOAD"cas:",R

### Export package to WAV format

Using `mcp -e myprogram.cas myprogram.wav` you can export the contents of the
tape into a WAV file. The WAV file can be reproduced with your sound card to
load the data into a real MSX hardware using the cassette interface.

    $ mcp -e myprogram.cas myprogram.wav
    Encoding block 0... 371 KiB
    Encoding block 1... 151 KiB
    Encoding block 2... 2788 KiB
    Encoding block 3... 11577 KiB
    Encoding block 4... 19166 KiB

    $ file myprogram.wav
    myprogram.wav: RIFF (little-endian) data, WAVE audio, Microsoft PCM, 8 bit, mono 43200 Hz

The resulting file is ready to be played and make your homebrew programs
loadable in your MSX computer.

## Acknowledgements

MCP was coded by porting several code fragments from
[CAS Tools](http://home.kabelfoon.nl/~vincentd/). Many thanks to Vincent van
Dam for his software.

MCP export feature has been tested with several games in
[OpenMSX](http://openmsx.sourceforge.net) emulator. Many thanks to its authors
and congratulations for making such a great software.
