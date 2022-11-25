use crate::{module, path, Context, Path};
use derivative::Derivative;

pub use treeldr::layout::Primitive;
use treeldr::TId;

pub mod enumeration;
mod generate;
pub mod structure;

use enumeration::Enum;
use structure::Struct;

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

	pub fn path<V, M>(&self, context: &Context<V, M>, ident: proc_macro2::Ident) -> Option<Path> {
		let mut path = context.parent_module_path(self.module)?;
		path.push(path::Segment::Ident(ident));
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
}

pub enum Description {
	BuiltIn(BuiltIn),
	Never,
	Alias(proc_macro2::Ident, TId<treeldr::Layout>),
	Reference,
	Primitive(Primitive),
	Struct(Struct),
	Enum(Enum),
}

impl Description {
	pub fn impl_default<V, M>(&self, context: &Context<V, M>) -> bool {
		match self {
			Self::BuiltIn(b) => b.impl_default(),
			Self::Never => false,
			Self::Alias(_, other) => {
				let ty = context.layout_type(*other).unwrap();
				ty.impl_default(context)
			}
			Self::Reference => false,
			Self::Primitive(_) => false,
			Self::Struct(s) => s.impl_default(context),
			Self::Enum(_) => false,
		}
	}
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Copy(bound = ""))]
pub enum BuiltIn {
	/// Required type, erased.
	Required(TId<treeldr::Layout>),

	/// Option.
	Option(TId<treeldr::Layout>),

	/// Vec.
	Vec(TId<treeldr::Layout>),

	/// BTreeSet.
	BTreeSet(TId<treeldr::Layout>),

	/// OneOrMany, for non empty sets.
	OneOrMany(TId<treeldr::Layout>),
}

impl BuiltIn {
	pub fn impl_default(&self) -> bool {
		!matches!(self, Self::Required(_))
	}
}

impl Description {
	pub fn new<V, M>(context: &Context<V, M>, layout_ref: TId<treeldr::Layout>) -> Self {
		let layout = context
			.model()
			.get(layout_ref)
			.expect("undefined described layout");

		match layout.as_layout().description().value() {
			treeldr::layout::Description::Never => Self::Never,
			treeldr::layout::Description::Alias(alias_ref) => {
				let name = layout.as_component().name().expect("unnamed alias");
				let ident = generate::type_ident_of_name(name);
				Self::Alias(ident, *alias_ref)
			}
			treeldr::layout::Description::Primitive(p) => {
				if p.is_restricted() {
					todo!("restricted primitives")
				} else {
					Self::Primitive(p.primitive())
				}
			}
			treeldr::layout::Description::Reference(_) => Self::Reference,
			treeldr::layout::Description::Struct(s) => {
				let name = layout.as_component().name().expect("unnamed struct");
				let ident = generate::type_ident_of_name(name);
				let mut fields = Vec::with_capacity(s.fields().len());
				for field_id in s.fields() {
					let field = context.model().get(**field_id).unwrap();
					let field_name = field.as_component().name().expect("unnamed field");
					let ident = generate::field_ident_of_name(field_name);
					fields.push(structure::Field::new(
						field_name.value().clone(),
						ident,
						**field
							.as_formatted()
							.format()
							.as_ref()
							.expect("missing field layout"),
						field.as_layout_field().property().map(|m| **m),
						field.preferred_label().map(String::from),
						field.comment().clone_stripped(),
					))
				}

				Self::Struct(Struct::new(ident, fields))
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
						variant.as_formatted().format().as_ref().map(|m| **m),
					))
				}

				Self::Enum(Enum::new(ident, variants))
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
			treeldr::layout::Description::OneOrMany(s) => {
				Self::BuiltIn(BuiltIn::OneOrMany(**s.item_layout()))
			}
		}
	}
}
