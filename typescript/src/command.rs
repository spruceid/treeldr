use std::path::PathBuf;
pub use crate::Error;

#[derive(clap::Args)]
/// Generate a TypeScript file.
pub struct Command {
	#[clap(short = 's')]
	/// Use the given number of spaces for indentation.
	indent_spaces: Option<u8>
}

#[derive(clap::Args)]
/// Generate a TypeScript package.
pub struct Package {
	/// Package name.
	name: String,

	#[clap(short = 'a', long = "author")]
	/// Package author.
	author: Option<String>,

	#[clap(short = 'v', long = "version")]
	/// Package version.
	version: Option<String>,

	#[clap(short = 'm', long = "readme")]
	/// Creates a `README.md` file, unless it already exists.
	readme: bool,

	#[clap(short = 'g', long = "git")]
	/// Initializes a git repository.
	/// 
	/// If a repository already exists,
	/// creates a `.gitignore` file.
	/// If such file already exists, add the missing entries in it.
	git: bool,

	#[clap(short = 'd', long = "dir")]
	/// Defines where to generate the package.
	/// 
	/// The default is the current working directory.
	directory: Option<PathBuf>,

	#[clap(short = 's')]
	/// Use the given number of spaces for indentation.
	indent_spaces: Option<u8>
}

impl Command {
	pub fn execute<F: Clone>(self, model: &treeldr::Model<F>) {
		match self.try_execute(model) {
			Ok(()) => (),
			Err(e) => {
				log::error!("{}", e);
				std::process::exit(1)
			}
		}
	}

	fn try_execute<F: Clone>(self, model: &treeldr::Model<F>) -> Result<(), Error> {
		use crate::Generate;

		let options = crate::Options {
			indent: match self.indent_spaces {
				Some(s) => crate::Indent::Spaces(s),
				None => crate::Indent::Tab
			}
		};

		print!("{}", ().generated(model, options));
		Ok(())
	}
}

impl Package {
	pub fn execute<F: Clone>(self, model: &treeldr::Model<F>) {
		match self.try_execute(model) {
			Ok(()) => (),
			Err(e) => {
				log::error!("{}", e);
				std::process::exit(1)
			}
		}
	}

	fn try_execute<F: Clone>(self, model: &treeldr::Model<F>) -> Result<(), Error> {
		let init_options = crate::InitOptions {
			name: self.name,
			author: self.author.unwrap_or_default(),
			version: self.version.unwrap_or_else(|| "0.1.0".to_string()),
			readme: self.readme,
			git: self.git
		};

		let gen_options = crate::Options {
			indent: match self.indent_spaces {
				Some(s) => crate::Indent::Spaces(s),
				None => crate::Indent::Tab
			}
		};

		let directory = self.directory.unwrap_or_default();

		crate::generate_package(model, directory, init_options, gen_options)
	}
}