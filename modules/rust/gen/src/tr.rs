use proc_macro2::Ident;
use quote::format_ident;
use rdf_types::Vocabulary;
use treeldr::{ty::PseudoProperty, value::Literal, Id, IriIndex, Name, TId};

use crate::{module, path, Context, Path};

mod generate;

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

	pub fn module(&self) -> Option<module::Parent> {
		self.module
	}

	pub fn ident(&self) -> &proc_macro2::Ident {
		&self.ident
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

	pub fn methods(&self) -> &[Method] {
		&self.methods
	}
}

/// Trait associated type.
pub struct AssociatedType {
	/// Associated type definition.
	ident: Ident,

	/// Label.
	label: Option<String>,

	/// Documentation.
	doc: treeldr::StrippedDocumentation,

	/// Trait bound.
	bound: AssociatedTypeBound,

	has_lifetime: bool,
}

impl AssociatedType {
	pub fn new(
		ident: Ident,
		label: Option<String>,
		doc: treeldr::StrippedDocumentation,
		bound: AssociatedTypeBound,
		has_lifetime: bool,
	) -> Self {
		Self {
			ident,
			label,
			doc,
			bound,
			has_lifetime,
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
			false,
		);

		if prop.as_property().is_functional() {
			assoc_ty
		} else {
			let i = associated_types.len();
			associated_types.push(assoc_ty);

			let ident = format_ident!("{}s", name.to_pascal_case());
			Self::new(
				ident,
				None,
				treeldr::StrippedDocumentation::default(),
				AssociatedTypeBound::Collection(i),
				true,
			)
		}
	}

	pub fn label(&self) -> Option<&str> {
		self.label.as_deref()
	}

	pub fn documentation(&self) -> &treeldr::StrippedDocumentation {
		&self.doc
	}

	pub fn has_lifetime(&self) -> bool {
		self.has_lifetime
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

/// Trait method.
pub struct Method {
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
		ident: Ident,
		label: Option<String>,
		doc: treeldr::StrippedDocumentation,
		ty: MethodType,
	) -> Self {
		Self {
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
		let has_lifetime = assoc_ty.has_lifetime();

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
				MethodType::Reference(i)
			} else {
				MethodType::Option(i)
			}
		} else {
			MethodType::Direct(i, has_lifetime)
		};

		Self::new(ident, label, doc, ty)
	}

	pub fn label(&self) -> Option<&str> {
		self.label.as_deref()
	}

	pub fn documentation(&self) -> &treeldr::StrippedDocumentation {
		&self.doc
	}
}

/// Method type.
pub enum MethodType {
	/// Direct associated type given by its index.
	///
	/// If the boolean is `true` a lifetime is added to the type:
	/// `Self::T<'_>`, otherwise `Self::T`.
	Direct(usize, bool),

	/// Referenced associated type given by its index.
	///
	/// `&Self::T`.
	Reference(usize),

	/// Optional referenced type given by its index.
	///
	/// `Option<&Self::T>`.
	Option(usize),
}
