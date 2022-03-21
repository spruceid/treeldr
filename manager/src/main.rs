use clap::Parser;

#[derive(Parser)]
#[clap(name="treeldr", author, version, about, long_about = None)]
struct Args {
	/// Sets the level of verbosity.
	#[clap(short, long = "verbose", parse(from_occurrences))]
	verbosity: usize,

	#[clap(subcommand)]
	command: Command,
}

#[derive(clap::Subcommand)]
pub enum Command {
	/// Compile the current package.
	Build
}

fn main() {
	// Parse options.
	let args = Args::parse();

	// Init logger.
	stderrlog::new().verbosity(args.verbosity).init().unwrap();

	match args.command {
		Command::Build => {
			println!("bye.")
		}
	}
}