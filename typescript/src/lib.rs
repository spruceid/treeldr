use std::path::Path;
use std::fmt;
use std::io::Write;
use treeldr::Ref;

mod init;
pub mod command;

pub use init::*;
pub use command::Command;

#[derive(Debug)]
pub enum Error {
	Init(InitError),
	IO(std::io::Error)
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Init(e) => write!(f, "package initialization failed: {}", e),
			Self::IO(e) => e.fmt(f)
		}
	}
}

pub fn generate_package<F>(
	model: &treeldr::Model<F>,
	directory: impl AsRef<Path>,
	init_options: InitOptions,
) -> Result<(), Error> {
	initialize_package(&directory, init_options).map_err(Error::Init)?;

	let mut main = std::fs::File::create(directory.as_ref().join("src/main.ts")).map_err(Error::IO)?;
	write!(main, "{}", ().generated(model)).map_err(Error::IO)?;

	Ok(())
}

pub trait Generate<F> {
	fn gen(&self, f: &mut fmt::Formatter, model: &treeldr::Model<F>) -> fmt::Result;

	fn generated<'m>(&self, model: &'m treeldr::Model<F>) -> Generated<'m, '_, F, Self> {
		Generated(model, self)
	}
}

pub struct Generated<'m, 'a, F, T: ?Sized>(&'m treeldr::Model<F>, &'a T);

impl<'m, 'a, F, T: Generate<F>> fmt::Display for Generated<'m, 'a, F, T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.1.gen(f, self.0)
	}
}

impl<F> Generate<F> for () {
	fn gen(&self, f: &mut fmt::Formatter, model: &treeldr::Model<F>) -> fmt::Result {
		for (layout_ref, _) in model.layouts().iter() {
			let layout = model.layouts().get(layout_ref).unwrap();
			layout.gen(f, model)?;
		}

		Ok(())
	}
}

impl<F> Generate<F> for treeldr::layout::Definition<F> {
	fn gen(&self, f: &mut fmt::Formatter, model: &treeldr::Model<F>) -> fmt::Result {
		if let treeldr::layout::Description::Struct(s) = self.description() {
			writeln!(f, "class {} {{", s.name())?;

			for field in s.fields() {
				let field_layout_ref = field.layout();
				writeln!(f, "\t{}: {}", field.name(), field_layout_ref.generated(model))?;
			}

			writeln!(f, "}}")?;
		}

		Ok(())
	}
}

impl<F> Generate<F> for Ref<treeldr::layout::Definition<F>> {
	fn gen(&self, f: &mut fmt::Formatter, model: &treeldr::Model<F>) -> fmt::Result {
		let layout = model.layouts().get(*self).unwrap();
	
		use treeldr::layout::Description;
		match layout.description() {
			Description::Struct(s) => {
				fmt::Display::fmt(s.name(), f)
			}
			Description::Reference(target, _) => {
				target.gen(f, model)
			}
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
					Native::Url => write!(f, "string")
				}
			}
		}
	}
}