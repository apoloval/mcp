//
// MSX CAS Packager
// Copyright (c) 2015-2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::fs;
use std::io;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use tape;

pub fn exists(file: &Path) -> bool {
    if let Ok(_) = fs::File::open(file) { true } else { false }
}

pub fn remove(file: &Path) -> io::Result<()> {
    fs::remove_file(file)
}

pub fn temporary(file: &Path) -> io::Result<PathBuf> {
    file.file_name()
        .and_then(|fname| fname.to_str())
        .map(|fname| file.with_file_name(format!("{}.temp", fname)))
        .ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("no temporary available for path {:?}", file)))
}

pub fn read_content(file: &Path) -> io::Result<Vec<u8>> {
    let mut data: Vec<u8> = Vec::with_capacity(64*1024);
    fs::File::open(file)
        .map(|mut f| f.read_to_end(&mut data))
        .map(|_| data)
}

pub fn write_content(path: &Path, content: &[u8]) -> io::Result<()> {
    let temp_path = try!(temporary(path));
    let mut file = try!(fs::File::create(&temp_path));
    try!(file.write_all(content));
    if exists(path) {
        try!(remove(path));
    }
    try!(fs::rename(&temp_path, path));
    Ok(())
}

pub fn file_name_of(path: &Path) -> io::Result<([u8;6], bool)> {
    let path_str = try!(path
        .file_stem()
        .and_then(|f| f.to_str())
        .ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("cannot convert path {:?} into string", path))));
    Ok(tape::file_name(path_str))
}

pub fn is_bin_file(path: &Path) -> bool {
    has_extension(path, "bin")
}

pub fn is_ascii_file(path: &Path) -> bool {
    has_extension(path, "asc")
}

pub fn is_basic_file(path: &Path) -> bool {
    has_extension(path, "bas")
}

fn has_extension(path: &Path, ext: &str) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase() == ext).unwrap_or(false)
}

#[cfg(test)]
mod tests {

    use std::path::{Path, PathBuf};

    use tempdir::TempDir;

    use super::*;

    #[test]
    fn should_write_and_read_content() {
        with_unexisting_file("write_and_read", |f| {
            let data = b"Hello World!";
            write_content(f, data).unwrap();
            assert_eq!(read_content(f).unwrap(), data);
        });
    }

    #[test]
    fn should_compute_temporary_file_name() {
        assert_eq!(
            temporary(Path::new("foobar")).unwrap(),
            PathBuf::from("foobar.temp"));
        assert_eq!(
            temporary(Path::new("foobar.cas")).unwrap(),
            PathBuf::from("foobar.cas.temp"));
        assert_eq!(
            temporary(Path::new("/path/to/foobar.cas")).unwrap(),
            PathBuf::from("/path/to/foobar.cas.temp"));
    }

    #[test]
    fn should_compute_file_name_of() {
        let (fname, truncated) = file_name_of(Path::new("foo")).unwrap();
        assert!(!truncated);
        assert_eq!(&fname, b"foo   ");
        let (fname, truncated) = file_name_of(Path::new("foobar")).unwrap();
        assert!(!truncated);
        assert_eq!(&fname, b"foobar");
        let (fname, truncated) = file_name_of(Path::new("guybrush")).unwrap();
        assert!(truncated);
        assert_eq!(&fname, b"guybru");
        let (fname, truncated) = file_name_of(Path::new("/path/to/foobar")).unwrap();
        assert!(!truncated);
        assert_eq!(&fname, b"foobar");
    }

    #[test]
    fn should_compute_is_bin_file() {
        assert!(is_bin_file(Path::new("foobar.bin")));
        assert!(is_bin_file(Path::new("foobar.BIN")));
        assert!(is_bin_file(Path::new("foobar.BiN")));
        assert!(!is_bin_file(Path::new("foobar")));
        assert!(!is_bin_file(Path::new("foobar.bina")));
    }

    #[test]
    fn should_compute_is_ascii_file() {
        assert!(is_ascii_file(Path::new("foobar.asc")));
        assert!(is_ascii_file(Path::new("foobar.ASC")));
        assert!(is_ascii_file(Path::new("foobar.AsC")));
        assert!(!is_ascii_file(Path::new("foobar")));
        assert!(!is_ascii_file(Path::new("foobar.asci")));
    }

    #[test]
    fn should_compute_is_basic_file() {
        assert!(is_basic_file(Path::new("foobar.bas")));
        assert!(is_basic_file(Path::new("foobar.BAS")));
        assert!(is_basic_file(Path::new("foobar.BaS")));
        assert!(!is_basic_file(Path::new("foobar")));
        assert!(!is_basic_file(Path::new("foobar.basi")));
    }

    fn with_unexisting_file<F>(filename: &str, f: F) where F: FnOnce(&Path) {
        let temp = TempDir::new("mcp").unwrap();
        let mut path_buf = temp.path().to_path_buf();
        path_buf.push(filename);
        f(&path_buf);
    }
}
