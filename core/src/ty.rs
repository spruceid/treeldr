use crate::{layout, prop, Causes, Documentation, Id};
use derivative::Derivative;
use shelves::Ref;
use std::collections::HashMap;

/// Type definition.
pub struct Definition<F> {
	/// Identifier.
	id: Id,

	/// Properties.
	properties: HashMap<Ref<prop::Definition<F>>, Causes<F>>,

	/// Documentation.
	doc: Documentation,

	/// Causes of the definition.
	causes: Causes<F>,
}

impl<F> Definition<F> {
	pub fn new(id: Id, causes: impl Into<Causes<F>>) -> Self {
		Self {
			id,
			causes: causes.into(),
			properties: HashMap::new(),
			doc: Documentation::default(),
		}
	}

	/// Returns the identifier of the defined type.
	pub fn id(&self) -> Id {
		self.id
	}

	pub fn causes(&self) -> &Causes<F> {
		&self.causes
	}

	pub fn documentation(&self) -> &Documentation {
		&self.doc
	}

	pub fn documentation_mut(&mut self) -> &mut Documentation {
		&mut self.doc
	}

	pub fn set_documentation(&mut self, doc: Documentation) {
		self.doc = doc
	}

	pub fn properties(&self) -> impl Iterator<Item = (Ref<prop::Definition<F>>, &Causes<F>)> {
		self.properties.iter().map(|(p, c)| (*p, c))
	}

	pub fn insert_property(
		&mut self,
		prop_ref: Ref<prop::Definition<F>>,
		causes: impl Into<Causes<F>>,
	) where
		F: Ord,
	{
		self.properties.insert(prop_ref, causes.into());
	}

	// pub fn default_fields(
	// 	&self,
	// 	model: &crate::Model<F>,
	// ) -> Result<Vec<layout::Field<F>>, Error<F>> where F: Clone {
	// 	struct PropertyIri<'a, F>(Iri<'a>, Ref<prop::Definition<F>>, &'a Causes<F>);

	// 	impl<'a, F> PartialEq for PropertyIri<'a, F> {
	// 		fn eq(&self, other: &Self) -> bool {
	// 			self.0 == other.0
	// 		}
	// 	}

	// 	impl<'a, F> Eq for PropertyIri<'a, F> {}

	// 	impl<'a, F> std::cmp::Ord for PropertyIri<'a, F> {
	// 		fn cmp(&self, other: &Self) -> std::cmp::Ordering {
	// 			self.0.cmp(&other.0)
	// 		}
	// 	}

	// 	impl<'a, F> std::cmp::PartialOrd for PropertyIri<'a, F> {
	// 		fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
	// 			self.0.partial_cmp(&other.0)
	// 		}
	// 	}

	// 	let mut properties: Vec<PropertyIri<F>> = self
	// 		.properties
	// 		.iter()
	// 		.map(|(prop_ref, causes)| {
	// 			let prop = model.properties().get(*prop_ref).unwrap();
	// 			let iri = model.vocabulary().get(prop.id()).unwrap();
	// 			PropertyIri(iri, *prop_ref, causes)
	// 		})
	// 		.collect();

	// 	properties.sort();

	// 	let mut fields = Vec::with_capacity(properties.len());
	// 	for PropertyIri(iri, prop_ref, causes) in properties {
	// 		let prop = model.properties().get(prop_ref).unwrap();
	// 		let name = iri
	// 			.path()
	// 			.file_name()
	// 			.expect("invalid property IRI")
	// 			.to_owned();
	// 		let layout_expr = match prop.ty() {
	// 			Some(ty) => ty
	// 				.expr()
	// 				.default_layout(model, ty.causes().preferred().cloned())?,
	// 			None => panic!("no known type"),
	// 		};

	// 		let mut field = layout::Field::new(
	// 			prop_ref,
	// 			name,
	// 			layout_expr,
	// 			causes.clone(),
	// 		);

	// 		field.set_required(prop.is_required());
	// 		field.set_functional(prop.is_functional());

	// 		fields.push(field);
	// 	}

	// 	Ok(fields)
	// }
}

// impl<F> Ref<Definition<F>> {
// 	pub fn with_model<'c>(&self, context: &'c crate::Model<F>) -> RefWithContext<'c, F> {
// 		RefWithContext(context, *self)
// 	}
// }

// pub struct RefWithContext<'c, F>(&'c crate::Model<F>, Ref<Definition<F>>);

// impl<'c, F> fmt::Display for RefWithContext<'c, F> {
// 	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
// 		let id = self.0.types().get(self.1).unwrap().id();
// 		id.display(self.0.vocabulary()).fmt(f)
// 	}
// }

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Copy(bound = ""))]
pub struct Expr<F> {
	ty_ref: Ref<Definition<F>>,
	implicit_layout_ref: Option<Ref<layout::Definition<F>>>,
}

impl<F> Expr<F> {
	pub fn new(
		ty_ref: Ref<Definition<F>>,
		implicit_layout_ref: Option<Ref<layout::Definition<F>>>,
	) -> Self {
		Self {
			ty_ref,
			implicit_layout_ref,
		}
	}

	pub fn ty(&self) -> Ref<Definition<F>> {
		self.ty_ref
	}

	pub fn implicit_layout(&self) -> Option<Ref<layout::Definition<F>>> {
		self.implicit_layout_ref
	}

	pub fn set_implicit_layout(&mut self, l: Ref<layout::Definition<F>>) {
		self.implicit_layout_ref = Some(l)
	}
}
