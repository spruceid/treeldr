use crate::syntax::Span;
use iref::IriBuf;
use std::{
	collections::{HashMap, BTreeSet},
	fmt,
	ops::{Deref, DerefMut, Range},
};

/// Source file identifier.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Id(usize);

/// Position in a source file.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Source {
	/// Source file identifier.
	file: Id,

	/// Span position in the source file.
	span: Span,
}

impl Source {
	/// Creates a new source location.
	pub fn new(file: Id, span: Span) -> Self {
		Self { file, span }
	}

	/// Source file.
	pub fn file(&self) -> Id {
		self.file
	}

	/// Span position in the source file.
	pub fn span(&self) -> Span {
		self.span
	}
}

/// Source file path.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Path {
	/// Local source path.
	Local(std::path::PathBuf),

	/// Foreign source path.
	Foreign(IriBuf),
}

impl fmt::Display for Path {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Local(filename) => filename.display().fmt(f),
			Self::Foreign(iri) => iri.fmt(f),
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
	fn as_mut_str(&mut self) -> &mut str {
		self.data.as_mut_str()
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

impl DerefMut for Buffer {
	fn deref_mut(&mut self) -> &mut str {
		self.as_mut_str()
	}
}

impl AsRef<str> for Buffer {
	fn as_ref(&self) -> &str {
		self.as_str()
	}
}

impl AsMut<str> for Buffer {
	fn as_mut(&mut self) -> &mut str {
		self.as_mut_str()
	}
}

/// Source file.
pub struct File {
	path: Path,
	buffer: Buffer,
}

impl File {
	/// Creates a new source file from its path and content buffer.
	fn new(path: Path, buffer: Buffer) -> Self {
		Self { path, buffer }
	}

	/// Path of the file.
	pub fn path(&self) -> &Path {
		&self.path
	}

	/// Content buffer of the file.
	pub fn buffer(&self) -> &Buffer {
		&self.buffer
	}
}

/// Source files database.
#[derive(Default)]
pub struct Files {
	/// The list of cached files.
	files: Vec<File>,

	/// Maps each cached file path to its id.
	map: HashMap<Path, Id>,
}

impl Files {
	pub fn new() -> Files {
		Self::default()
	}

	/// Adds a file in the database.
	///
	/// Returns the id of the file and a reference to it.
	/// If the file is already cached, changes nothing and returns its id and reference.
	pub fn add(&mut self, path: Path, content: String) -> (Id, &File) {
		use std::collections::hash_map::Entry;
		let id = match self.map.entry(path) {
			Entry::Occupied(entry) => {
				let id = *entry.get();
				id
			}
			Entry::Vacant(entry) => {
				let id = Id(self.files.len());
				self.files
					.push(File::new(entry.key().clone(), Buffer::new(content)));
				id
			}
		};

		(id, &self.files[id.0])
	}

	/// Gets the file associated to the given id, if any.
	pub fn get(&self, id: Id) -> Option<&File> {
		self.files.get(id.0)
	}
}

impl<'a> codespan_reporting::files::Files<'a> for Files {
	type FileId = Id;
	type Name = &'a Path;
	type Source = &'a Buffer;

	fn name(&'a self, id: Id) -> Result<&'a Path, codespan_reporting::files::Error> {
		self.get(id)
			.map(File::path)
			.ok_or(codespan_reporting::files::Error::FileMissing)
	}

	fn source(&'a self, id: Id) -> Result<&'a Buffer, codespan_reporting::files::Error> {
		self.get(id)
			.map(File::buffer)
			.ok_or(codespan_reporting::files::Error::FileMissing)
	}

	fn line_index(
		&'a self,
		id: Id,
		byte_index: usize,
	) -> Result<usize, codespan_reporting::files::Error> {
		self.get(id)
			.map(|file| file.buffer().line_index(byte_index))
			.ok_or(codespan_reporting::files::Error::FileMissing)
	}

	fn line_range(
		&'a self,
		id: Id,
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

/// Cause.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Cause {
	/// Explicitly caused by the given source.
	Explicit(Source),

	/// Implicitly caused by the given source.
	Implicit(Source)
}

impl Cause {
	pub fn is_explicit(&self) -> bool {
		matches!(self, Self::Explicit(_))
	}

	pub fn is_implicit(&self) -> bool {
		matches!(self, Self::Implicit(_))
	}

	pub fn source(&self) -> Source {
		match self {
			Self::Explicit(s) => *s,
			Self::Implicit(s) => *s
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Caused<T> {
	t: T,
	cause: Option<Cause>
}

impl<T> Caused<T> {
	pub fn new(t: T, cause: Option<Cause>) -> Self {
		Self {
			t, cause
		}
	}

	pub fn inner(&self) -> &T {
		&self.t
	}

	pub fn cause(&self) -> Option<Cause> {
		self.cause
	}
}

#[derive(Default)]
pub struct Causes {
	set: BTreeSet<Cause>
}

impl Causes {
	pub fn new() -> Self {
		Self::default()
	}

	/// Adds a new cause.
	pub fn add(&mut self, cause: Cause) {
		self.set.insert(cause);
	}

	/// Picks the preferred cause, unless there are no causes.
	pub fn preferred(&self) -> Option<Cause> {
		self.set.iter().next().cloned()
	}
}

impl From<Cause> for Causes {
	fn from(cause: Cause) -> Self {
		let mut causes = Self::new();
		causes.add(cause);
		causes
	}
}

impl From<Option<Cause>> for Causes {
	fn from(cause: Option<Cause>) -> Self {
		let mut causes = Self::new();
		if let Some(cause) = cause {
			causes.add(cause);
		}
		causes
	}
}