use std::path::Path;
use std::io::Write;
use std::fmt;

/// Package initialization options.
#[derive(Debug)]
pub struct InitOptions {
	/// Package name.
	pub name: String,

	/// Author.
	pub author: String,

	/// Version.
	pub version: String,

	/// Creates a `README.md` file, unless it already exists.
	pub readme: bool,

	/// Initializes a git repository.
	/// 
	/// If a repository already exists,
	/// creates a `.gitignore` file.
	/// If such file already exists, add the missing entries in it.
	pub git: bool
}

#[derive(Debug)]
pub enum InitError {
	/// I/O  error.
	IO(std::io::Error),

	/// The `git init` command exited with non zero code.
	Git,

	/// Existing `package.json` manifest could not be parsed.
	InvalidManifest(serde_json::Error),

	/// Could not write `package.json` manifest.
	ManifestWrite(serde_json::Error),

	/// Could not write `tsconfig.json` file.
	TSConfigWrite(serde_json::Error)
}

impl From<std::io::Error> for InitError {
	fn from(e: std::io::Error) -> Self {
		Self::IO(e)
	}
}

impl fmt::Display for InitError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::IO(e) => e.fmt(f),
			Self::Git => write!(f, "unable to initialize Git repository"),
			Self::InvalidManifest(e) => write!(f, "invalid `package.json` file: {}", e),
			Self::ManifestWrite(e) => write!(f, "unable to write `package.json` file: {}", e),
			Self::TSConfigWrite(e) => write!(f, "unable to write `tsconfig.json` file: {}", e)
		}
	}
}

const GITIGNORE_ENTRIES: [&'static str; 2] = [
	"node_modules",
	"/lib"
];

macro_rules! json_vec {
	[ $($e:expr),* ] => {
		vec![ $(serde_json::Value::from($e)),* ]
	};
}

pub fn initialize_package(directory: impl AsRef<Path>, options: InitOptions) -> Result<(), InitError> {
	let directory = directory.as_ref();
	
	// Create target directory.
	std::fs::create_dir_all(directory)?;

	// Create `src` directory.
	std::fs::create_dir_all(directory.join("src"))?;
	
	// Create Git repository.
	if options.git {
		// Call `git init`.
		if !directory.join(".git").exists() {
			let output = std::process::Command::new("git")
				.arg("-C")
				.arg(directory)
				.arg("init")
				.output()?;

			if output.status.code() != Some(0) {
				let mut out = std::io::stderr();
				out.write_all(&output.stderr)?;
				return Err(InitError::Git)
			}
		}
	}

	// Create or edit `.gitignore` file.
	let gitignore_path = directory.join(".gitignore");
	let gitignore_content = if gitignore_path.exists() {
		let mut content = std::fs::read_to_string(&gitignore_path)?;
		let mut middle_of_line = content.chars().rev().next().map(|c| c != '\n').unwrap_or(false);
		for entry in GITIGNORE_ENTRIES {
			if content.lines().all(|line| line != entry) {
				if middle_of_line {
					content.push('\n');
					middle_of_line = false
				}

				content.push_str(entry);
				content.push('\n');
			}
		}
		content
	} else {
		let mut content = String::new();
		for entry in GITIGNORE_ENTRIES {
			content.push_str(entry);
			content.push('\n');
		}
		content
	};
	std::fs::write(gitignore_path, gitignore_content)?;

	// Create `README.md` file.
	if options.readme {
		let filename = directory.join("README.md");
		if !filename.exists() {
			std::fs::write(filename, "# TypeScript package\n")?
		}
	}

	// Create `tsconfig.json` file.
	let tsconfig_path = directory.join("tsconfig.json");
	if !tsconfig_path.exists() {
		let mut compiler_options = serde_json::Map::new();
		compiler_options.insert("target".into(), "es5".into());
		compiler_options.insert("module".into(), "commonjs".into());
		compiler_options.insert("declaration".into(), true.into());
		compiler_options.insert("outDir".into(), "./lib".into());
		compiler_options.insert("strict".into(), true.into());

		let mut tsconfig = serde_json::Map::new();
		tsconfig.insert("compilerOptions".into(), compiler_options.into());
		tsconfig.insert("include".into(), json_vec!["src"].into());
		tsconfig.insert("exclude".into(), json_vec!["node_modules", "**/__tests__/*"].into());

		let file = std::fs::File::create(tsconfig_path)?;
		serde_json::to_writer_pretty(file, &tsconfig).map_err(InitError::TSConfigWrite)?;
	}

	// Create `package.json` file.
	let package_path = directory.join("package.json");
	let mut package = if package_path.exists() {
		let content = std::fs::read_to_string(&package_path)?;
		serde_json::from_str(&content).map_err(InitError::InvalidManifest)?
	} else {
		serde_json::Map::new()
	};

	fn try_insert(map: &mut serde_json::Map<String, serde_json::Value>, key: &str, value: impl Into<serde_json::Value>) -> bool {
		if !map.contains_key(key) {
			map.insert(key.to_owned(), value.into());
			true
		} else {
			false
		}
	}

	let mut changed = false;
	changed |= try_insert(&mut package, "name", options.name);
	changed |= try_insert(&mut package, "author", options.author);
	changed |= try_insert(&mut package, "version", options.version);
	changed |= try_insert(&mut package, "main", "main.js");
	changed |= try_insert(&mut package, "build", "tsc");
	changed |= try_insert(&mut package, "files", json_vec!["lib/**/*"]);
	
	match package.get_mut("devDependencies") {
		Some(serde_json::Value::Object(dev_dependencies)) => {
			match dev_dependencies.get("typescript") {
				Some(_version) => {
					// TODO check minimal version.
				},
				None => {
					dev_dependencies.insert("typescript".into(), "^4.6.0".into());
				}
			}
		}
		_ => {
			let mut dev_dependencies = serde_json::Map::new();
			dev_dependencies.insert("typescript".into(), "^4.6.0".into());
			package.insert("devDependencies".into(), dev_dependencies.into());
		}
	}

	if changed {
		let file = std::fs::File::create(package_path)?;
		serde_json::to_writer_pretty(file, &package).map_err(InitError::ManifestWrite)?;
	}

	Ok(())
}