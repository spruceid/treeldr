use iref::{Iri, IriBuf};
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use std::ops::{Deref, Range};
use std::path::{Path, PathBuf};

pub trait DisplayPath<'a> {
	type Display: 'a + fmt::Display;

	fn display(&'a self) -> Self::Display;
}

impl<'a> DisplayPath<'a> for PathBuf {
	type Display = std::path::Display<'a>;

	fn display(&'a self) -> Self::Display {
		Path::display(self)
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct FileId(usize);

pub type Location = locspan::Location<FileId>;

pub type Metadata = treeldr::Metadata<Location>;

pub struct File<P = PathBuf> {
	source: P,
	base_iri: Option<IriBuf>,
	buffer: Buffer,
	mime_type: Option<MimeType>,
}

impl<P> File<P> {
	pub fn source(&self) -> &P {
		&self.source
	}

	pub fn base_iri(&self) -> Option<Iri> {
		self.base_iri.as_ref().map(IriBuf::as_iri)
	}

	pub fn buffer(&self) -> &Buffer {
		&self.buffer
	}

	pub fn mime_type(&self) -> Option<MimeType> {
		self.mime_type
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MimeType {
	/// application/treeldr
	TreeLdr,

	/// application/schema+json
	JsonSchema,
}

impl MimeType {
	fn name(&self) -> &'static str {
		match self {
			Self::TreeLdr => "application/treeldr",
			Self::JsonSchema => "application/schema+json",
		}
	}

	fn infer(source: &Path, _content: &str) -> Option<MimeType> {
		source
			.extension()
			.and_then(std::ffi::OsStr::to_str)
			.and_then(|ext| match ext {
				"tldr" => Some(MimeType::TreeLdr),
				"json" => Some(MimeType::JsonSchema),
				_ => None,
			})
	}
}

impl fmt::Display for MimeType {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.name().fmt(f)
	}
}

pub struct Files<P = PathBuf> {
	files: Vec<File<P>>,
	sources: HashMap<P, FileId>,
}

impl<P> Default for Files<P> {
	fn default() -> Self {
		Self {
			files: Vec::new(),
			sources: HashMap::new(),
		}
	}
}

impl<P> Files<P> {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn get(&self, id: FileId) -> Option<&File<P>> {
		self.files.get(id.0)
	}

	// pub fn source(&self, id: FileId) -> Option<&Path> {
	// 	self.get(id).map(File::source)
	// }

	// pub fn content(&self, id: FileId) -> Option<&Buffer> {
	// 	self.get(id).map(File::buffer)
	// }

	pub fn load(
		&mut self,
		source: &(impl ?Sized + AsRef<Path>),
		base_iri: Option<IriBuf>,
		mime_type: Option<MimeType>,
	) -> std::io::Result<FileId>
	where
		P: Clone + Eq + Hash + for<'a> From<&'a Path>,
	{
		let path = source.as_ref();
		let source: P = path.into();
		match self.sources.get(&source).cloned() {
			Some(id) => Ok(id),
			None => {
				let content = std::fs::read_to_string(path)?;
				let id = FileId(self.files.len());
				let mime_type = mime_type.or_else(|| MimeType::infer(path, &content));
				self.files.push(File {
					source: source.clone(),
					base_iri,
					buffer: Buffer::new(content),
					mime_type,
				});
				self.sources.insert(source, id);
				Ok(id)
			}
		}
	}

	pub fn load_content(
		&mut self,
		source: P,
		base_iri: Option<IriBuf>,
		mime_type: Option<MimeType>,
		content: String,
	) -> FileId
	where
		P: Clone + Eq + Hash,
	{
		use std::collections::hash_map::Entry;
		match self.sources.entry(source) {
			Entry::Occupied(entry) => *entry.get(),
			Entry::Vacant(entry) => {
				let id = FileId(self.files.len());
				self.files.push(File {
					source: entry.key().clone(),
					base_iri,
					buffer: Buffer::new(content),
					mime_type,
				});
				entry.insert(id);
				id
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
	pub fn as_str(&self) -> &str {
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

impl<'a, P: DisplayPath<'a>> codespan_reporting::files::Files<'a> for Files<P> {
	type FileId = FileId;
	type Name = P::Display;
	type Source = &'a Buffer;

	fn name(&'a self, id: FileId) -> Result<Self::Name, codespan_reporting::files::Error> {
		Ok(self
			.get(id)
			.ok_or(codespan_reporting::files::Error::FileMissing)?
			.source()
			.display())
	}

	fn source(&'a self, id: FileId) -> Result<Self::Source, codespan_reporting::files::Error> {
		Ok(self
			.get(id)
			.ok_or(codespan_reporting::files::Error::FileMissing)?
			.buffer())
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

#[cfg(feature = "json-ld-context")]
impl<P: Send + Sync + Clone + Eq + Hash + for<'a> From<&'a Path>>
	treeldr_json_ld_context::command::Files for Files<P>
{
	type Id = FileId;

	type Metadata = Metadata;

	fn load(&mut self, path: &Path, base_iri: IriBuf) -> Result<(Self::Id, &str), std::io::Error> {
		let id = self.load(path, Some(base_iri), None)?;
		let content = self.get(id).unwrap().buffer().as_str();
		Ok((id, content))
	}

	fn build_metadata(&self, id: Self::Id, span: locspan::Span) -> Self::Metadata {
		Metadata::Extern(Location::new(id, span))
	}
}
