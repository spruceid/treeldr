use std::{fmt, ops::Deref};

use iref::IriBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
#[repr(transparent)]
pub struct Nsid(str);

impl Nsid {
	pub fn new(s: &str) -> Result<&Self, InvalidNsid> {
		if check(s.bytes()) {
			Ok(unsafe { std::mem::transmute(s) })
		} else {
			Err(InvalidNsid(s.to_string()))
		}
	}

	/// Creates a new NSID from the given string.
	///
	/// # Safety
	///
	/// The input string must be a NSID.
	pub unsafe fn new_unchecked(s: &str) -> &Self {
		std::mem::transmute(s)
	}

	pub fn as_str(&self) -> &str {
		&self.0
	}

	pub fn as_iri(&self) -> IriBuf {
		let mut iri = IriBuf::from_scheme("lexicon".try_into().unwrap());

		for segment in self.0.split('.') {
			iri.path_mut().push(segment.try_into().unwrap())
		}

		iri
	}
}

#[derive(Debug, Serialize)]
pub struct NsidBuf(String);

#[derive(Debug, thiserror::Error)]
#[error("invalid NSID `{0}`")]
pub struct InvalidNsid(pub String);

impl NsidBuf {
	pub fn new(s: String) -> Result<Self, InvalidNsid> {
		if check(s.bytes()) {
			Ok(Self(s))
		} else {
			Err(InvalidNsid(s))
		}
	}

	pub fn into_string(self) -> String {
		self.0
	}
}

impl Deref for NsidBuf {
	type Target = Nsid;

	fn deref(&self) -> &Self::Target {
		unsafe { Nsid::new_unchecked(&self.0) }
	}
}

impl fmt::Display for NsidBuf {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.0.fmt(f)
	}
}

fn check(mut bytes: impl Iterator<Item = u8>) -> bool {
	enum State {
		SegmentAlpha(usize),
		Segment(usize),
	}

	let mut state = State::SegmentAlpha(0);

	loop {
		state = match state {
			State::SegmentAlpha(n) => match bytes.next() {
				Some(b'a'..=b'z' | b'A'..=b'Z') => State::Segment(n),
				_ => break false,
			},
			State::Segment(n) => match bytes.next() {
				Some(b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-') => State::Segment(n),
				Some(b'.') => State::SegmentAlpha(n + 1),
				None if n >= 2 => break true,
				_ => break false,
			},
		}
	}
}

impl<'de> Deserialize<'de> for NsidBuf {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		struct Visitor;

		impl<'de> serde::de::Visitor<'de> for Visitor {
			type Value = NsidBuf;

			fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
				write!(f, "a NSID")
			}

			fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
			where
				E: serde::de::Error,
			{
				self.visit_string(v.to_string())
			}

			fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
			where
				E: serde::de::Error,
			{
				NsidBuf::new(v).map_err(|InvalidNsid(value)| {
					E::invalid_value(serde::de::Unexpected::Str(&value), &self)
				})
			}
		}

		deserializer.deserialize_string(Visitor)
	}
}
