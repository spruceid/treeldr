use locspan::Meta;
use rdf_types::Generator;
use treeldr::{metadata::Merge, vocab::TldrVocabulary, Id};
use treeldr_build::Context;

use super::{Build, Declare, Error, LocalContext};

impl<M: Clone + Merge> Declare<M> for Meta<crate::PropertyDefinition<M>, M> {
	fn declare(
		&self,
		local_context: &mut LocalContext<M>,
		context: &mut Context<M>,
		vocabulary: &mut TldrVocabulary,
		generator: &mut impl Generator<TldrVocabulary>,
	) -> Result<(), Error<M>> {
		let Meta(id, _) = self
			.id
			.clone()
			.build(local_context, context, vocabulary, generator)?;
		context.declare_property(id, self.metadata().clone());
		Ok(())
	}
}

impl<M: Clone + Merge> Build<M> for Meta<crate::PropertyDefinition<M>, M> {
	type Target = Meta<Id, M>;

	fn build(
		self,
		local_context: &mut LocalContext<M>,
		context: &mut Context<M>,
		vocabulary: &mut TldrVocabulary,
		generator: &mut impl Generator<TldrVocabulary>,
	) -> Result<Self::Target, Error<M>> {
		let Meta(def, _) = self;
		let Meta(id, id_loc) = def
			.id
			.build(local_context, context, vocabulary, generator)?;

		let doc = def
			.doc
			.map(|doc| doc.build(local_context, context, vocabulary, generator))
			.transpose()?
			.flatten();

		let mut functional = true;
		let mut functional_loc = None;
		let mut required = false;
		let mut required_loc = None;

		let range = def
			.ty
			.map(|Meta(ty, _)| -> Result<_, Error<M>> {
				let scope = local_context.scope.take();
				let range = ty
					.expr
					.build(local_context, context, vocabulary, generator)?;
				local_context.scope = scope;

				for Meta(ann, ann_loc) in ty.annotations {
					match ann {
						crate::Annotation::Multiple => {
							functional = false;
						}
						crate::Annotation::Required => {
							required = true;
							required_loc = Some(ann_loc);
						}
						crate::Annotation::Single => {
							functional = true;
							functional_loc = Some(ann_loc);
						}
					}
				}

				Ok(range)
			})
			.transpose()?;

		let node = context.get_mut(id).unwrap();
		if let Some(comment) = doc {
			node.comment_mut().insert_base(comment.cast())
		}

		if functional {
			node.type_mut().insert_base(Meta(
				treeldr_build::prop::Type::FunctionalProperty.into(),
				functional_loc.unwrap_or_else(|| id_loc.clone()),
			));
		}

		let prop = node.as_property_mut();
		if let Some(Meta(range, range_loc)) = range {
			prop.range_mut().insert_base(Meta(range, range_loc));
		}

		if let Some(required_loc) = required_loc {
			prop.required_mut()
				.insert_base(Meta(required, required_loc));
		}

		Ok(Meta(id, id_loc))
	}
}
