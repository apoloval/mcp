//
// MSX CAS Packager
// Copyright (c) 2015 Alvaro Polo
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::f32;
use std::convert::From;
use std::io::Write;
use std::io::Error as IoError;
use std::iter::FromIterator;
use std::result;

use byteorder::{LittleEndian, WriteBytesExt};

const SHORT_PULSE: u32 = 2400;
const LONG_PULSE: u32 = 1200;

const SHORT_HEADER: u32 = 4000;
const LONG_HEADER: u32 = 16000;

pub enum Error {
	Io(IoError)
}

impl From<IoError> for Error {
	fn from(e: IoError) -> Error { Error::Io(e) }
}

type Result<T> = result::Result<T, Error>;

/// An object capable to export binary data in WAV format
///
/// The exporter object works by encoding silences, headers and data into
/// an internal buffer. When all the necessary data is encoded, you may use
/// the `export()` method to generate the corresponding WAV header and dump
/// the content into a valid WAV file.
pub struct Exporter {
	bauds: u32,
	sample_rate: u32,
	buffer: Vec<u8>,
}

impl Exporter {

	/// Create a new exporter using default settings
	///
	/// Default settins are 1200 bauds and 43200 samples per second.
	pub fn new() -> Exporter {
		Exporter {
			bauds: 1200,
			sample_rate: 43200,
			buffer: Vec::new(),
		}
	}

	/// Export the encoded data to the given `Write` instance
	///
	/// This method dumps the encoded data into the given `Write` instance. Before
	/// calling this method, you must use the `write_X()` functions to encode
	/// some data.
	pub fn export<W: Write>(&self, w: &mut W) -> Result<()> {
		try!(self.write_wave(w));
		try!(w.write(&*self.buffer));
		Ok(())
	}

	/// Write a short header to the internal buffer
	pub fn write_short_header(&mut self) -> Result<usize> {
		self.write_header(SHORT_HEADER)
	}

	/// Write a long header to the internal buffer
	pub fn write_long_header(&mut self) -> Result<usize> {
		self.write_header(LONG_HEADER)
	}

	/// Write a header comprised by the given amount of pulses to the internal buffer
	pub fn write_header(&mut self, pulses: u32) -> Result<usize> {
		let to = pulses * self.bauds / 1200;
		let mut nbytes = 0;
		for _ in 0..to {
			nbytes += try!(self.write_pulse(SHORT_PULSE));
		}
		Ok(nbytes)
	}

	/// Write a short silence (1 second) to the internal buffer
	pub fn write_short_silence(&mut self) -> Result<usize> {
		let pulses = self.sample_rate;
		self.write_silence(pulses)
	}

	/// Write a long silence (2 seconds) to the internal buffer
	pub fn write_long_silence(&mut self) -> Result<usize> {
		let pulses = self.sample_rate * 2;
		self.write_silence(pulses)
	}

	/// Write a silence comprised by the given amount of pulses to the internal buffer
	pub fn write_silence(&mut self, pulses: u32) ->  Result<usize> {
		let mut nbytes = 0;
		for _ in 0..pulses {
			nbytes += try!(self.buffer.write(&[0x80]).map_err(Error::from));
		}
		Ok(nbytes)
	}

	/// Write binary data to the internal buffer
	pub fn write_data(&mut self, data: &[u8]) -> Result<usize> {
		let mut nbytes = 0;
		for byte in data {
			nbytes += try!(self.write_byte(*byte));
		}
		Ok(nbytes)
	}

	fn write_wave<W: Write>(&self, w: &mut W) -> Result<()> {
		let data_len = self.buffer.len() as u32;
		let file_len = data_len + 44;

		// RIFF chunk start
		try!(write!(w, "RIFF"));

		// RIFF chunk length (size of overall file)
		try!(w.write_u32::<LittleEndian>(file_len));

		// WAVE chunk start
		try!(write!(w, "WAVE"));

		// Format chunk start
		try!(write!(w, "fmt "));

		// Format chunk length
		try!(w.write_u32::<LittleEndian>(16));

		// Type of format (PCM)
		try!(w.write_u16::<LittleEndian>(1));

		// Number of channels
		try!(w.write_u16::<LittleEndian>(1));

		// Sample rate
		try!(w.write_u32::<LittleEndian>(self.sample_rate));

		// Sample rate * bits per sample * channels / 8
		try!(w.write_u32::<LittleEndian>(self.sample_rate));

		// Bits per sample * channels
		try!(w.write_u16::<LittleEndian>(8));

		// Bits per sample
		try!(w.write_u16::<LittleEndian>(8));

		// Data chunk start
		try!(write!(w, "data"));

		// Data chunk length
		try!(w.write_u32::<LittleEndian>(data_len));

		Ok(())
	}

	fn write_byte(&mut self, byte: u8) -> Result<usize> {
		let mut nbytes = 0;
		nbytes += try!(self.write_pulse(LONG_PULSE));
		let mut bits = byte;
		for _ in 0..8 {
			if bits & 0x01 > 0 {
				nbytes += try!(self.write_pulse(SHORT_PULSE));
				nbytes += try!(self.write_pulse(SHORT_PULSE));
			} else {
				nbytes += try!(self.write_pulse(LONG_PULSE));
			}
			bits = bits >> 1;
		}
		for _ in 0..4 {
			nbytes += try!(self.write_pulse(SHORT_PULSE));
		}
		Ok(nbytes)
	}

	fn write_pulse(&mut self, freq: u32) -> Result<usize> {
		let len = self.sample_rate / (self.bauds * (freq / 1200));
		let scale = 2.0 * f32::consts::PI  / len as f32;
		let func = |x: f32| (f32::sin(scale * x) * 127.0) as i8 as u8 ^ 0x80;
		let bytes = Vec::from_iter((0..len).map(|x| func(x as f32)));
		self.buffer.write(&bytes[..]).map_err(Error::from)
	}
}

#[cfg(test)]
mod test {

	use byteorder::{ByteOrder, LittleEndian};

	use super::*;

	#[test]
	fn should_export_empty_data() {
		let exporter = Exporter::new();
		let mut output: Vec<u8> = Vec::new();
		exporter.export(&mut output).ok();
		assert_eq!("RIFF".as_bytes(), &output[0..4]);
		assert_eq!(44, LittleEndian::read_u32(&output[4..8]));
		assert_eq!("WAVE".as_bytes(), &output[8..12]);
		assert_eq!("fmt ".as_bytes(), &output[12..16]);
		assert_eq!(16, LittleEndian::read_u32(&output[16..20]));
		assert_eq!(1, LittleEndian::read_u16(&output[20..22]));
		assert_eq!(1, LittleEndian::read_u16(&output[22..24]));
		assert_eq!(43200, LittleEndian::read_u32(&output[24..28]));
		assert_eq!(43200, LittleEndian::read_u32(&output[28..32]));
		assert_eq!(8, LittleEndian::read_u16(&output[32..34]));
		assert_eq!(8, LittleEndian::read_u16(&output[34..36]));
		assert_eq!("data".as_bytes(), &output[36..40]);
		assert_eq!(0, LittleEndian::read_u32(&output[40..44]));
	}
}
