use crate::{module, path, Context, Path};
use derivative::Derivative;
use shelves::Ref;

pub use treeldr::layout::Primitive;

pub mod enumeration;
mod generate;
pub mod structure;

use enumeration::Enum;
use structure::Struct;

pub struct Type<F> {
	module: Option<module::Parent<F>>,
	desc: Description<F>,
	label: Option<String>,
	doc: treeldr::Documentation,
}

impl<F> Type<F> {
	pub fn new(
		module: Option<module::Parent<F>>,
		desc: Description<F>,
		label: Option<String>,
		doc: treeldr::Documentation,
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

	pub fn documentation(&self) -> &treeldr::Documentation {
		&self.doc
	}

	pub fn path(&self, context: &Context<F>, ident: proc_macro2::Ident) -> Option<Path> {
		let mut path = context.parent_module_path(self.module)?;
		path.push(path::Segment::Ident(ident));
		Some(path)
	}

	pub fn impl_default(&self, context: &Context<F>) -> bool {
		self.desc.impl_default(context)
	}

	pub fn module(&self) -> Option<module::Parent<F>> {
		self.module
	}

	pub fn description(&self) -> &Description<F> {
		&self.desc
	}
}

pub enum Description<F> {
	BuiltIn(BuiltIn<F>),
	Never,
	Alias(proc_macro2::Ident, Ref<treeldr::layout::Definition<F>>),
	Reference,
	Primitive(Primitive),
	Struct(Struct<F>),
	Enum(Enum<F>),
}

impl<F> Description<F> {
	pub fn impl_default(&self, context: &Context<F>) -> bool {
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
pub enum BuiltIn<F> {
	/// Required type, erased.
	Required(Ref<treeldr::layout::Definition<F>>),

	/// Option.
	Option(Ref<treeldr::layout::Definition<F>>),

	/// Vec.
	Vec(Ref<treeldr::layout::Definition<F>>),

	/// BTreeSet.
	BTreeSet(Ref<treeldr::layout::Definition<F>>),

	/// OneOrMany, for non empty sets.
	OneOrMany(Ref<treeldr::layout::Definition<F>>),
}

impl<F> BuiltIn<F> {
	pub fn impl_default(&self) -> bool {
		!matches!(self, Self::Required(_))
	}
}

impl<F> Description<F> {
	pub fn new(context: &Context<F>, layout_ref: Ref<treeldr::layout::Definition<F>>) -> Self {
		let layout = context
			.model()
			.layouts()
			.get(layout_ref)
			.expect("undefined described layout");

		match layout.description() {
			treeldr::layout::Description::Never(_) => Self::Never,
			treeldr::layout::Description::Alias(name, alias_ref) => {
				let ident = generate::type_ident_of_name(name);
				Self::Alias(ident, *alias_ref)
			}
			treeldr::layout::Description::Primitive(p, _) => {
				if p.is_restricted() {
					todo!("restricted primitives")
				} else {
					Self::Primitive(p.primitive())
				}
			}
			treeldr::layout::Description::Reference(_) => Self::Reference,
			treeldr::layout::Description::Struct(s) => {
				let ident = generate::type_ident_of_name(s.name());
				let mut fields = Vec::with_capacity(s.fields().len());
				for field in s.fields() {
					let ident = generate::field_ident_of_name(field.name());
					fields.push(structure::Field::new(
						field.name().clone(),
						ident,
						field.layout(),
						field.property(),
						field.preferred_label(context.model()).map(String::from),
						field.preferred_documentation(context.model()).clone(),
					))
				}

				Self::Struct(Struct::new(ident, fields))
			}
			treeldr::layout::Description::Enum(e) => {
				let ident = generate::type_ident_of_name(e.name());
				let mut variants = Vec::with_capacity(e.variants().len());
				for variant in e.variants() {
					let ident = generate::variant_ident_of_name(variant.name());
					variants.push(enumeration::Variant::new(ident, variant.layout()))
				}

				Self::Enum(Enum::new(ident, variants))
			}
			treeldr::layout::Description::Required(r) => {
				Self::BuiltIn(BuiltIn::Required(r.item_layout()))
			}
			treeldr::layout::Description::Option(o) => {
				Self::BuiltIn(BuiltIn::Option(o.item_layout()))
			}
			treeldr::layout::Description::Array(a) => Self::BuiltIn(BuiltIn::Vec(a.item_layout())),
			treeldr::layout::Description::Set(s) => {
				Self::BuiltIn(BuiltIn::BTreeSet(s.item_layout()))
			}
		}
	}
}
