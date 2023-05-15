use codespan_reporting::term::{
	self,
	termcolor::{ColorChoice, StandardStream},
};
use contextual::WithContext;
use proc_macro::TokenStream;
use proc_macro_error::{abort, abort_call_site, proc_macro_error};
use std::{collections::HashMap, path::PathBuf};
use treeldr_load as load;

mod module;
use module::Module;

#[proc_macro_attribute]
#[proc_macro_error]
pub fn tldr(attr: TokenStream, item: TokenStream) -> TokenStream {
	match module::Inputs::from_stream(attr.into()) {
		Ok(inputs) => {
			let options = treeldr_rust_gen::Options {
				impl_rdf: !inputs.no_rdf(),
			};

			let item = syn::parse_macro_input!(item as syn::Item);
			match Module::from_item(item) {
				Ok(mut module) => {
					let mut files = load::Files::<PathBuf>::new();
					let mut documents = Vec::new();

					let mut file_id_span = HashMap::new();
					for input in inputs {
						match load::Document::load(&mut files, &input.filename) {
							Ok((doc, file_id)) => {
								file_id_span.insert(file_id, input.span);
								documents.push(doc)
							}
							Err(e) => {
								abort!(input.span, "{}", e)
							}
						}
					}

					let mut vocabulary = rdf_types::IndexVocabulary::new();
					let mut generator = rdf_types::generator::Blank::new();
					let mut build_context = load::BuildContext::new();

					match load::build_all(
						&mut vocabulary,
						&mut generator,
						&mut build_context,
						documents,
					) {
						Ok(model) => {
							let mut gen_context =
								treeldr_rust_gen::Context::new(&model, &vocabulary, options);
							module.bind(&vocabulary, &mut gen_context);

							gen_context.run_pre_computations();
							match module.generate(&gen_context) {
								Ok(tokens) => tokens.into(),
								Err(e) => {
									use treeldr_rust_gen::fmt::Display;
									abort_call_site!("{}", e.display(&gen_context))
								}
							}
						}
						Err(e) => {
							use load::reporting::Diagnose;

							let diagnostic = e.with(&vocabulary).diagnostic();
							let writer = StandardStream::stderr(ColorChoice::Always);
							let config = codespan_reporting::term::Config::default();
							term::emit(&mut writer.lock(), &config, &files, &diagnostic)
								.expect("diagnostic failed");

							abort_call_site!("compilation failed")
						}
					}
				}
				Err((e, span)) => {
					abort!(span, "{}", e)
				}
			}
		}
		Err((e, span)) => {
			abort!(span, "{}", e)
		}
	}
}
