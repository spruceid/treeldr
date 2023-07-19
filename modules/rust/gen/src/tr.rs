use std::collections::{BTreeSet, HashSet};

use proc_macro2::Ident;
use quote::{format_ident, quote};
use rdf_types::Vocabulary;
use treeldr::{ty::PseudoProperty, value::Literal, Id, IriIndex, Name, TId};

use crate::{module, path, Context, GenerateSyntax, Path};

mod class_provider;
mod generate;

pub use class_provider::ProviderOf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContextBound(pub TId<treeldr::Type>);

pub trait CollectContextBounds {
	fn generate_context_bounds<
		V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>,
		M,
	>(
		&self,
		context: &Context<V, M>,
		tr: TId<treeldr::Type>,
		scope: &crate::Scope,
	) -> Result<Vec<syn::TraitBound>, crate::Error> {
		let mut context_bound_set = BTreeSet::new();
		self.collect_context_bounds(context, tr, |b| {
			context_bound_set.insert(b);
		});

		let mut context_bounds = Vec::with_capacity(context_bound_set.len());
		for b in context_bound_set {
			context_bounds.push(b.generate_syntax(context, scope)?)
		}
		Ok(context_bounds)
	}

	fn collect_context_bounds<V, M>(
		&self,
		context: &Context<V, M>,
		tr: TId<treeldr::Type>,
		mut f: impl FnMut(ContextBound),
	) {
		let mut visited = HashSet::new();
		self.collect_context_bounds_from(context, tr, &mut visited, &mut f)
	}

	fn collect_context_bounds_from<V, M>(
		&self,
		context: &Context<V, M>,
		tr: TId<treeldr::Type>,
		visited: &mut HashSet<TId<treeldr::Layout>>,
		f: &mut impl FnMut(ContextBound),
	);
}

impl CollectContextBounds for TId<treeldr::Layout> {
	fn collect_context_bounds_from<V, M>(
		&self,
		context: &Context<V, M>,
		tr: TId<treeldr::Type>,
		visited: &mut HashSet<TId<treeldr::Layout>>,
		f: &mut impl FnMut(ContextBound),
	) {
		if visited.insert(*self) {
			let ty = context.layout_type(*self).unwrap();
			ty.collect_context_bounds_from(context, tr, visited, f)
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub struct TraitModules {
	/// The main trait definition module.
	pub main: Option<module::Parent>,

	/// Where the `*Provider` trait is defined.
	pub provider: Option<module::Parent>,
}

impl Default for TraitModules {
	fn default() -> Self {
		Self {
			main: Some(module::Parent::Extern),
			provider: Some(module::Parent::Extern),
		}
	}
}

pub struct Trait {
	modules: TraitModules,
	ident: Ident,
	label: Option<String>,
	doc: treeldr::StrippedDocumentation,
	super_traits: Vec<TId<treeldr::Type>>,
	associated_types: Vec<AssociatedType>,
	methods: Vec<Method>,
}

impl Trait {
	pub fn new(
		modules: TraitModules,
		ident: Ident,
		label: Option<String>,
		doc: treeldr::StrippedDocumentation,
		super_traits: Vec<TId<treeldr::Type>>,
		associated_types: Vec<AssociatedType>,
		methods: Vec<Method>,
	) -> Self {
		Self {
			modules,
			ident,
			label,
			doc,
			super_traits,
			associated_types,
			methods,
		}
	}

	pub fn build<V, M>(
		context: &Context<V, M>,
		modules: TraitModules,
		type_ref: TId<treeldr::Type>,
	) -> Option<Self>
	where
		V: Vocabulary<Iri = IriIndex>,
	{
		match type_ref.id() {
			Id::Iri(iri_index) => {
				let ty = context.model().get(type_ref).expect("undefined type");
				let label = ty.preferred_label().map(Literal::to_string);
				let doc = ty.comment().clone_stripped();

				let iri = context.vocabulary().iri(&iri_index).unwrap();
				let ident = Name::from_iri(iri)
					.ok()
					.flatten()
					.map(|name| format_ident!("{}", name.to_pascal_case()));

				ident.map(|ident| {
					let mut super_traits = Vec::new();

					if let Some(sub_class_of) = ty.as_type().sub_class_of() {
						for t in sub_class_of {
							super_traits.push(**t.value)
						}
					}

					let mut associated_types = Vec::new();
					let mut methods = Vec::new();

					for pseudo_prop in context.model().type_properties(type_ref).unwrap() {
						if let PseudoProperty::Property(p) = pseudo_prop {
							if p.property().id()
								!= Id::Iri(IriIndex::Iri(treeldr::vocab::Term::TreeLdr(
									treeldr::vocab::TreeLdr::Self_,
								))) {
								if let Some(name) =
									Name::from_id(context.vocabulary(), p.property().id())
										.ok()
										.flatten()
								{
									let prop = context.model().get(p.property()).unwrap();
									if prop.as_property().domain().contains(&type_ref) {
										methods.push(Method::build(
											context,
											&mut associated_types,
											&name,
											p,
										));
									}
								}
							}
						}
					}

					Self::new(
						modules,
						ident,
						label,
						doc,
						super_traits,
						associated_types,
						methods,
					)
				})
			}
			Id::Blank(_) => None,
		}
	}

	pub fn path<V, M>(&self, context: &Context<V, M>) -> Option<Path> {
		let mut path = context.parent_module_path(self.modules.main)?;
		path.push(path::Segment::Ident(self.ident.clone()));
		path.parameters_mut().identifier = true;
		Some(path)
	}

	pub fn context_path<V, M>(&self, context: &Context<V, M>) -> Option<Path> {
		let mut path = context.parent_module_path(self.modules.provider)?;
		path.push(path::Segment::Ident(self.context_ident()));
		path.parameters_mut().identifier = true;
		Some(path)
	}

	pub fn module(&self) -> Option<module::Parent> {
		self.modules.main
	}

	pub fn ident(&self) -> &Ident {
		&self.ident
	}

	pub fn context_ident(&self) -> Ident {
		format_ident!("{}Provider", self.ident)
	}

	pub fn label(&self) -> Option<&str> {
		self.label.as_deref()
	}

	pub fn documentation(&self) -> &treeldr::StrippedDocumentation {
		&self.doc
	}

	pub fn super_traits(&self) -> &[TId<treeldr::Type>] {
		&self.super_traits
	}

	pub fn associated_types(&self) -> &[AssociatedType] {
		&self.associated_types
	}

	pub fn associated_type_for(&self, prop: TId<treeldr::Property>) -> Option<&AssociatedType> {
		self.associated_types.iter().find(|a| a.prop == prop)
	}

	pub fn methods(&self) -> &[Method] {
		&self.methods
	}
}

/// Trait associated type.
pub struct AssociatedType {
	/// Property.
	prop: TId<treeldr::Property>,

	ident: Ident,

	collection_ident: Option<Ident>,

	/// Label.
	label: Option<String>,

	/// Documentation.
	doc: treeldr::StrippedDocumentation,

	bounds: AssociatedPropertyTypeBounds,
}

impl AssociatedType {
	pub fn new(
		prop: TId<treeldr::Property>,
		ident: Ident,
		collection_ident: Option<Ident>,
		label: Option<String>,
		doc: treeldr::StrippedDocumentation,
		bounds: AssociatedPropertyTypeBounds,
	) -> Self {
		Self {
			prop,
			ident,
			collection_ident,
			label,
			doc,
			bounds,
		}
	}

	pub fn build<V, M>(
		context: &Context<V, M>,
		name: &Name,
		p: treeldr::ty::properties::RestrictedProperty<M>,
	) -> Self {
		let ident = format_ident!("{}", name.to_pascal_case());
		let prop = context.model().get(p.property()).unwrap();
		let label = prop.preferred_label().map(Literal::to_string);
		let doc = prop.comment().clone_stripped();

		let collection_ident = if prop.as_property().is_functional() {
			None
		} else {
			Some(format_ident!("{}s", name.to_pascal_case()))
		};

		Self::new(
			p.property(),
			ident,
			collection_ident,
			label,
			doc,
			AssociatedPropertyTypeBounds(
				prop.as_property()
					.range()
					.iter()
					.map(|r| **r.value)
					.collect(),
			),
		)
	}

	pub fn property(&self) -> TId<treeldr::Property> {
		self.prop
	}

	pub fn ident(&self) -> &Ident {
		&self.ident
	}

	pub fn collection_ident(&self) -> Option<&Ident> {
		self.collection_ident.as_ref()
	}

	pub fn is_collection(&self) -> bool {
		self.collection_ident.is_some()
	}

	pub fn label(&self) -> Option<&str> {
		self.label.as_deref()
	}

	pub fn documentation(&self) -> &treeldr::StrippedDocumentation {
		&self.doc
	}

	pub fn bounds(&self) -> &AssociatedPropertyTypeBounds {
		&self.bounds
	}

	pub fn collection_bound(&self) -> AssociatedCollectionTypeBound {
		AssociatedCollectionTypeBound(&self.ident)
	}
}

pub struct AssociatedPropertyTypeBounds(Vec<TId<treeldr::Type>>);

impl AssociatedPropertyTypeBounds {
	pub fn traits(&self) -> &[TId<treeldr::Type>] {
		&self.0
	}
}

pub struct AssociatedCollectionTypeBound<'a>(&'a Ident);

impl<'a> AssociatedCollectionTypeBound<'a> {
	pub fn item_ident(&self) -> &'a Ident {
		self.0
	}
}

/// Trait method.
pub struct Method {
	/// Property.
	prop: TId<treeldr::Property>,

	/// Identifier.
	ident: Ident,

	/// Label.
	label: Option<String>,

	/// Documentation.
	doc: treeldr::StrippedDocumentation,

	/// Type.
	ty: MethodType,
}

impl Method {
	pub fn new(
		prop: TId<treeldr::Property>,
		ident: Ident,
		label: Option<String>,
		doc: treeldr::StrippedDocumentation,
		ty: MethodType,
	) -> Self {
		Self {
			prop,
			ident,
			label,
			doc,
			ty,
		}
	}

	pub fn build<V, M>(
		context: &Context<V, M>,
		associated_types: &mut Vec<AssociatedType>,
		name: &Name,
		p: treeldr::ty::properties::RestrictedProperty<M>,
	) -> Self {
		let assoc_ty = AssociatedType::build(context, name, p);
		let i = associated_types.len();
		associated_types.push(assoc_ty);

		let ident = match name.as_str() {
			"type" => format_ident!("type_"),
			"for" => format_ident!("for_"),
			_ => format_ident!("{}", name.to_snake_case()),
		};

		let prop = context.model().get(p.property()).unwrap();
		let label = prop.preferred_label().map(Literal::to_string);
		let doc = prop.comment().clone_stripped();

		let ty = if prop.as_property().is_functional() {
			if prop.as_property().is_required() {
				MethodType::Required(i)
			} else {
				MethodType::Option(i)
			}
		} else {
			MethodType::Required(i)
		};

		Self::new(p.property(), ident, label, doc, ty)
	}

	pub fn property(&self) -> TId<treeldr::Property> {
		self.prop
	}

	pub fn ident(&self) -> &Ident {
		&self.ident
	}

	pub fn label(&self) -> Option<&str> {
		self.label.as_deref()
	}

	pub fn documentation(&self) -> &treeldr::StrippedDocumentation {
		&self.doc
	}

	pub fn type_(&self) -> &MethodType {
		&self.ty
	}

	pub fn return_type_expr(&self, tr: &Trait) -> syn::Type {
		match &self.ty {
			MethodType::Required(i) => {
				let a = &tr.associated_types()[*i];
				match a.collection_ident() {
					Some(collection_ident) => {
						syn::parse2(quote!(Self::#collection_ident<'r>)).unwrap()
					}
					None => {
						let ident = a.ident();
						syn::parse2(quote!(&'r Self::#ident)).unwrap()
					}
				}
			}
			MethodType::Option(i) => {
				let a = &tr.associated_types()[*i];
				match a.collection_ident() {
					Some(collection_ident) => {
						syn::parse2(quote!(Option<Self::#collection_ident<'r>>)).unwrap()
					}
					None => {
						let ident = a.ident();
						syn::parse2(quote!(Option<&'r Self::#ident>)).unwrap()
					}
				}
			}
		}
	}
}

/// Method type.
pub enum MethodType {
	/// Direct associated type given by its index.
	///
	/// `Self::T<'_>`.
	Required(usize),

	/// Optional referenced type given by its index.
	///
	/// `Option<Self::T<'a>>`.
	Option(usize),
}
