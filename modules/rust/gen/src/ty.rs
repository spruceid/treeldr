use crate::{
	module::{self, TraitId, TraitImpl},
	path, syntax,
	tr::{CollectContextBounds, ContextBound},
	Context, Error, GenerateSyntax, Path, Referenced, Scope,
};

use quote::{format_ident, quote};
use rdf_types::Vocabulary;
pub use treeldr::layout::Primitive;
use treeldr::{value::Literal, BlankIdIndex, IriIndex, Name, TId};

pub mod alias;
pub mod built_in;
pub mod enumeration;
pub mod params;
pub mod primitive;
pub mod structure;

pub use alias::Alias;
pub use built_in::BuiltIn;
pub use enumeration::Enum;
pub use params::{Parameter, Parameters};
pub use structure::Struct;

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
			Description::DerivedPrimitive(r) => r.ident().clone(),
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
			Description::Never | Description::Primitive(_) | Description::DerivedPrimitive(_) => {
				Parameters::default()
			}
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
			Description::DerivedPrimitive(d) => Some(d.layout()),
			_ => None,
		};

		if let Some(layout_ref) = layout {
			let layout = context.model().get(layout_ref).unwrap();

			if context.options().impl_rdf {
				f(TraitId::FromRdf.impl_for(layout_ref));
				f(TraitId::AsJsonLd.impl_for(layout_ref));
				f(TraitId::IntoJsonLd.impl_for(layout_ref));
				f(TraitId::TriplesAndValues.impl_for(layout_ref));
			}

			f(TraitId::IntoJsonLdSyntax.impl_for(layout_ref));

			let mut stack: Vec<_> = layout.as_layout().ty().iter().map(|v| **v.value).collect();
			while let Some(ty_ref) = stack.pop() {
				if f(TraitId::Class(ty_ref).impl_for(layout_ref)) {
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

pub enum Description {
	BuiltIn(BuiltIn),
	Never,
	Alias(Alias),
	Reference(TId<treeldr::Type>),
	Primitive(Primitive),
	DerivedPrimitive(primitive::Derived),
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
			Self::DerivedPrimitive(_) => false,
			Self::Struct(s) => s.impl_default(context),
			Self::Enum(_) => false,
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
				let ident = type_ident_of_name(name);
				Self::Alias(Alias::new(ident, layout_ref, *alias_ref.value()))
			}
			treeldr::layout::Description::Primitive(p) => Self::Primitive(*p),
			treeldr::layout::Description::Derived(p) => {
				let ident = layout
					.as_component()
					.name()
					.map(type_ident_of_name)
					.unwrap_or_else(|| context.next_anonymous_type_ident());

				Self::DerivedPrimitive(primitive::Derived::new(
					layout_ref,
					ident,
					p.primitive().layout(),
					p.restrictions()
						.into_iter()
						.flat_map(|r| r.iter().map(primitive::Restriction::new))
						.collect(),
					p.default_value().clone_into_literal(),
				))
			}
			treeldr::layout::Description::Reference(_) => {
				Self::Reference(**layout.as_layout().ty().first().unwrap().value)
			}
			treeldr::layout::Description::Struct(s) => {
				let ident = layout
					.as_component()
					.name()
					.map(type_ident_of_name)
					.unwrap_or_else(|| context.next_anonymous_type_ident());
				let mut fields = Vec::with_capacity(s.fields().len());
				for (i, field_id) in s.fields().iter().enumerate() {
					let field = context.model().get(**field_id).unwrap();
					let field_name = field
						.as_component()
						.name()
						.cloned()
						.unwrap_or_else(|| Name::new(format!("field_{i}_")).unwrap());
					let field_ident = field_ident_of_name(&field_name);
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
				let ident = type_ident_of_name(name);
				let mut variants = Vec::with_capacity(e.variants().len());
				for variant_id in e.variants() {
					let variant = context.model().get(**variant_id).unwrap();
					let variant_name = variant.as_component().name().expect("unnamed variant");
					let ident = variant_ident_of_name(variant_name);
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

impl<M> GenerateSyntax<M> for Type {
	type Output = Option<syntax::LayoutTypeDefinition>;

	fn generate_syntax<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: &Scope,
	) -> Result<Self::Output, Error> {
		match self.description() {
			Description::Alias(a) => Ok(Some(syntax::LayoutTypeDefinition::Alias(
				a.generate_syntax(context, scope)?,
			))),
			Description::Struct(s) => Ok(Some(syntax::LayoutTypeDefinition::Struct(
				s.generate_syntax(context, scope)?,
			))),
			Description::Enum(e) => Ok(Some(syntax::LayoutTypeDefinition::Enum(
				e.generate_syntax(context, scope)?,
			))),
			Description::DerivedPrimitive(r) => {
				Ok(Some(syntax::LayoutTypeDefinition::RestrictedPrimitive(
					r.generate_syntax(context, scope)?,
				)))
			}
			_ => Ok(None),
		}
	}
}

impl<M> GenerateSyntax<M> for TId<treeldr::Layout> {
	type Output = syn::Type;

	fn generate_syntax<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: &Scope,
	) -> Result<Self::Output, Error> {
		let ty = context
			.layout_type(*self)
			.expect("undefined generated layout");
		match ty.description() {
			Description::Never => Ok(syn::parse2(quote!(::std::convert::Infallible)).unwrap()),
			Description::Primitive(p) => p.generate_syntax(context, scope),
			Description::DerivedPrimitive(r) => {
				let path = context
					.module_path(scope.module)
					.to(&ty
						.path(context, r.ident().clone())
						.ok_or(Error::UnreachableType(*self))?)
					.generate_syntax(context, scope)?;

				Ok(syn::Type::Path(syn::TypePath { qself: None, path }))
			}
			Description::Alias(a) => {
				let path = context
					.module_path(scope.module)
					.to(&ty
						.path(context, a.ident().clone())
						.ok_or(Error::UnreachableType(*self))?)
					.generate_syntax(context, scope)?;

				Ok(syn::Type::Path(syn::TypePath { qself: None, path }))
			}
			Description::Struct(s) => {
				let path = context
					.module_path(scope.module)
					.to(&ty
						.path(context, s.ident().clone())
						.ok_or(Error::UnreachableType(*self))?)
					.generate_syntax(context, scope)?;

				Ok(syn::Type::Path(syn::TypePath { qself: None, path }))
			}
			Description::Enum(e) => {
				let path = context
					.module_path(scope.module)
					.to(&ty
						.path(context, e.ident().clone())
						.ok_or(Error::UnreachableType(*self))?)
					.generate_syntax(context, scope)?;

				Ok(syn::Type::Path(syn::TypePath { qself: None, path }))
			}
			Description::Reference(_) => {
				let id = scope
					.bound_params()
					.get(crate::ty::Parameter::Identifier)
					.unwrap();
				Ok(syn::parse2(quote!(::treeldr_rust_prelude::Id<#id>)).unwrap())
			}
			Description::BuiltIn(b) => b.generate_syntax(context, scope),
		}
	}
}

impl<M> GenerateSyntax<M> for Referenced<TId<treeldr::Layout>> {
	type Output = syn::Type;

	fn generate_syntax<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: &Scope,
	) -> Result<Self::Output, Error> {
		let ty = context
			.layout_type(self.0)
			.expect("undefined generated layout");
		match ty.description() {
			Description::Never => Ok(syn::parse2(quote!(::std::convert::Infallible)).unwrap()),
			Description::Primitive(p) => Referenced(*p).generate_syntax(context, scope),
			Description::DerivedPrimitive(r) => {
				let path = context
					.module_path(scope.module)
					.to(&ty
						.path(context, r.ident().clone())
						.ok_or(Error::UnreachableType(self.0))?)
					.generate_syntax(context, scope)?;

				Ok(syn::Type::Path(syn::TypePath { qself: None, path }))
			}
			Description::Alias(a) => {
				let path = context
					.module_path(scope.module)
					.to(&ty
						.path(context, a.ident().clone())
						.ok_or(Error::UnreachableType(self.0))?)
					.generate_syntax(context, scope)?;

				Ok(syn::Type::Path(syn::TypePath { qself: None, path }))
			}
			Description::Struct(s) => {
				let path = context
					.module_path(scope.module)
					.to(&ty
						.path(context, s.ident().clone())
						.ok_or(Error::UnreachableType(self.0))?)
					.generate_syntax(context, scope)?;

				Ok(syn::Type::Path(syn::TypePath { qself: None, path }))
			}
			Description::Enum(e) => {
				let path = context
					.module_path(scope.module)
					.to(&ty
						.path(context, e.ident().clone())
						.ok_or(Error::UnreachableType(self.0))?)
					.generate_syntax(context, scope)?;

				Ok(syn::Type::Path(syn::TypePath { qself: None, path }))
			}
			Description::Reference(_) => {
				let id = scope
					.bound_params()
					.get(crate::ty::Parameter::Identifier)
					.unwrap();
				Ok(syn::parse2(quote!(&::treeldr_rust_prelude::Id<#id>)).unwrap())
			}
			Description::BuiltIn(b) => Referenced(*b).generate_syntax(context, scope),
		}
	}
}

pub struct InContext<T>(pub T);

impl<M> GenerateSyntax<M> for InContext<TId<treeldr::Layout>> {
	type Output = syn::Type;

	fn generate_syntax<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: &Scope,
	) -> Result<Self::Output, Error> {
		let ty = context
			.layout_type(self.0)
			.expect("undefined generated layout");
		match ty.description() {
			Description::Never => Ok(syn::parse2(quote!(::std::convert::Infallible)).unwrap()),
			Description::Primitive(p) => p.generate_syntax(context, scope),
			Description::DerivedPrimitive(r) => {
				let path = context
					.module_path(scope.module)
					.to(&ty
						.path(context, r.ident().clone())
						.ok_or(Error::UnreachableType(self.0))?)
					.generate_syntax(context, scope)?;

				Ok(syn::Type::Path(syn::TypePath { qself: None, path }))
			}
			Description::Alias(a) => {
				let path = context
					.module_path(scope.module)
					.to(&ty
						.path(context, a.ident().clone())
						.ok_or(Error::UnreachableType(self.0))?)
					.generate_syntax(context, scope)?;

				Ok(syn::Type::Path(syn::TypePath { qself: None, path }))
			}
			Description::Struct(s) => {
				let path = context
					.module_path(scope.module)
					.to(&ty
						.path(context, s.ident().clone())
						.ok_or(Error::UnreachableType(self.0))?)
					.generate_syntax(context, scope)?;

				Ok(syn::Type::Path(syn::TypePath { qself: None, path }))
			}
			Description::Enum(e) => {
				let path = context
					.module_path(scope.module)
					.to(&ty
						.path(context, e.ident().clone())
						.ok_or(Error::UnreachableType(self.0))?)
					.generate_syntax(context, scope)?;

				Ok(syn::Type::Path(syn::TypePath { qself: None, path }))
			}
			Description::Reference(ty_id) => {
				let tr = context.type_trait(*ty_id).unwrap();
				let ident = tr.ident();
				let context_path = context
					.module_path(scope.module)
					.to(&tr
						.context_path(context)
						.ok_or_else(|| Error::unreachable_trait(*ty_id))?)
					.generate_syntax(context, scope)?;
				Ok(syn::parse2(quote! { <C as #context_path >::#ident }).unwrap())
			}
			Description::BuiltIn(b) => b.generate_syntax(context, scope),
		}
	}
}

pub fn type_ident_of_name(name: &treeldr::Name) -> proc_macro2::Ident {
	quote::format_ident!("{}", name.to_pascal_case())
}

pub fn field_ident_of_name(name: &treeldr::Name) -> proc_macro2::Ident {
	let mut name = name.to_snake_case();
	if matches!(name.as_str(), "type") {
		name.push('_')
	}

	quote::format_ident!("{}", name)
}

pub fn variant_ident_of_name(name: &treeldr::Name) -> proc_macro2::Ident {
	quote::format_ident!("{}", name.to_pascal_case())
}
