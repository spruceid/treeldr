use iref::IriRefBuf;

#[derive(clap::Args)]
/// Generate a JSON Schema from a TreeLDR model.
pub struct Command {
	/// Layout schema to generate.
	layout: IriRefBuf
}

impl Command {
	pub fn execute(self, model: &treeldr::Model) {
		log::info!("generating JSON Schema.");
		match crate::generate(model, self.layout.as_iri_ref()) {
			Ok(()) => (),
			Err(e) => {
				log::error!("{}", e);
				std::process::exit(1)
			}
		}
	}
}