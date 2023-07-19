use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};

pub mod primitive;

pub enum TypeDefinition {
	ClassTraitObject(ClassDynTraitDefinition),
	Layout(LayoutTypeDefinition, Vec<String>),
}

impl ToTokens for TypeDefinition {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		match self {
			Self::ClassTraitObject(d) => d.to_tokens(tokens),
			Self::Layout(l, doc) => {
				for doc in doc {
					tokens.extend(quote!(#[doc = #doc]))
				}

				l.to_tokens(tokens)
			}
		}
	}
}

/// Layout type definition.
pub enum LayoutTypeDefinition {
	Alias(Alias),
	RestrictedPrimitive(primitive::Derived),
	Struct(Struct),
	Enum(Enum),
}

impl ToTokens for LayoutTypeDefinition {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		match self {
			Self::Alias(a) => a.to_tokens(tokens),
			Self::RestrictedPrimitive(r) => r.to_tokens(tokens),
			Self::Struct(s) => s.to_tokens(tokens),
			Self::Enum(e) => e.to_tokens(tokens),
		}
	}
}

macro_rules! layout_params_type {
	{ $( $name:ident ),* } => {
		pub struct LayoutParameters {
			$( pub $name : Option<Ident> ),*
		}

		impl LayoutParameters {
			pub fn is_empty(&self) -> bool {
				$( self.$name.is_none() )&&*
			}

			pub fn iter(&self) -> LayoutParametersIter {
				LayoutParametersIter {
					$( $name: self.$name.as_ref() ),*
				}
			}
		}

		pub struct LayoutParametersIter<'a> {
			$( $name : Option<&'a Ident> ),*
		}

		impl<'a> Iterator for LayoutParametersIter<'a> {
			type Item = &'a Ident;

			fn next(&mut self) -> Option<Self::Item> {
				$(
					if let Some(ty) = self.$name.take() {
						return Some(ty)
					}
				)*

				None
			}
		}
	};
}

layout_params_type! {
	identifier
}

impl ToTokens for LayoutParameters {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		if !self.is_empty() {
			let types = self.iter();
			tokens.extend(quote!(< #(#types),* >))
		}
	}
}

pub struct Alias {
	pub ident: Ident,
	pub params: LayoutParameters,
	pub target: syn::Type,
}

impl ToTokens for Alias {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let ident = &self.ident;
		let params = &self.params;
		let target = &self.target;

		tokens.extend(quote!(pub type #ident #params = #target #params ;))
	}
}

macro_rules! derives_type {
	{ $( $name:ident : $variant:ident ),* } => {
		#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
		pub struct Derives {
			$( pub $name : bool ),*
		}

		impl IntoIterator for Derives {
			type Item = Derive;
			type IntoIter = DerivesIter;

			fn into_iter(self) -> Self::IntoIter {
				DerivesIter {
					$($name: self.$name),*
				}
			}
		}

		#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
		pub enum Derive {
			$( $variant ),*
		}

		impl Derives {
			pub fn is_empty(&self) -> bool {
				$( !self.$name )&&*
			}
		}

		#[derive(Debug, Clone)]
		pub struct DerivesIter {
			$( pub $name : bool ),*
		}

		impl Iterator for DerivesIter {
			type Item = Derive;

			fn next(&mut self) -> Option<Self::Item> {
				$(
					if self.$name {
						self.$name = false;
						return Some(Derive::$variant)
					}
				)*

				None
			}
		}

		impl ToTokens for Derive {
			fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
				match self {
					$(Self::$variant => {
						tokens.extend(quote!($variant))
					}),*
				}
			}
		}
	};
}

derives_type! {
	clone: Clone,
	partial_eq: PartialEq,
	eq: Eq,
	partial_ord: PartialOrd,
	ord: Ord,
	debug: Debug,
	default: Default
}

impl ToTokens for Derives {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		if !self.is_empty() {
			let derives = self.into_iter();
			tokens.extend(quote!(#[derive(#(#derives),*)]))
		}
	}
}

pub struct Struct {
	pub derives: Derives,
	pub ident: Ident,
	pub params: LayoutParameters,
	pub fields: Vec<Field>,

	/// Inputs of the `new` constructor.
	pub constructor_inputs: Vec<(Ident, syn::Type)>,
}

impl ToTokens for Struct {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let derives = &self.derives;
		let ident = &self.ident;
		let params = &self.params;
		let fields = &self.fields;

		tokens.extend(quote! {
			#derives
			pub struct #ident #params {
				#(#fields),*
			}
		});

		let inputs = self
			.constructor_inputs
			.iter()
			.map(|(id, ty)| quote!(#id : #ty));
		let fields_init = self.fields.iter().map(|f| {
			let f_ident = &f.ident;
			let value = &f.initial_value;
			quote!(#f_ident: #value)
		});

		tokens.extend(quote! {
			impl #params #ident #params {
				pub fn new(#(#inputs),*) -> Self {
					Self {
						#(#fields_init),*
					}
				}
			}
		});

		if !derives.default && self.constructor_inputs.is_empty() {
			tokens.extend(quote! {
				impl #params Default for #ident #params {
					fn default() -> Self {
						Self::new()
					}
				}
			})
		}
	}
}

/// Field definition.
pub struct Field {
	pub ident: Ident,
	pub type_: syn::Type,

	/// Expression used to initialize the field in the `new` constructor.
	pub initial_value: syn::Expr,
}

impl ToTokens for Field {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let ident = &self.ident;
		let ty = &self.type_;

		tokens.extend(quote!(pub #ident : #ty))
	}
}

pub struct Enum {
	pub derives: Derives,
	pub ident: Ident,
	pub params: LayoutParameters,
	pub variants: Vec<Variant>,
}

impl ToTokens for Enum {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let derives = &self.derives;
		let ident = &self.ident;
		let params = &self.params;
		let variants = &self.variants;

		tokens.extend(quote! {
			#derives
			pub enum #ident #params {
				#(#variants),*
			}
		})
	}
}

/// Variant definition.
pub struct Variant {
	pub ident: Ident,
	pub type_: Option<syn::Type>,
}

impl ToTokens for Variant {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let ident = &self.ident;

		match self.type_.as_ref() {
			Some(ty) => tokens.extend(quote!(#ident ( #ty ))),
			None => tokens.extend(quote!(#ident)),
		}
	}
}

pub struct ClassDynTraitDefinition {
	pub table: ClassDynTable,
	pub associated_types_trait_objects: Vec<ClassAssociatedTypeTraitObject>,
}

impl ToTokens for ClassDynTraitDefinition {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		self.table.to_tokens(tokens);
		for t in &self.associated_types_trait_objects {
			t.to_tokens(tokens)
		}
	}
}

pub struct ClassDynTable {
	pub trait_path: syn::Path,
	pub ident: Ident,
	pub instance_ident: Ident,
	pub fields: Vec<ClassDynTableField>,
}

impl ToTokens for ClassDynTable {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let trait_path = &self.trait_path;
		let ident = &self.ident;
		let instance_ident = &self.instance_ident;
		let fields = &self.fields;
		let fields_init = self.fields.iter().map(|f| {
			let f_ident = &f.ident;
			let value = &f.initial_value;
			quote!(#f_ident: #value)
		});

		let (fields, fields_init) = if fields.is_empty() {
			(
				quote!(_d: ::std::marker::PhantomData<&'r C>),
				quote!(_d: ::std::marker::PhantomData),
			)
		} else {
			(quote!(#(#fields),*), quote!(#(#fields_init,)*))
		};

		tokens.extend(quote! {
			pub struct #ident <I, C: ?Sized>(std::marker::PhantomData<(I, C)>);

			impl<I, C: ?Sized> ::treeldr_rust_prelude::Table for #ident <I, C> {
				type Instance<'r> = #instance_ident <'r, I, C> where Self: 'r;
			}

			pub struct #instance_ident <'r, I, C: ?Sized> {
				#fields
			}

			impl<'r, I, C: ?Sized> Clone for #instance_ident <'r, I, C> {
				fn clone(&self) -> Self {
					*self
				}
			}

			impl<'r, I, C: ?Sized> Copy for #instance_ident <'r, I, C> {}

			impl<'r, I, C: ?Sized> #instance_ident <'r, I, C> {
				pub fn new<T: 'r + #trait_path>() -> Self {
					Self {
						#fields_init
					}
				}
			}
		})
	}
}

pub struct ClassDynTableField {
	pub ident: Ident,
	pub ty: syn::Type,
	pub initial_value: syn::Expr,
}

impl ToTokens for ClassDynTableField {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let ident = &self.ident;
		let ty = &self.ty;

		tokens.extend(quote! {
			pub #ident: unsafe fn (*const u8, context: ::treeldr_rust_prelude::ContravariantReference<'r, C>) -> #ty
		})
	}
}

pub struct ClassAssociatedTypeTraitObject {
	pub ident: Ident,
	pub tables: Vec<ClassAssociatedTypeTraitObjectTable>,
	pub trait_bounds: Vec<syn::TraitBound>,
	pub trait_impls: Vec<ClassAssociatedTypeTraitObjectTraitImpl>,
}

impl ToTokens for ClassAssociatedTypeTraitObject {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let ident = &self.ident;
		let tables = &self.tables;
		let trait_bounds = &self.trait_bounds;
		let trait_impls = &self.trait_impls;

		let fields = if tables.is_empty() {
			quote!(_p: ::std::marker::PhantomData<&'d (I, C)>)
		} else {
			quote! {
				_p: ::std::marker::PhantomData<&'d (I, C)>,
				ptr: *const u8,
				tables: (#(#tables,)*)
			}
		};

		let constructor = if fields.is_empty() {
			quote! {
				pub fn new<T: #(#trait_bounds+)* ::treeldr_rust_prelude::Reference<'d>>(_value: T) -> Self {
					Self {
						_p: ::std::marker::PhantomData
					}
				}
			}
		} else {
			let tables_init = self.tables.iter().map(|t| &t.initial_value);

			quote! {
				pub fn new<T: #(#trait_bounds+)* ::treeldr_rust_prelude::Reference<'d>>(value: T) -> Self {
					let ptr;
					let tables = (#(#tables_init,)*);

					Self {
						_p: ::std::marker::PhantomData,
						ptr,
						tables
					}
				}
			}
		};

		tokens.extend(quote! {
			pub struct #ident <'d, I> {
				#fields
			}

			impl<'d, I> #ident <'d, I> {
				#constructor
			}

			impl<'d, I> Clone for #ident <'d, I> {
				fn clone(&self) -> Self {
					*self
				}
			}

			impl<'d, I, C: ?Sized> Copy for #ident <'d, I, C> {}

			impl<'d, I, C: ?Sized> ::treeldr_rust_prelude::Reference<'d> for #ident <'d, I, C> {}

			#(#trait_impls)*
		})
	}
}

pub struct ClassAssociatedTypeTraitObjectTable {
	pub ty: syn::Type,
	pub initial_value: TokenStream,
}

impl ToTokens for ClassAssociatedTypeTraitObjectTable {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		self.ty.to_tokens(tokens)
	}
}

pub struct ClassAssociatedTypeTraitObjectTraitImpl {
	pub ident: Ident,
	pub trait_path: syn::Path,
	pub table_path: syn::Path,
	pub table_instance_path: syn::Path,
	pub table_index: usize,
	pub associated_types: Vec<(Ident, syn::Type)>,
	pub methods: Vec<ClassAssociatedTypeTraitObjectTraitImplMethod>,
}

impl ToTokens for ClassAssociatedTypeTraitObjectTraitImpl {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let ident = &self.ident;
		let tr_path = &self.trait_path;

		let assoc_types = self
			.associated_types
			.iter()
			.map(|(id, ty)| quote!(type #id <'r> = #ty where Self: 'r;));

		let methods = &self.methods;
		let table_path = &self.table_path;
		let table_instance_path = &self.table_instance_path;
		let index = syn::Index::from(self.table_index);

		tokens.extend(quote! {
			impl <'d, I> #tr_path for #ident <'d, I> {
				#(#assoc_types)*
				#(#methods)*
			}

			unsafe impl <'d, I> ::treeldr_rust_prelude::AsTraitObject<I, #table_path> for #ident <'d, I> {
				fn as_trait_object<'r>(&'r self) -> (*const u8, #table_instance_path) {
					(self.ptr, self.tables.#index)
				}
				fn into_trait_object<'r>(self) -> (*const u8, #table_instance_path) where Self: ::treeldr_rust_prelude::Reference<'r> {
					(self.ptr, self.tables.#index)
				}
			}
		})
	}
}

pub struct ClassAssociatedTypeTraitObjectTraitImplMethod {
	pub ident: Ident,
	pub return_ty: syn::Type,
	pub table_index: usize,
}

impl ToTokens for ClassAssociatedTypeTraitObjectTraitImplMethod {
	fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
		let ident = &self.ident;
		let return_ty = &self.return_ty;
		let index = syn::Index::from(self.table_index);

		tokens.extend(quote! {
			fn #ident <'r> (&'r self) -> #return_ty {
				unsafe { (self.tables.#index.#ident)(self.ptr) }
			}
		})
	}
}
