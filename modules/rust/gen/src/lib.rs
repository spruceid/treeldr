use derivative::Derivative;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use shelves::Shelf;
use std::collections::{HashMap, HashSet};
use std::fmt;
use thiserror::Error;

pub use shelves::Ref;

#[derive(Error)]
pub enum Error<F> {
	UnreachableType(Ref<treeldr::layout::Definition<F>>),
}

impl<F> Display<F> for Error<F> {
	fn fmt(&self, context: &Context<F>, f: &mut fmt::Formatter) -> fmt::Result {
		use treeldr::vocab::Display;
		match self {
			Self::UnreachableType(layout_ref) => {
				let layout = context.model().layouts().get(*layout_ref).unwrap();
				let id = layout.id();

				write!(f, "unbound layout `{}`", id.display(context.vocabulary()))
			}
		}
	}
}

pub trait Generate<F> {
	fn generate(
		&self,
		context: &Context<F>,
		scope: Option<Ref<Module<F>>>,
		tokens: &mut TokenStream,
	) -> Result<(), Error<F>>;

	fn with<'a, 'c>(
		&self,
		context: &'c Context<'a, F>,
		scope: Option<Ref<Module<F>>>,
	) -> With<'a, 'c, '_, F, Self> {
		With(context, scope, self)
	}
}

pub struct With<'a, 'c, 't, F, T: ?Sized>(&'c Context<'a, F>, Option<Ref<Module<F>>>, &'t T);

impl<'a, 'c, 't, F, T: ?Sized + Generate<F>> With<'a, 'c, 't, F, T> {
	pub fn into_tokens(self) -> Result<TokenStream, Error<F>> {
		let mut tokens = TokenStream::new();
		self.2.generate(self.0, self.1, &mut tokens)?;
		Ok(tokens)
	}
}

pub trait Display<F> {
	fn fmt(&self, context: &Context<F>, f: &mut fmt::Formatter) -> fmt::Result;

	fn display<'a, 'c>(&self, context: &'c Context<'a, F>) -> DisplayWith<'a, 'c, '_, F, Self> {
		DisplayWith(context, self)
	}
}

pub struct DisplayWith<'a, 'c, 't, F, T: ?Sized>(&'c Context<'a, F>, &'t T);

impl<'a, 'c, 't, F, T: ?Sized + Display<F>> fmt::Display for DisplayWith<'a, 'c, 't, F, T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.1.fmt(self.0, f)
	}
}

pub struct Referenced<T>(T);

#[derive(Default)]
pub struct Path(Vec<Segment>);

impl Path {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn len(&self) -> usize {
		self.0.len()
	}

	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	pub fn segments(&self) -> std::slice::Iter<Segment> {
		self.0.iter()
	}

	pub fn longest_common_prefix(&self, other: &Self) -> Self {
		self.segments()
			.zip(other.segments())
			.take_while(|(a, b)| a == b)
			.map(|(s, _)| s)
			.cloned()
			.collect()
	}

	pub fn to(&self, other: &Self) -> Self {
		let lcp = self.longest_common_prefix(other);
		let mut path = Path::new();

		for _ in lcp.len()..self.len() {
			path.push(Segment::Super)
		}

		for i in lcp.len()..other.len() {
			path.push(other.0[i].clone())
		}

		path
	}

	pub fn push(&mut self, segment: Segment) {
		self.0.push(segment)
	}
}

impl FromIterator<Segment> for Path {
	fn from_iter<I: IntoIterator<Item = Segment>>(iter: I) -> Self {
		Self(iter.into_iter().collect())
	}
}

impl ToTokens for Path {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		for (i, segment) in self.segments().enumerate() {
			if i > 0 {
				tokens.extend(quote! { :: })
			}

			segment.to_tokens(tokens)
		}
	}
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Copy(bound = ""))]
pub enum ParentModule<F> {
	/// The parent module is unreachable.
	Extern,
	Ref(Ref<Module<F>>),
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Segment {
	Super,
	Ident(proc_macro2::Ident),
}

impl quote::ToTokens for Segment {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		match self {
			Self::Super => tokens.extend(quote! { super }),
			Self::Ident(id) => id.to_tokens(tokens),
		}
	}
}

impl<F> Generate<F> for Ref<treeldr::layout::Definition<F>> {
	fn generate(
		&self,
		context: &Context<F>,
		scope: Option<Ref<Module<F>>>,
		tokens: &mut TokenStream,
	) -> Result<(), Error<F>> {
		let layout = context.layout(*self).expect("undefined generated layout");
		match &layout.desc {
			Description::Never => {
				tokens.extend(quote! { ! });
				Ok(())
			}
			Description::Primitive(p) => p.generate(context, scope, tokens),
			Description::Alias(ident, _) => {
				let path = layout
					.path(context, ident.clone())
					.ok_or(Error::UnreachableType(*self))?;
				context.module_path(scope).to(&path).to_tokens(tokens);
				Ok(())
			}
			Description::Struct(s) => {
				let path = layout
					.path(context, s.ident().clone())
					.ok_or(Error::UnreachableType(*self))?;
				context.module_path(scope).to(&path).to_tokens(tokens);
				Ok(())
			}
			Description::Enum(e) => {
				let path = layout
					.path(context, e.ident().clone())
					.ok_or(Error::UnreachableType(*self))?;
				context.module_path(scope).to(&path).to_tokens(tokens);
				Ok(())
			}
			Description::Reference => {
				tokens.extend(quote! { ::iref::IriBuf });
				Ok(())
			}
			Description::BuiltIn(b) => b.generate(context, scope, tokens),
		}
	}
}

impl<F> Generate<F> for Referenced<Ref<treeldr::layout::Definition<F>>> {
	fn generate(
		&self,
		context: &Context<F>,
		scope: Option<Ref<Module<F>>>,
		tokens: &mut TokenStream,
	) -> Result<(), Error<F>> {
		let layout = context.layout(self.0).expect("undefined generated layout");
		match &layout.desc {
			Description::Never => {
				tokens.extend(quote! { ! });
				Ok(())
			}
			Description::Primitive(p) => Referenced(*p).generate(context, scope, tokens),
			Description::Alias(ident, _) => {
				let abs_path = layout
					.path(context, ident.clone())
					.ok_or(Error::UnreachableType(self.0))?;
				let path = context.module_path(scope).to(&abs_path);
				tokens.extend(quote! { &#path });
				Ok(())
			}
			Description::Struct(s) => {
				let abs_path = layout
					.path(context, s.ident().clone())
					.ok_or(Error::UnreachableType(self.0))?;
				let path = context.module_path(scope).to(&abs_path);
				tokens.extend(quote! { &#path });
				Ok(())
			}
			Description::Enum(e) => {
				let abs_path = layout
					.path(context, e.ident().clone())
					.ok_or(Error::UnreachableType(self.0))?;
				let path = context.module_path(scope).to(&abs_path);
				tokens.extend(quote! { &#path });
				Ok(())
			}
			Description::Reference => {
				tokens.extend(quote! { ::iref::Iri });
				Ok(())
			}
			Description::BuiltIn(b) => Referenced(*b).generate(context, scope, tokens),
		}
	}
}

impl<F> Generate<F> for treeldr::layout::Primitive {
	fn generate(
		&self,
		_context: &Context<F>,
		_scope: Option<Ref<Module<F>>>,
		tokens: &mut TokenStream,
	) -> Result<(), Error<F>> {
		tokens.extend(match self {
			Self::Boolean => quote! { bool },
			Self::Integer => quote! { i32 },
			Self::UnsignedInteger => quote! { u32 },
			Self::Float => quote! { f32 },
			Self::Double => quote! { f64 },
			Self::String => quote! { ::std::string::String },
			Self::Date => quote! { ::chrono::Date<::chrono::Utc> },
			Self::DateTime => quote! { ::chrono::DateTime<::chrono::Utc> },
			Self::Time => quote! { ::chrono::NaiveTime },
			Self::Url => quote! { ::iref::IriBuf },
			Self::Uri => quote! { ::iref::IriBuf },
			Self::Iri => quote! { ::iref::IriBuf },
		});

		Ok(())
	}
}

impl<F> Generate<F> for Referenced<treeldr::layout::Primitive> {
	fn generate(
		&self,
		_context: &Context<F>,
		_scope: Option<Ref<Module<F>>>,
		tokens: &mut TokenStream,
	) -> Result<(), Error<F>> {
		use treeldr::layout::Primitive;
		tokens.extend(match self.0 {
			Primitive::Boolean => quote! { bool },
			Primitive::Integer => quote! { i32 },
			Primitive::UnsignedInteger => quote! { u32 },
			Primitive::Float => quote! { f32 },
			Primitive::Double => quote! { f64 },
			Primitive::String => quote! { &str },
			Primitive::Date => quote! { ::chrono::Date },
			Primitive::DateTime => quote! { ::chrono::DateTime },
			Primitive::Time => quote! { ::chrono::NaiveTime },
			Primitive::Url => quote! { ::iref::Iri },
			Primitive::Uri => quote! { ::iref::Iri },
			Primitive::Iri => quote! { ::iref::Iri },
		});

		Ok(())
	}
}

pub struct Layout<F> {
	module: Option<ParentModule<F>>,
	desc: Description<F>,
}

pub enum Description<F> {
	BuiltIn(BuiltIn<F>),
	Never,
	Alias(proc_macro2::Ident, Ref<treeldr::layout::Definition<F>>),
	Reference,
	Primitive(treeldr::layout::Primitive),
	Struct(Struct<F>),
	Enum(Enum<F>),
}

impl<F> Description<F> {
	pub fn impl_default(&self, context: &Context<F>) -> bool {
		match self {
			Self::BuiltIn(b) => b.impl_default(),
			Self::Never => false,
			Self::Alias(_, other) => {
				let layout = context.layout(*other).unwrap();
				layout.impl_default(context)
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
	Vec(Ref<treeldr::layout::Definition<F>>),
	BTreeSet(Ref<treeldr::layout::Definition<F>>),
}

impl<F> BuiltIn<F> {
	pub fn impl_default(&self) -> bool {
		true
	}
}

impl<F> Generate<F> for BuiltIn<F> {
	fn generate(
		&self,
		context: &Context<F>,
		scope: Option<Ref<Module<F>>>,
		tokens: &mut TokenStream,
	) -> Result<(), Error<F>> {
		tokens.extend(match self {
			Self::Vec(item) => {
				let item = item.with(context, scope).into_tokens()?;
				quote! { Vec<#item> }
			}
			Self::BTreeSet(item) => {
				let item = item.with(context, scope).into_tokens()?;
				quote! { std::collections::BTreeSet<#item> }
			}
		});

		Ok(())
	}
}

impl<F> Generate<F> for Referenced<BuiltIn<F>> {
	fn generate(
		&self,
		context: &Context<F>,
		scope: Option<Ref<Module<F>>>,
		tokens: &mut TokenStream,
	) -> Result<(), Error<F>> {
		tokens.extend(match self.0 {
			BuiltIn::Vec(item) => {
				let item = item.with(context, scope).into_tokens()?;
				quote! { &[#item] }
			}
			BuiltIn::BTreeSet(item) => {
				let item = item.with(context, scope).into_tokens()?;
				quote! { &std::collections::BTreeSet<#item> }
			}
		});

		Ok(())
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
				let ident = type_ident_of_name(name);
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
				let ident = type_ident_of_name(s.name());
				let mut fields = Vec::with_capacity(s.fields().len());
				for field in s.fields() {
					let ident = field_ident_of_name(field.name());
					fields.push(Field::new(
						ident,
						FieldType::new(field.layout(), field.is_required()),
					))
				}

				Self::Struct(Struct::new(ident, fields))
			}
			treeldr::layout::Description::Enum(e) => {
				let ident = type_ident_of_name(e.name());
				let mut variants = Vec::with_capacity(e.variants().len());
				for variant in e.variants() {
					let ident = variant_ident_of_name(variant.name());
					variants.push(Variant::new(ident, variant.layout()))
				}

				Self::Enum(Enum::new(ident, variants))
			}
			treeldr::layout::Description::Array(a) => Self::BuiltIn(BuiltIn::Vec(a.item_layout())),
			treeldr::layout::Description::Set(s) => {
				Self::BuiltIn(BuiltIn::BTreeSet(s.item_layout()))
			}
		}
	}
}

pub struct Struct<F> {
	ident: proc_macro2::Ident,
	fields: Vec<Field<F>>,
}

impl<F> Struct<F> {
	pub fn new(ident: proc_macro2::Ident, fields: Vec<Field<F>>) -> Self {
		Self { ident, fields }
	}

	pub fn ident(&self) -> &proc_macro2::Ident {
		&self.ident
	}

	pub fn fields(&self) -> &[Field<F>] {
		&self.fields
	}

	pub fn impl_default(&self, context: &Context<F>) -> bool {
		self.fields.iter().all(|f| f.ty.impl_default(context))
	}
}

#[derive(Derivative)]
#[derivative(Clone(bound = ""), Copy(bound = ""))]
pub struct FieldType<F> {
	ty: Ref<treeldr::layout::Definition<F>>,
	required: bool,
}

impl<F> FieldType<F> {
	pub fn new(ty: Ref<treeldr::layout::Definition<F>>, required: bool) -> Self {
		Self { ty, required }
	}

	pub fn ty(&self) -> Ref<treeldr::layout::Definition<F>> {
		self.ty
	}

	pub fn is_required(&self) -> bool {
		self.required
	}

	pub fn impl_default(&self, context: &Context<F>) -> bool {
		let ty = context.layout(self.ty).unwrap();
		!self.required || ty.impl_default(context)
	}
}

impl<F> Generate<F> for FieldType<F> {
	fn generate(
		&self,
		context: &Context<F>,
		scope: Option<Ref<Module<F>>>,
		tokens: &mut TokenStream,
	) -> Result<(), Error<F>> {
		let ty = self.ty.with(context, scope).into_tokens()?;

		if self.required {
			tokens.extend(ty)
		} else {
			tokens.extend(quote! { Option<#ty> })
		}

		Ok(())
	}
}

pub struct Field<F> {
	ident: proc_macro2::Ident,
	ty: FieldType<F>,
}

impl<F> Field<F> {
	pub fn new(ident: proc_macro2::Ident, ty: FieldType<F>) -> Self {
		Self { ident, ty }
	}

	pub fn ty(&self) -> FieldType<F> {
		self.ty
	}
}

impl<F> Generate<F> for Field<F> {
	fn generate(
		&self,
		context: &Context<F>,
		scope: Option<Ref<Module<F>>>,
		tokens: &mut TokenStream,
	) -> Result<(), Error<F>> {
		let ident = &self.ident;
		let ty = self.ty.with(context, scope).into_tokens()?;

		tokens.extend(quote! {
			pub #ident: #ty
		});

		Ok(())
	}
}

pub struct Enum<F> {
	ident: proc_macro2::Ident,
	variants: Vec<Variant<F>>,
}

impl<F> Enum<F> {
	pub fn new(ident: proc_macro2::Ident, variants: Vec<Variant<F>>) -> Self {
		Self { ident, variants }
	}

	pub fn ident(&self) -> &proc_macro2::Ident {
		&self.ident
	}

	pub fn variants(&self) -> &[Variant<F>] {
		&self.variants
	}
}

pub struct Variant<F> {
	ident: proc_macro2::Ident,
	ty: Option<Ref<treeldr::layout::Definition<F>>>,
}

impl<F> Variant<F> {
	pub fn new(ident: proc_macro2::Ident, ty: Option<Ref<treeldr::layout::Definition<F>>>) -> Self {
		Self { ident, ty }
	}
}

impl<F> Generate<F> for Variant<F> {
	fn generate(
		&self,
		context: &Context<F>,
		scope: Option<Ref<Module<F>>>,
		tokens: &mut TokenStream,
	) -> Result<(), Error<F>> {
		let ident = &self.ident;

		match self.ty.as_ref() {
			Some(ty) => {
				let ty = ty.with(context, scope).into_tokens()?;

				tokens.extend(quote! {
					#ident(#ty)
				})
			}
			None => tokens.extend(quote! { #ident }),
		}

		Ok(())
	}
}

impl<F> Layout<F> {
	pub fn new(module: Option<ParentModule<F>>, desc: Description<F>) -> Self {
		Self { module, desc }
	}

	pub fn path(&self, context: &Context<F>, ident: proc_macro2::Ident) -> Option<Path> {
		let mut path = context.parent_module_path(self.module)?;
		path.push(Segment::Ident(ident));
		Some(path)
	}

	pub fn impl_default(&self, context: &Context<F>) -> bool {
		self.desc.impl_default(context)
	}
}

impl<F> Generate<F> for Layout<F> {
	fn generate(
		&self,
		context: &Context<F>,
		scope: Option<Ref<Module<F>>>,
		tokens: &mut TokenStream,
	) -> Result<(), Error<F>> {
		match &self.desc {
			Description::Alias(ident, alias_ref) => {
				let alias = alias_ref.with(context, scope).into_tokens()?;

				tokens.extend(quote! {
					pub type #ident = #alias;
				})
			}
			Description::Struct(s) => {
				let ident = s.ident();
				let mut fields = Vec::with_capacity(s.fields().len());
				let mut required_inputs = Vec::new();
				let mut fields_init = Vec::new();
				let mut derives = vec![quote! { Clone }];

				for field in s.fields() {
					fields.push(field.with(context, scope).into_tokens()?);

					let field_ident = &field.ident;
					let init = if field.ty().impl_default(context) {
						quote! {
							Default::default()
						}
					} else {
						let ty = field.ty().with(context, scope).into_tokens()?;
						required_inputs.push(quote! {
							#field_ident : #ty,
						});

						quote! {
							#field_ident
						}
					};

					fields_init.extend(quote! { #field_ident : #init, })
				}

				if required_inputs.is_empty() {
					derives.push(quote! { Default });
				}

				tokens.extend(quote! {
					#[derive(#(#derives),*)]
					pub struct #ident {
						#(#fields),*
					}
				});

				if !required_inputs.is_empty() {
					tokens.extend(quote! {
						impl #ident {
							pub fn new(#(#required_inputs)*) -> Self {
								Self {
									#(#fields_init)*
								}
							}
						}
					})
				}
			}
			Description::Enum(e) => {
				let ident = e.ident();
				let mut variants = Vec::with_capacity(e.variants().len());

				for variant in e.variants() {
					variants.push(variant.with(context, scope).into_tokens()?)
				}

				tokens.extend(quote! {
					#[derive(Clone)]
					pub enum #ident {
						#(#variants),*
					}
				})
			}
			_ => (),
		}

		Ok(())
	}
}

pub struct Context<'a, F> {
	model: &'a treeldr::Model<F>,
	vocabulary: &'a treeldr::Vocabulary,
	modules: Shelf<Vec<Module<F>>>,
	layouts: shelves::Map<treeldr::layout::Definition<F>, HashMap<usize, Layout<F>>>,
}

impl<'a, F> Context<'a, F> {
	pub fn new(model: &'a treeldr::Model<F>, vocabulary: &'a treeldr::Vocabulary) -> Self {
		Self {
			model,
			vocabulary,
			modules: Shelf::default(),
			layouts: shelves::Map::default(),
		}
	}

	pub fn model(&self) -> &'a treeldr::Model<F> {
		self.model
	}

	pub fn vocabulary(&self) -> &'a treeldr::Vocabulary {
		self.vocabulary
	}

	pub fn module(&self, r: Ref<Module<F>>) -> Option<&Module<F>> {
		self.modules.get(r)
	}

	pub fn module_path(&self, r: Option<Ref<Module<F>>>) -> Path {
		match r {
			Some(module_ref) => self
				.module(module_ref)
				.expect("undefined module")
				.path(self),
			None => Path::new(),
		}
	}

	pub fn parent_module_path(&self, r: Option<ParentModule<F>>) -> Option<Path> {
		match r {
			Some(ParentModule::Extern) => None,
			Some(ParentModule::Ref(module_ref)) => Some(
				self.module(module_ref)
					.expect("undefined module")
					.path(self),
			),
			None => Some(Path::new()),
		}
	}

	pub fn layout(&self, r: Ref<treeldr::layout::Definition<F>>) -> Option<&Layout<F>> {
		self.layouts.get(r)
	}

	pub fn add_module(
		&mut self,
		parent: Option<Ref<Module<F>>>,
		ident: proc_macro2::Ident,
	) -> Ref<Module<F>> {
		let r = self.modules.insert(Module::new(parent, ident));
		if let Some(parent) = parent {
			self.modules
				.get_mut(parent)
				.expect("undefined parent module")
				.sub_modules
				.insert(r);
		}
		r
	}

	pub fn add_layout(
		&mut self,
		module: Option<ParentModule<F>>,
		layout_ref: Ref<treeldr::layout::Definition<F>>,
	) {
		self.layouts.insert(
			layout_ref,
			Layout::new(module, Description::new(self, layout_ref)),
		);
		if let Some(ParentModule::Ref(module)) = module {
			self.modules
				.get_mut(module)
				.expect("undefined module")
				.layouts
				.insert(layout_ref);
		}
	}
}

pub struct Module<F> {
	parent: Option<Ref<Self>>,
	ident: proc_macro2::Ident,
	sub_modules: HashSet<Ref<Self>>,
	layouts: HashSet<Ref<treeldr::layout::Definition<F>>>,
}

impl<F> Module<F> {
	pub fn new(parent: Option<Ref<Self>>, ident: proc_macro2::Ident) -> Self {
		Self {
			parent,
			ident,
			sub_modules: HashSet::new(),
			layouts: HashSet::new(),
		}
	}

	pub fn ident(&self) -> &proc_macro2::Ident {
		&self.ident
	}

	pub fn path(&self, context: &Context<F>) -> Path {
		let mut path = context.module_path(self.parent);
		path.push(Segment::Ident(self.ident.clone()));
		path
	}
}

impl<F> Generate<F> for Module<F> {
	fn generate(
		&self,
		context: &Context<F>,
		scope: Option<Ref<Module<F>>>,
		tokens: &mut TokenStream,
	) -> Result<(), Error<F>> {
		for module_ref in &self.sub_modules {
			module_ref.generate(context, scope, tokens)?;
		}

		for layout_ref in &self.layouts {
			let layout = context.layout(*layout_ref).expect("undefined layout");
			layout.generate(context, scope, tokens)?
		}

		Ok(())
	}
}

impl<F> Generate<F> for Ref<Module<F>> {
	fn generate(
		&self,
		context: &Context<F>,
		_scope: Option<Ref<Module<F>>>,
		tokens: &mut TokenStream,
	) -> Result<(), Error<F>> {
		let module = context.module(*self).expect("undefined module");
		let ident = module.ident();
		let content = module.with(context, Some(*self)).into_tokens()?;

		tokens.extend(quote! {
			pub mod #ident {
				#content
			}
		});

		Ok(())
	}
}
