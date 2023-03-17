use std::collections::HashSet;

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use rdf_types::Vocabulary;
use treeldr::{ty::PseudoProperty, value::Literal, Id, IriIndex, Name, TId};

use crate::{module, path, Context, Path};

mod generate;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContextBound(pub TId<treeldr::Type>);

pub trait CollectContextBounds {
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

pub struct Trait {
	module: Option<module::Parent>,
	ident: Ident,
	label: Option<String>,
	doc: treeldr::StrippedDocumentation,
	super_traits: Vec<TId<treeldr::Type>>,
	associated_types: Vec<AssociatedType>,
	methods: Vec<Method>,
}

impl Trait {
	pub fn new(
		module: Option<module::Parent>,
		ident: Ident,
		label: Option<String>,
		doc: treeldr::StrippedDocumentation,
		super_traits: Vec<TId<treeldr::Type>>,
		associated_types: Vec<AssociatedType>,
		methods: Vec<Method>,
	) -> Self {
		Self {
			module,
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
		module: Option<module::Parent>,
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

				let ident = match context
					.model()
					.get(TId::<treeldr::Layout>::new(type_ref.id()))
				{
					Some(layout) => layout
						.as_component()
						.name()
						.map(|name| format_ident!("Any{}", name.to_pascal_case())),
					None => {
						let iri = context.vocabulary().iri(&iri_index).unwrap();
						Name::from_iri(iri)
							.ok()
							.flatten()
							.map(|name| format_ident!("{}", name.to_pascal_case()))
					}
				};

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
						module,
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
		let mut path = context.parent_module_path(self.module)?;
		path.push(path::Segment::Ident(self.ident.clone()));
		Some(path)
	}

	pub fn context_path<V, M>(&self, context: &Context<V, M>) -> Option<Path> {
		let mut path = context.parent_module_path(self.module)?;
		path.push(path::Segment::Ident(self.context_ident()));
		Some(path)
	}

	pub fn dyn_table_path<V, M>(&self, context: &Context<V, M>) -> Option<Path> {
		let mut path = context.parent_module_path(self.module)?;
		path.push(path::Segment::Ident(self.dyn_table_ident()));
		Some(path)
	}

	pub fn module(&self) -> Option<module::Parent> {
		self.module
	}

	pub fn ident(&self) -> &Ident {
		&self.ident
	}

	pub fn context_ident(&self) -> Ident {
		format_ident!("{}Provider", self.ident)
	}

	pub fn dyn_table_ident(&self) -> Ident {
		format_ident!("{}DynTable", self.ident)
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

	pub fn associated_type_for(&self, prop: TId<treeldr::Property>, collection: bool) -> Option<&AssociatedType> {
		self.associated_types.iter().find(|a| a.prop == prop && a.is_collection() == collection)
	}

	pub fn methods(&self) -> &[Method] {
		&self.methods
	}
}

/// Trait associated type.
pub struct AssociatedType {
	/// Property.
	prop: TId<treeldr::Property>,

	/// Associated type definition.
	ident: Ident,

	/// Label.
	label: Option<String>,

	/// Documentation.
	doc: treeldr::StrippedDocumentation,

	/// Trait bound.
	bound: AssociatedTypeBound,
}

impl AssociatedType {
	pub fn new(
		prop: TId<treeldr::Property>,
		ident: Ident,
		label: Option<String>,
		doc: treeldr::StrippedDocumentation,
		bound: AssociatedTypeBound,
	) -> Self {
		Self {
			prop,
			ident,
			label,
			doc,
			bound,
		}
	}

	pub fn build<V, M>(
		context: &Context<V, M>,
		associated_types: &mut Vec<AssociatedType>,
		name: &Name,
		p: treeldr::ty::properties::RestrictedProperty<M>,
	) -> Self {
		let ident = format_ident!("{}", name.to_pascal_case());
		let prop = context.model().get(p.property()).unwrap();
		let label = prop.preferred_label().map(Literal::to_string);
		let doc = prop.comment().clone_stripped();
		let assoc_ty = Self::new(
			p.property(),
			ident,
			label,
			doc,
			AssociatedTypeBound::Types(
				prop.as_property()
					.range()
					.iter()
					.map(|r| **r.value)
					.collect(),
			),
		);

		if prop.as_property().is_functional() {
			assoc_ty
		} else {
			let i = associated_types.len();
			associated_types.push(assoc_ty);

			let ident = format_ident!("{}s", name.to_pascal_case());
			Self::new(
				p.property(),
				ident,
				None,
				treeldr::StrippedDocumentation::default(),
				AssociatedTypeBound::Collection(i),
			)
		}
	}

	pub fn property(&self) -> TId<treeldr::Property> {
		self.prop
	}

	pub fn ident(&self) -> &Ident {
		&self.ident
	}

	/// Identifier of the trait object for this associated type.
	pub fn trait_object_ident(&self, tr: &Trait) -> Option<Ident> {
		if self.is_collection() {
			None
		} else {
			Some(format_ident!("Dyn{}{}", tr.ident(), &self.ident))
		}
	}

	pub fn trait_object_path<V, M>(&self, context: &Context<V, M>, tr: &Trait) -> Option<Path> {
		self.trait_object_ident(tr).map(|ident| {
			let mut path = context.parent_module_path(tr.module()).unwrap();
			path.push(ident);
			path
		})
	}

	pub fn label(&self) -> Option<&str> {
		self.label.as_deref()
	}

	pub fn documentation(&self) -> &treeldr::StrippedDocumentation {
		&self.doc
	}

	pub fn bound(&self) -> &AssociatedTypeBound {
		&self.bound
	}

	pub fn is_collection(&self) -> bool {
		self.bound.is_collection()
	}

	pub fn collection_item_type(&self) -> Option<usize> {
		self.bound.collection_item_type()
	}
}

/// Associated type bound.
pub enum AssociatedTypeBound {
	/// RDF type.
	Types(Vec<TId<treeldr::Type>>),

	/// Collection of the other associated type (given by its index).
	///
	/// The actual bound will be `Iterator<Item = &'a Self::T>`.
	Collection(usize),
}

impl AssociatedTypeBound {
	pub fn is_collection(&self) -> bool {
		matches!(self, Self::Collection(_))
	}

	pub fn collection_item_type(&self) -> Option<usize> {
		match self {
			Self::Collection(item_ty) => Some(*item_ty),
			Self::Types(_) => None
		}
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
		let assoc_ty = AssociatedType::build(context, associated_types, name, p);
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

	pub fn return_type_expr(&self, tr: &Trait) -> TokenStream {
		match &self.ty {
			MethodType::Required(i) => {
				let a_ident = tr.associated_types()[*i].ident();
				quote!(Self::#a_ident<'a>)
			}
			MethodType::Option(i) => {
				let a_ident = tr.associated_types()[*i].ident();
				quote!(Option<Self::#a_ident<'a>>)
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
