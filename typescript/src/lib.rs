use std::fmt;
use std::io::Write;
use std::path::Path;
use treeldr::Ref;
pub use treeldr_codegen::{Indent, IndentBy};

pub mod command;
mod init;

pub use command::Command;
pub use init::*;

#[derive(Debug)]
pub enum Error {
	Init(InitError),
	IO(std::io::Error),
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Init(e) => write!(f, "package initialization failed: {}", e),
			Self::IO(e) => e.fmt(f),
		}
	}
}

pub fn generate_package<F>(
	model: &treeldr::Model<F>,
	directory: impl AsRef<Path>,
	init_options: InitOptions,
	gen_options: Options,
) -> Result<(), Error> {
	initialize_package(&directory, init_options).map_err(Error::Init)?;

	let mut main =
		std::fs::File::create(directory.as_ref().join("src/main.ts")).map_err(Error::IO)?;
	write!(main, "{}", ().generated(model, gen_options)).map_err(Error::IO)?;

	Ok(())
}

#[derive(Clone, Copy)]
pub struct Options {
	/// Indentation string.
	indent: Indent,
}

pub trait Generate<F> {
	fn gen(
		&self,
		f: &mut fmt::Formatter,
		model: &treeldr::Model<F>,
		options: Options,
	) -> fmt::Result;

	fn generated<'m>(
		&self,
		model: &'m treeldr::Model<F>,
		options: Options,
	) -> Generated<'m, '_, F, Self> {
		Generated(model, self, options)
	}
}

pub struct Generated<'m, 'a, F, T: ?Sized>(&'m treeldr::Model<F>, &'a T, Options);

impl<'m, 'a, F, T: Generate<F>> fmt::Display for Generated<'m, 'a, F, T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.1.gen(f, self.0, self.2)
	}
}

impl<F> Generate<F> for () {
	fn gen(
		&self,
		f: &mut fmt::Formatter,
		model: &treeldr::Model<F>,
		options: Options,
	) -> fmt::Result {
		let mut first = true;
		for (layout_ref, _) in model.layouts().iter() {
			let layout = model.layouts().get(layout_ref).unwrap();

			if layout.description().is_struct() {
				if !first {
					writeln!(f)?;
				}
				first = false;

				let mut doc_comment = String::new();
				if let Some(label) = layout.preferred_label(model) {
					doc_comment.push_str(label);
				}

				if let Some(doc) = layout.preferred_documentation(model).as_str() {
					if !doc_comment.is_empty() {
						doc_comment.push_str("\n\n");
					}

					doc_comment.push_str(doc);
				}

				if !doc_comment.is_empty() {
					let comment = treeldr_codegen::doc::Comment::new(
						doc_comment,
						treeldr_codegen::doc::CommentSyntax::Single("// "),
						options.indent.by(0),
					);

					writeln!(f, "{}", comment)?;
				}

				layout.gen(f, model, options)?;
			}
		}

		Ok(())
	}
}

impl<F> Generate<F> for treeldr::layout::Definition<F> {
	fn gen(
		&self,
		f: &mut fmt::Formatter,
		model: &treeldr::Model<F>,
		options: Options,
	) -> fmt::Result {
		if let treeldr::layout::Description::Struct(s) = self.description() {
			writeln!(f, "class {} {{", s.name())?;

			for field in s.fields() {
				write!(
					f,
					"{}{}: {}",
					options.indent.by(1),
					field.name(),
					field.annotated_layout().generated(model, options)
				)?;

				if let Some(value) = default_value(field.annotated_layout()) {
					write!(f, " = {}", value.generated(model, options))?;
				}

				writeln!(f, ";")?;
			}

			let required_fields = s.fields().iter().filter(|f| f.is_required());
			if required_fields.clone().next().is_some() {
				write!(f, "\n{}constructor(", options.indent.by(1))?;
				for (i, field) in required_fields.clone().enumerate() {
					if i > 0 {
						write!(f, ", ")?;
					}

					write!(
						f,
						"{}: {}",
						field.name(),
						field.annotated_layout().generated(model, options)
					)?;
				}
				writeln!(f, ") {{")?;
				for field in required_fields {
					writeln!(
						f,
						"{}this.{name} = {name};",
						options.indent.by(2),
						name = field.name()
					)?;
				}
				writeln!(f, "{}}}", options.indent.by(1))?;
			}

			writeln!(f, "}}")?;
		}

		Ok(())
	}
}

pub enum Value<F> {
	Null,
	EmptyArray(Ref<treeldr::layout::Definition<F>>),
}

impl<F> Generate<F> for Value<F> {
	fn gen(
		&self,
		f: &mut fmt::Formatter,
		_model: &treeldr::Model<F>,
		_options: Options,
	) -> fmt::Result {
		match self {
			Self::Null => write!(f, "null"),
			Self::EmptyArray(_) => {
				write!(f, "[]")
			}
		}
	}
}

fn default_value<F>(layout: &treeldr::layout::AnnotatedRef<F>) -> Option<Value<F>> {
	if layout.is_required() {
		None
	} else if layout.is_functional() {
		Some(Value::Null)
	} else {
		Some(Value::EmptyArray(layout.layout()))
	}
}

impl<F> Generate<F> for treeldr::layout::AnnotatedRef<F> {
	fn gen(
		&self,
		f: &mut fmt::Formatter,
		model: &treeldr::Model<F>,
		options: Options,
	) -> fmt::Result {
		if self.is_functional() {
			if self.is_required() {
				self.layout().gen(f, model, options)
			} else {
				write!(f, "{} | null", self.layout().generated(model, options))
			}
		} else {
			write!(f, "{}[]", self.layout().generated(model, options))
		}
	}
}

impl<F> Generate<F> for Ref<treeldr::layout::Definition<F>> {
	fn gen(
		&self,
		f: &mut fmt::Formatter,
		model: &treeldr::Model<F>,
		options: Options,
	) -> fmt::Result {
		let layout = model.layouts().get(*self).unwrap();

		use treeldr::layout::Description;
		match layout.description() {
			Description::Struct(s) => fmt::Display::fmt(s.name(), f),
			Description::Reference(target, _) => target.gen(f, model, options),
			Description::Native(n, _) => {
				use treeldr::layout::Native;
				match n {
					Native::Boolean => write!(f, "boolean"),
					Native::Integer => write!(f, "number"),
					Native::PositiveInteger => write!(f, "number"),
					Native::Float => write!(f, "number"),
					Native::Double => write!(f, "number"),
					Native::String => write!(f, "string"),
					Native::Date => write!(f, "string"),
					Native::Time => write!(f, "string"),
					Native::DateTime => write!(f, "string"),
					Native::Iri => write!(f, "string"),
					Native::Uri => write!(f, "string"),
					Native::Url => write!(f, "string"),
				}
			}
		}
	}
}
