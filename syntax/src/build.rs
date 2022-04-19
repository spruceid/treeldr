use treeldr::{
	WithCauses,
	Causes,
	Caused,
	Id,
	vocab::*,
	Vocabulary
};
use treeldr_build::{
	Context,
	context,
	SubLayout,
	ParentLayout,
	Item,
	Dependencies
};
use iref::{
	IriBuf,
	IriRef
};
use locspan::{
	Location,
	Loc
};
use std::collections::{
	HashMap,
	BTreeMap
};
use std::fmt;

pub enum Error<F> {
	Global(treeldr_build::Error<F>),
	Local(Loc<LocalError<F>, F>)
}

impl<F> From<treeldr_build::Error<F>> for Error<F> {
	fn from(e: treeldr_build::Error<F>) -> Self {
		Self::Global(e)
	}
}

impl<F> From<Loc<LocalError<F>, F>> for Error<F> {
	fn from(e: Loc<LocalError<F>, F>) -> Self {
		Self::Local(e)
	}
}

#[derive(Debug)]
pub enum LocalError<F> {
	InvalidExpandedCompactIri(String),
	UndefinedPrefix(String),
	AlreadyDefinedPrefix(String, Location<F>),
	NoBaseIri,
	BaseIriMismatch {
		expected: Box<IriBuf>,
		found: Box<IriBuf>,
		because: Location<F>,
	},
	TypeAlias(Id, Location<F>),
	LayoutAlias(Id, Location<F>),
}

impl<'c, 't, F: Clone> crate::reporting::Diagnose<F> for Loc<LocalError<F>, F> {
	fn message(&self) -> String {
		match self.value() {
			LocalError::InvalidExpandedCompactIri(_) => "invalid expanded compact IRI".to_string(),
			LocalError::UndefinedPrefix(_) => "undefined prefix".to_string(),
			LocalError::AlreadyDefinedPrefix(_, _) => "already defined prefix".to_string(),
			LocalError::NoBaseIri => "no base IRI".to_string(),
			LocalError::BaseIriMismatch { .. } => "base IRI mismatch".to_string(),
			LocalError::TypeAlias(_, _) => "type aliases are not supported".to_string(),
			LocalError::LayoutAlias(_, _) => "layout aliases are not supported".to_string(),
		}
	}

	fn labels(&self) -> Vec<codespan_reporting::diagnostic::Label<F>> {
		let mut labels = vec![self
			.location()
			.clone()
			.into_primary_label()
			.with_message(self.to_string())];

		match self.value() {
			LocalError::AlreadyDefinedPrefix(_, original_loc) => labels.push(
				original_loc
					.clone()
					.into_secondary_label()
					.with_message("original prefix defined here".to_string()),
			),
			LocalError::BaseIriMismatch { because, .. } => labels.push(
				because
					.clone()
					.into_secondary_label()
					.with_message("original base IRI defined here".to_string()),
			),
			_ => (),
		}

		labels
	}
}

impl<F> fmt::Display for LocalError<F> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::InvalidExpandedCompactIri(expanded) => {
				write!(f, "`{}` is not a valid IRI", expanded)
			}
			Self::UndefinedPrefix(prefix) => write!(f, "prefix `{}` is undefined", prefix),
			Self::AlreadyDefinedPrefix(prefix, _) => {
				write!(f, "prefix `{}` is already defined", prefix)
			}
			Self::NoBaseIri => "cannot resolve the IRI reference without a base IRI".fmt(f),
			Self::BaseIriMismatch { expected, .. } => write!(f, "should be `{}`", expected),
			Self::TypeAlias(_, _) => write!(f, "type aliases are not supported"),
			Self::LayoutAlias(_, _) => write!(f, "layout aliases are not supported"),
		}
	}
}

pub struct Definitions;

impl<F: Clone + Ord> treeldr_build::Definitions<F> for Definitions {
	type Error = Error<F>;

	type Type = treeldr_build::ty::Definition<F>;
	type Property = treeldr_build::prop::Definition<F>;
	type Layout = Layout<F>;
}

/// Build context.
pub struct LocalContext<F> {
	/// Base IRI of th parsed document.
	base_iri: Option<IriBuf>,

	/// Bound prefixes.
	prefixes: HashMap<String, Loc<IriBuf, F>>,

	/// Current scope.
	scope: Option<Term>,

	next_id: Option<Loc<Id, F>>,

	/// Associates each literal type/value to a blank node label.
	literal: BTreeMap<Loc<crate::Literal, F>, Loc<Id, F>>,

	/// Associates each union type (location) to a blank node label.
	anonymous_types: BTreeMap<Location<F>, Loc<Id, F>>,
}

impl<F> LocalContext<F> {
	pub fn new(base_iri: Option<IriBuf>) -> Self {
		Self {
			base_iri,
			prefixes: HashMap::new(),
			scope: None,
			next_id: None,
			literal: BTreeMap::new(),
			anonymous_types: BTreeMap::new(),
		}
	}

	pub fn next_id(&mut self, vocabulary: &mut Vocabulary, loc: Location<F>) -> Loc<Id, F> {
		self.next_id
			.take()
			.unwrap_or_else(|| Loc(Id::Blank(vocabulary.new_blank_label()), loc))
	}
}

impl<F: Clone> LocalContext<F> {
	pub fn base_iri(&self, vocabulary: &mut Vocabulary, loc: Location<F>) -> Result<IriBuf, Loc<LocalError<F>, F>> {
		match &self.scope {
			Some(scope) => {
				let mut iri = scope.iri(vocabulary).unwrap().to_owned();
				iri.path_mut().open();
				Ok(iri)
			}
			None => self.base_iri.clone().ok_or(Loc(LocalError::NoBaseIri, loc)),
		}
	}

	pub fn set_base_iri(&mut self, base_iri: IriBuf) {
		self.base_iri = Some(base_iri)
	}

	pub fn declare_prefix(
		&mut self,
		prefix: String,
		iri: IriBuf,
		loc: Location<F>,
	) -> Result<(), Loc<LocalError<F>, F>> {
		use std::collections::hash_map::Entry;
		match self.prefixes.entry(prefix) {
			Entry::Occupied(entry) => Err(Loc(
				LocalError::AlreadyDefinedPrefix(entry.key().to_owned(), entry.get().location().clone()),
				loc,
			)),
			Entry::Vacant(entry) => {
				entry.insert(Loc(iri, loc));
				Ok(())
			}
		}
	}

	pub fn expand_compact_iri(
		&self,
		prefix: &str,
		iri_ref: IriRef,
		loc: &Location<F>,
	) -> Result<IriBuf, Loc<LocalError<F>, F>> {
		match self.prefixes.get(prefix) {
			Some(iri) => match IriBuf::try_from(iri.as_str().to_string() + iri_ref.as_str()) {
				Ok(iri) => Ok(iri),
				Err((_, string)) => Err(Loc(LocalError::InvalidExpandedCompactIri(string), loc.clone())),
			},
			None => Err(Loc(LocalError::UndefinedPrefix(prefix.to_owned()), loc.clone())),
		}
	}
}

impl<F: Clone + Ord> treeldr_build::Document<F, Definitions> for crate::Document<F> {
	type LocalContext = LocalContext<F>;
	type Error = Error<F>;

	fn declare(&self, local_context: &mut Self::LocalContext, context: &mut Context<F, Definitions>) -> Result<(), Error<F>> {
		let mut declared_base_iri = None;
		for Loc(base_iri, loc) in &self.bases {
			match declared_base_iri.take() {
				Some(Loc(declared_base_iri, d_loc)) => {
					if declared_base_iri != *base_iri {
						return Err(Loc(
							LocalError::BaseIriMismatch {
								expected: Box::new(declared_base_iri),
								found: Box::new(base_iri.clone()),
								because: d_loc,
							},
							loc.clone(),
						).into());
					}
				}
				None => {
					local_context.set_base_iri(base_iri.clone());
					declared_base_iri = Some(Loc(base_iri.clone(), loc.clone()));
				}
			}
		}

		// for import in doc.uses {
		// 	import.build(ctx, quads)?
		// }

		// for ty in doc.types {
		// 	ty.build(ctx, quads)?
		// }

		// for layout in doc.layouts {
		// 	layout.build(ctx, quads)?
		// }

		Ok(())
	}

	fn relate(self, local_context: &mut Self::LocalContext, context: &mut Context<F, Definitions>) -> Result<(), Error<F>> {
		todo!()
	}
}

pub enum Layout<F> {
	/// Concrete layout definition.
	Concrete(treeldr_build::layout::Definition<F>),

	/// Pseudo layout definition.
	Restriction
}

impl<F: Clone + Ord> treeldr_build::Layout<F, Definitions> for Layout<F> {
	fn sub_layouts(&self, context: &Context<F, Definitions>) -> Result<Vec<SubLayout<F>>, treeldr_build::Error<F>> {
		todo!()
	}

	fn name(&self) -> Option<&WithCauses<Name, F>> {
		todo!()
	}

	fn set_name(&mut self, name: Name, cause: Option<Location<F>>) -> Result<(), treeldr_build::Error<F>> {
		todo!()
	}

	fn default_name(
		&self,
		context: &Context<F, Definitions>,
		parent_layouts: &[WithCauses<ParentLayout, F>],
		cause: Option<Location<F>>,
	) -> Result<Option<Caused<Name, F>>, treeldr_build::Error<F>> {
		todo!()
	}
}

impl<F: Ord + Clone> treeldr_build::Build<F> for Layout<F> {
	type Target = treeldr::layout::Definition<F>;
	type Error = Error<F>;

	fn dependencies(
		&self,
		id: Id,
		nodes: &context::AllocatedNodes<F>,
		causes: &Causes<F>,
	) -> Result<Vec<Item<F>>, Error<F>> {
		todo!()
	}

	fn build(
		mut self,
		id: Id,
		vocab: &Vocabulary,
		nodes: &context::AllocatedNodes<F>,
		dependencies: Dependencies<F>,
		causes: Causes<F>,
	) -> Result<Self::Target, Error<F>> {
		todo!()
	}
}