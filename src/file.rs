//
// MSX CAS Packager
// Copyright (c) 2015-2016 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::ffi::OsStr;
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
    let temp_path = temporary(path)?;
    let mut file = fs::File::create(&temp_path)?;
    file.write_all(content)?;
    if exists(path) {
        remove(path)?;
    }
    fs::rename(&temp_path, path)?;
    Ok(())
}

pub fn file_name_of(path: &Path) -> io::Result<([u8;6], bool)> {
    let path_str = path
        .file_stem()
        .and_then(|f| f.to_str())
        .ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("cannot convert path {:?} into string", path)))?;
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

pub fn unique_filename(path: &Path) -> io::Result<(PathBuf, bool)> {
    if !exists(path) {
        Ok((path.to_path_buf(), false))
    } else {
        unique_filename_for_suffix(path, 1).map(|f| (f, true))
    }
}

fn unique_filename_for_suffix(path: &Path, suffix: usize) -> io::Result<PathBuf> {
    let stem = extract_from_path(path, |p| p.file_stem())?;
    let ext = path.extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{}", e))
        .unwrap_or("".to_string());
    let target = path.with_file_name(format!("{}-{}{}", stem, suffix, ext));
    if !exists(&target) {
        Ok(target)
    } else {
        unique_filename_for_suffix(path, suffix + 1)
    }
}

fn extract_from_path<F>(path: &Path, f: F) -> io::Result<&str>
where F: FnOnce(&Path) -> Option<&OsStr> {
    f(path).and_then(|s| s.to_str()).ok_or_else(|| io::Error::new(
        io::ErrorKind::InvalidInput,
        format!("cannot extract path element from {:?}", path)))
}

#[cfg(test)]
mod tests {

    use std::ffi::OsStr;
    use std::fs::File;
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


    #[test]
    fn should_compute_unique_filename() {
        with_unexisting_file("foobar", |f| {
            let (alt, clash) = unique_filename(f).unwrap();
            assert!(!clash);
            assert_eq!(alt, f);
        });
        with_existing_file("foobar", |f| {
            let (alt, clash) = unique_filename(f).unwrap();
            assert!(clash);
            assert_eq!(alt, f.with_file_name("foobar-1"));
        });
        with_existing_file("foobar.bin", |f| {
            let (alt, clash) = unique_filename(f).unwrap();
            assert!(clash);
            assert_eq!(alt, f.with_file_name("foobar-1.bin"));
        });
        with_existing_file("foobar.bin", |f1| {
            with_existing_file_from(f1, "foobar-1.bin", |_| {
                let (alt, clash) = unique_filename(f1).unwrap();
                assert!(clash);
                assert_eq!(alt, f1.with_file_name("foobar-2.bin"));
            })
        });
    }

    fn with_unexisting_file<P, F>(filename: P, f: F) where P: AsRef<Path>, F: FnOnce(&Path) {
        let temp = TempDir::new("mcp").unwrap();
        let mut path_buf = temp.path().to_path_buf();
        path_buf.push(filename);
        f(&path_buf);
    }

    fn with_existing_file<P, F>(filename: P, f: F) where P: AsRef<Path>, F: FnOnce(&Path) {
        with_unexisting_file(filename, |file| {
            { File::create(file).unwrap(); }
            f(file);
        })
    }

    fn with_existing_file_from<P1, P2, F>(filename: P1, new_name: P2, f: F)
    where P1: AsRef<Path>, P2: AsRef<OsStr>, F: FnOnce(&Path) {
        let file: &Path = filename.as_ref();
        let other = file.with_file_name(new_name);
        with_existing_file(other, f)
    }
}
