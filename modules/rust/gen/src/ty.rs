use crate::{
	module::{self, TraitId, TraitImpl},
	path,
	tr::{CollectContextBounds, ContextBound},
	Context, Path,
};

use quote::format_ident;
pub use treeldr::layout::Primitive;
use treeldr::{value::Literal, Name, TId};

pub mod alias;
pub mod enumeration;
mod generate;
pub mod params;
pub mod structure;

use alias::Alias;
use enumeration::Enum;
pub use params::{Parameter, Parameters, ParametersValues};
use structure::Struct;

#[derive(Debug)]
pub struct Type {
	module: Option<module::Parent>,
	desc: Description,
	label: Option<String>,
	doc: treeldr::StrippedDocumentation,
}

impl Type {
	pub fn new(
		module: Option<module::Parent>,
		desc: Description,
		label: Option<String>,
		doc: treeldr::StrippedDocumentation,
	) -> Self {
		Self {
			module,
			desc,
			label,
			doc,
		}
	}

	pub fn label(&self) -> Option<&str> {
		self.label.as_deref()
	}

	pub fn documentation(&self) -> &treeldr::StrippedDocumentation {
		&self.doc
	}

	pub fn ident(&self) -> proc_macro2::Ident {
		match self.description() {
			Description::Never => format_ident!("!"),
			Description::Alias(a) => a.ident().clone(),
			Description::Struct(s) => s.ident().clone(),
			Description::Enum(e) => e.ident().clone(),
			Description::Primitive(_) => {
				todo!()
			}
			Description::BuiltIn(_) => {
				todo!()
			}
			Description::Reference(_) => {
				format_ident!("I")
			}
		}
	}

	pub(crate) fn compute_params(
		&self,
		dependency_params: impl FnMut(TId<treeldr::Layout>) -> Parameters,
	) -> Parameters {
		match self.description() {
			Description::Alias(a) => a.compute_params(dependency_params),
			Description::Reference(_) => Parameters::identifier_parameter(),
			Description::Struct(s) => s.compute_params(dependency_params),
			Description::Enum(e) => e.compute_params(dependency_params),
			Description::BuiltIn(p) => p.compute_params(dependency_params),
			Description::Never | Description::Primitive(_) => Parameters::default(),
		}
	}

	pub(crate) fn set_params(&mut self, p: Parameters) {
		match &mut self.desc {
			Description::Alias(a) => a.set_params(p),
			Description::Struct(s) => s.set_params(p),
			Description::Enum(e) => e.set_params(p),
			_ => (),
		}
	}

	pub fn params(&self) -> Parameters {
		match self.description() {
			Description::Struct(s) => s.params(),
			Description::Enum(e) => e.params(),
			_ => Parameters::default(),
		}
	}

	pub fn path<V, M>(&self, context: &Context<V, M>, ident: proc_macro2::Ident) -> Option<Path> {
		let mut path = context.parent_module_path(self.module)?;
		path.push(path::Segment::Ident(ident));
		*path.parameters_mut() = self.params();
		Some(path)
	}

	pub fn impl_default<V, M>(&self, context: &Context<V, M>) -> bool {
		self.desc.impl_default(context)
	}

	pub fn module(&self) -> Option<module::Parent> {
		self.module
	}

	pub fn description(&self) -> &Description {
		&self.desc
	}

	/// Collect all the trait implementation required for this type.
	pub fn collect_trait_implementations<V, M>(
		&self,
		context: &Context<V, M>,
		mut f: impl FnMut(TraitImpl) -> bool,
	) {
		let layout = match self.description() {
			Description::Struct(s) => Some(s.layout()),
			Description::Enum(e) => Some(e.layout()),
			Description::Primitive(p) => Some(TId::new(p.id())),
			_ => None,
		};

		if let Some(layout_ref) = layout {
			let layout = context.model().get(layout_ref).unwrap();

			if context.options().impl_rdf {
				f(TraitId::FromRdf.impl_for(layout_ref));
				f(TraitId::IntoJsonLd.impl_for(layout_ref));
				f(TraitId::TriplesAndValues.impl_for(layout_ref));
			}

			f(TraitId::IntoJsonLdSyntax.impl_for(layout_ref));

			let mut stack: Vec<_> = layout.as_layout().ty().iter().map(|v| **v.value).collect();
			while let Some(ty_ref) = stack.pop() {
				if f(TraitId::Defined(ty_ref).impl_for(layout_ref)) {
					let ty = context.model().get(ty_ref).unwrap();
					if let Some(super_classes) = ty.as_type().sub_class_of() {
						stack.extend(super_classes.iter().map(|s| **s.value))
					}
				}
			}
		}
	}
}

impl CollectContextBounds for Type {
	fn collect_context_bounds_from<V, M>(
		&self,
		context: &Context<V, M>,
		tr: TId<treeldr::Type>,
		visited: &mut std::collections::HashSet<TId<treeldr::Layout>>,
		f: &mut impl FnMut(ContextBound),
	) {
		match self.description() {
			Description::Struct(s) => s.collect_context_bounds_from(context, tr, visited, f),
			Description::Enum(e) => e.collect_context_bounds_from(context, tr, visited, f),
			Description::Reference(tr) => f(ContextBound(*tr)),
			Description::BuiltIn(b) => match b {
				BuiltIn::Required(item) => {
					item.collect_context_bounds_from(context, tr, visited, f)
				}
				BuiltIn::Option(item) => item.collect_context_bounds_from(context, tr, visited, f),
				BuiltIn::Vec(item) => item.collect_context_bounds_from(context, tr, visited, f),
				BuiltIn::BTreeSet(item) => {
					item.collect_context_bounds_from(context, tr, visited, f)
				}
				BuiltIn::BTreeMap(key, value) => {
					key.collect_context_bounds_from(context, tr, visited, f);
					value.collect_context_bounds_from(context, tr, visited, f)
				}
				BuiltIn::OneOrMany(item) => {
					item.collect_context_bounds_from(context, tr, visited, f)
				}
			},
			_ => (),
		}
	}
}

#[derive(Debug)]
pub enum Description {
	BuiltIn(BuiltIn),
	Never,
	Alias(Alias),
	Reference(TId<treeldr::Type>),
	Primitive(Primitive),
	Struct(Struct),
	Enum(Enum),
}

impl Description {
	pub fn impl_default<V, M>(&self, context: &Context<V, M>) -> bool {
		match self {
			Self::BuiltIn(b) => b.impl_default(),
			Self::Never => false,
			Self::Alias(a) => {
				let ty = context.layout_type(a.target()).unwrap();
				ty.impl_default(context)
			}
			Self::Reference(_) => false,
			Self::Primitive(_) => false,
			Self::Struct(s) => s.impl_default(context),
			Self::Enum(_) => false,
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub enum BuiltIn {
	/// Required type, erased.
	Required(TId<treeldr::Layout>),

	/// Option.
	Option(TId<treeldr::Layout>),

	/// Vec.
	Vec(TId<treeldr::Layout>),

	/// BTreeSet.
	BTreeSet(TId<treeldr::Layout>),

	/// BTreeMap.
	BTreeMap(TId<treeldr::Layout>, TId<treeldr::Layout>),

	/// OneOrMany, for non empty sets.
	OneOrMany(TId<treeldr::Layout>),
}

impl BuiltIn {
	pub fn impl_default(&self) -> bool {
		!matches!(self, Self::Required(_))
	}

	pub(crate) fn compute_params(
		&self,
		mut dependency_params: impl FnMut(TId<treeldr::Layout>) -> Parameters,
	) -> Parameters {
		match self {
			Self::BTreeSet(i) => dependency_params(*i),
			Self::BTreeMap(k, v) => dependency_params(*k).union_with(dependency_params(*v)),
			Self::OneOrMany(i) => dependency_params(*i),
			Self::Option(i) => dependency_params(*i),
			Self::Required(i) => dependency_params(*i),
			Self::Vec(i) => dependency_params(*i),
		}
	}
}

impl Description {
	pub fn new<V, M>(context: &mut Context<V, M>, layout_ref: TId<treeldr::Layout>) -> Self {
		let layout = context
			.model()
			.get(layout_ref)
			.expect("undefined described layout");

		match layout.as_layout().description() {
			treeldr::layout::Description::Never => Self::Never,
			treeldr::layout::Description::Alias(alias_ref) => {
				let name = layout.as_component().name().expect("unnamed alias");
				let ident = generate::type_ident_of_name(name);
				Self::Alias(Alias::new(ident, layout_ref, *alias_ref.value()))
			}
			treeldr::layout::Description::Primitive(p) => Self::Primitive(*p),
			treeldr::layout::Description::Derived(p) => {
				if p.is_restricted() {
					todo!("restricted primitives")
				} else {
					Self::Primitive(p.primitive())
				}
			}
			treeldr::layout::Description::Reference(_) => {
				Self::Reference(**layout.as_layout().ty().first().unwrap().value)
			}
			treeldr::layout::Description::Struct(s) => {
				let ident = layout
					.as_component()
					.name()
					.map(generate::type_ident_of_name)
					.unwrap_or_else(|| context.next_anonymous_type_ident());
				let mut fields = Vec::with_capacity(s.fields().len());
				for (i, field_id) in s.fields().iter().enumerate() {
					let field = context.model().get(**field_id).unwrap();
					let field_name = field
						.as_component()
						.name()
						.cloned()
						.unwrap_or_else(|| Name::new(format!("field_{i}_")).unwrap());
					let field_ident = generate::field_ident_of_name(&field_name);
					fields.push(structure::Field::new(
						field_name,
						field_ident,
						field.as_formatted().format().expect("missing field layout"),
						field.as_layout_field().property().copied(),
						field.preferred_label().map(Literal::to_string),
						field.comment().clone_stripped(),
					))
				}

				Self::Struct(Struct::new(layout_ref, ident, fields))
			}
			treeldr::layout::Description::Enum(e) => {
				let name = layout.as_component().name().expect("unnamed enum");
				let ident = generate::type_ident_of_name(name);
				let mut variants = Vec::with_capacity(e.variants().len());
				for variant_id in e.variants() {
					let variant = context.model().get(**variant_id).unwrap();
					let variant_name = variant.as_component().name().expect("unnamed variant");
					let ident = generate::variant_ident_of_name(variant_name);
					variants.push(enumeration::Variant::new(
						ident,
						variant.as_formatted().format().as_ref().copied(),
					))
				}

				Self::Enum(Enum::new(layout_ref, ident, variants))
			}
			treeldr::layout::Description::Required(r) => {
				Self::BuiltIn(BuiltIn::Required(**r.item_layout()))
			}
			treeldr::layout::Description::Option(o) => {
				Self::BuiltIn(BuiltIn::Option(**o.item_layout()))
			}
			treeldr::layout::Description::Array(a) => {
				Self::BuiltIn(BuiltIn::Vec(**a.item_layout()))
			}
			treeldr::layout::Description::Set(s) => {
				Self::BuiltIn(BuiltIn::BTreeSet(**s.item_layout()))
			}
			treeldr::layout::Description::Map(m) => {
				Self::BuiltIn(BuiltIn::BTreeMap(**m.key_layout(), **m.value_layout()))
			}
			treeldr::layout::Description::OneOrMany(s) => {
				Self::BuiltIn(BuiltIn::OneOrMany(**s.item_layout()))
			}
		}
	}
}
