use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::ops::{Range, Deref};
use iref::{Iri, IriBuf};

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct FileId(usize);

pub struct File {
	source: PathBuf,
	base_iri: Option<IriBuf>,
	buffer: Buffer
}

impl File {
	pub fn source(&self) -> &Path {
		&self.source
	}

	pub fn base_iri(&self) -> Option<Iri> {
		self.base_iri.as_ref().map(IriBuf::as_iri)
	}

	pub fn buffer(&self) -> &Buffer {
		&self.buffer
	}
}

#[derive(Default)]
pub struct Files {
	files: Vec<File>,
	sources: HashMap<PathBuf, FileId>
}

impl Files {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn get(&self, id: FileId) -> Option<&File> {
		self.files.get(id.0)
	}

	// pub fn source(&self, id: FileId) -> Option<&Path> {
	// 	self.get(id).map(File::source)
	// }

	// pub fn content(&self, id: FileId) -> Option<&Buffer> {
	// 	self.get(id).map(File::buffer)
	// }

	pub fn load(&mut self, source: &impl AsRef<Path>, base_iri: Option<IriBuf>) -> std::io::Result<FileId> {
		let source = source.as_ref();
		match self.sources.get(source).cloned() {
			Some(id) => Ok(id),
			None => {
				let content = std::fs::read_to_string(source)?;
				let id = FileId(self.files.len());
				self.files.push(File {
					source: source.into(),
					base_iri,
					buffer: Buffer::new(content)
				});
				self.sources.insert(source.into(), id);
				Ok(id)
			}
		}
	}
}

/// Source file buffer.
///
/// Stores the file content and lines index.
pub struct Buffer {
	/// Buffer data.
	data: String,

	/// Lines index.
	line_starts: Vec<usize>,
}

impl Buffer {
	/// Creates a new buffer from its content.
	#[inline(always)]
	pub fn new(content: String) -> Self {
		let line_starts = codespan_reporting::files::line_starts(&content).collect();

		Self {
			data: content,
			line_starts,
		}
	}

	#[inline(always)]
	fn as_str(&self) -> &str {
		self.data.as_str()
	}

	#[inline(always)]
	pub fn line_count(&self) -> usize {
		self.line_starts.len()
	}

	/// The index of the line at the given byte index.
	///
	/// If the byte index is past the end of the buffer,
	/// returns the maximum line index in the file.
	#[inline(always)]
	pub fn line_index(&self, byte_index: usize) -> usize {
		match self.line_starts.binary_search(&byte_index) {
			Ok(line) => line,
			Err(next_line) => next_line - 1,
		}
	}

	pub fn line_range(&self, line_index: usize) -> Option<Range<usize>> {
		if line_index < self.line_starts.len() {
			let range = if line_index + 1 < self.line_starts.len() {
				self.line_starts[line_index]..self.line_starts[line_index + 1]
			} else {
				self.line_starts[line_index]..self.len()
			};

			Some(range)
		} else {
			None
		}
	}
}

impl Deref for Buffer {
	type Target = str;

	fn deref(&self) -> &str {
		self.as_str()
	}
}

impl AsRef<str> for Buffer {
	fn as_ref(&self) -> &str {
		self.as_str()
	}
}

impl<'a> codespan_reporting::files::Files<'a> for Files {
	type FileId = FileId;
	type Name = std::path::Display<'a>;
	type Source = &'a Buffer;

	fn name(&'a self, id: FileId) -> Result<Self::Name, codespan_reporting::files::Error> {
		Ok(self.get(id).ok_or(codespan_reporting::files::Error::FileMissing)?.source().display())
	}

	fn source(&'a self, id: FileId) -> Result<Self::Source, codespan_reporting::files::Error> {
		Ok(self.get(id).ok_or(codespan_reporting::files::Error::FileMissing)?.buffer())
	}

	fn line_index(
		&'a self,
		id: FileId,
		byte_index: usize,
	) -> Result<usize, codespan_reporting::files::Error> {
		self.get(id)
			.map(|file| file.buffer().line_index(byte_index))
			.ok_or(codespan_reporting::files::Error::FileMissing)
	}

	fn line_range(
		&'a self,
		id: FileId,
		line_index: usize,
	) -> Result<Range<usize>, codespan_reporting::files::Error> {
		self.get(id)
			.map(|file| {
				file.buffer().line_range(line_index).ok_or_else(|| {
					codespan_reporting::files::Error::LineTooLarge {
						given: line_index,
						max: file.buffer().line_count(),
					}
				})
			})
			.transpose()?
			.ok_or(codespan_reporting::files::Error::FileMissing)
	}
}