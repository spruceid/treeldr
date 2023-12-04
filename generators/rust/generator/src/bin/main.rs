use iref::{iri::PathBuf, IriBuf};

#[derive(clap::Parser)]
#[clap(name="treeldr", author, version, about, long_about = None)]
struct Args {
	/// Input files.
	filenames: Vec<PathBuf>,

	/// Layout to generate.
	#[clap(long, short)]
	layout: Option<IriBuf>,

	/// Sets the level of verbosity.
	#[clap(short, long = "verbose", action = clap::ArgAction::Count)]
	verbosity: u8,
}

fn main() {
	// Parse options.
	let args: Args = clap::Parser::parse();

	// Initialize logger.
	stderrlog::new()
		.verbosity(args.verbosity as usize)
		.init()
		.unwrap();

	for filename in args.filenames {
		todo!()
	}

	todo!()
}
