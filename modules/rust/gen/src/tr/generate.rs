use std::collections::BTreeSet;

use contextual::WithContext;
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use rdf_types::Vocabulary;
use treeldr::{BlankIdIndex, IriIndex, TId};

use crate::{
	ty::params::{Parameters, ParametersBounds, ParametersValues},
	Context, Error, Generate, GenerateIn,
};

use super::{AssociatedType, AssociatedTypeBound, Method, MethodType, Trait};

fn collect_type_traits<V, M>(
	context: &Context<V, M>,
	ty_ref: TId<treeldr::Type>,
	mut f: impl FnMut(TId<treeldr::Type>) -> bool,
) {
	let mut stack = vec![ty_ref];
	while let Some(ty_ref) = stack.pop() {
		if f(ty_ref) {
			let ty = context.model().get(ty_ref).unwrap();
			if let Some(super_classes) = ty.as_type().sub_class_of() {
				stack.extend(super_classes.iter().map(|s| **s.value))
			}
		}
	}
}

fn associated_trait_object_type<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>, M>(
	context: &Context<V, M>,
	scope: Option<shelves::Ref<crate::Module>>,
	ident: Ident,
	classes: &[TId<treeldr::Type>],
) -> Result<TokenStream, Error> {
	let mut all_classes = BTreeSet::new();
	for &c in classes {
		collect_type_traits(context, c, |c| all_classes.insert(c))
	}

	let params_values = ParametersValues::new_for_trait(quote!(C));
	let mut trait_bounds = Vec::with_capacity(classes.len());
	for ty in classes {
		trait_bounds.push(
			ty.generate_in_with(context, scope, &params_values)
				.into_tokens()?,
		)
	}

	let mut tables_init = Vec::with_capacity(all_classes.len());
	let tables = all_classes.iter().enumerate().map(|(i, ty)| {
		let tr = context.type_trait(*ty).unwrap();
		let path = context
			.module_path(scope)
			.to(&tr.dyn_table_path(context).unwrap());
		let instance_path = context
			.module_path(scope)
			.to(&tr.dyn_table_instance_path(context).unwrap());
		let into_trait_object =
			quote!( ::treeldr_rust_prelude::AsTraitObject::<#path<C>>::into_trait_object(value) );
		if i == 0 {
			tables_init.push(quote!( {
				let (p, t) = #into_trait_object;
				ptr = p;
				t
			} ));
		} else {
			tables_init.push(quote!( #into_trait_object.1 ));
		}
		quote!(#instance_path <'d, C>)
	});

	let trait_impls = all_classes.iter().enumerate().map(|(i, ty)| {
		let index = syn::Index::from(i);
		let tr = context.type_trait(*ty).unwrap();
		let tr_path = context.module_path(scope).to(&tr.path(context).unwrap());

		let assoc_types = tr.associated_types().iter().map(|a| {
			let a_ident = a.ident();
			match a.trait_object_path(context, tr) {
				Some(path) => {
					let path = context.module_path(scope).to(&path);
					quote!(type #a_ident <'a> = #path <'a, C> where Self: 'a, C: 'a;)
				}
				None => {
					let item_a = &tr.associated_types()[a.collection_item_type().unwrap()];
					let item_path = context.module_path(scope).to(&item_a.trait_object_path(context, tr).unwrap());
					quote!(type #a_ident <'a> = ::treeldr_rust_prelude::BoxedDynIterator<'a, #item_path <'a, C>> where Self: 'a, C: 'a;)
				}
			}
		});

		let methods = tr.methods().iter().map(|m| {
			let m_ident = m.ident();
			let return_ty = m.return_type_expr(tr);

			quote! {
				fn #m_ident <'a> (&'a self, context: &'a C) -> #return_ty {
					unsafe { (self.tables.#index.#m_ident)(self.ptr, ::treeldr_rust_prelude::ContravariantReference::new(context)) }
				}
			}
		});

		let dyn_table_path = context.module_path(scope).to(&tr.dyn_table_path(context).unwrap());
		let dyn_table_instance_path = context.module_path(scope).to(&tr.dyn_table_instance_path(context).unwrap());

		quote! {
			impl <'d, C: ?Sized> #tr_path <C> for #ident <'d, C> {
				#(#assoc_types)*
				#(#methods)*
			}

			unsafe impl <'d, C: ?Sized> ::treeldr_rust_prelude::AsTraitObject<#dyn_table_path<C>> for #ident <'d, C> {
				fn as_trait_object(&self) -> (*const u8, #dyn_table_instance_path<C>) {
					(self.ptr, self.tables.#index)
				}
				fn into_trait_object<'r>(self) -> (*const u8, #dyn_table_instance_path<'r, C>) where Self: ::treeldr_rust_prelude::Reference<'r> {
					(self.ptr, self.tables.#index)
				}
			}
		}
	});

	Ok(quote! {
		pub struct #ident <'d, C: ?Sized> {
			_p: ::std::marker::PhantomData<&'d C>,
			ptr: *const u8,
			tables: (#(#tables,)*)
		}

		impl<'d, C: ?Sized> #ident <'d, C> {
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

		impl<'d, C: ?Sized> Clone for #ident <'d, C> {
			fn clone(&self) -> Self {
				*self
			}
		}

		impl<'d, C: ?Sized> Copy for #ident <'d, C> {}

		impl<'d, C: ?Sized> ::treeldr_rust_prelude::Reference<'d> for #ident <'d, C> {}

		#(#trait_impls)*
	})
}

impl Trait {
	pub fn generate_associated_type_expr(&self, ty_expr: TokenStream, index: usize) -> TokenStream {
		let ident = &self.associated_types[index].ident;
		quote!(#ty_expr :: #ident)
	}
}

impl<M> Generate<M> for Trait {
	fn generate<
		V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>,
	>(
		&self,
		context: &crate::Context<V, M>,
		scope: Option<shelves::Ref<crate::Module>>,
		tokens: &mut proc_macro2::TokenStream,
	) -> Result<(), crate::Error> {
		let ident = &self.ident;
		let params_values = ParametersValues::new_for_trait(quote!(C));
		let params_bounds = ParametersBounds::new_for_trait(quote!(?Sized));
		let params = Parameters::context_parameter()
			.instantiate(&params_values)
			.with_bounds(&params_bounds);

		let mut super_traits = TokenStream::new();
		for &ty_ref in &self.super_traits {
			super_traits.extend(quote!(+));
			super_traits.extend(
				ty_ref
					.with(self)
					.generate_in_with(context, scope, &params_values)
					.into_tokens()?,
			)
		}

		let mut associated_types = Vec::with_capacity(self.associated_types.len());
		let mut never_associated_types = Vec::with_capacity(self.associated_types.len());
		let mut ref_associated_types = Vec::with_capacity(self.associated_types.len());
		let mut associated_types_trait_objects = Vec::with_capacity(self.associated_types.len());
		for ty in &self.associated_types {
			associated_types.push(
				ty.with(self)
					.generate_in_with(context, scope, &params_values)
					.into_tokens()?,
			);

			let a_ident = ty.ident();
			let never_expr = match ty.bound() {
				AssociatedTypeBound::Types(classes) => {
					associated_types_trait_objects.push(associated_trait_object_type(
						context,
						scope,
						ty.trait_object_ident(self).unwrap(),
						classes,
					)?);

					quote!(&'a ::std::convert::Infallible)
				}
				AssociatedTypeBound::Collection(_) => {
					quote!(::std::iter::Empty<&'a ::std::convert::Infallible>)
				}
			};
			never_associated_types.push(quote! {
				type #a_ident <'a> = #never_expr where Self: 'a, C: 'a;
			});
			ref_associated_types.push(quote! {
				type #a_ident <'a> = T::#a_ident<'a> where Self: 'a, C: 'a;
			})
		}

		let mut methods = Vec::with_capacity(self.methods.len());
		let mut never_methods = Vec::with_capacity(self.methods.len());
		let mut ref_methods = Vec::with_capacity(self.methods.len());
		let mut table_fields = Vec::with_capacity(self.methods.len());
		let mut table_fields_init = Vec::with_capacity(self.methods.len());
		for m in &self.methods {
			methods.push(m.with(self).generate_with(context, scope).into_tokens()?);

			let m_ident = m.ident();
			let (return_ty, table_field_ty, wrap) = match m.type_() {
				MethodType::Required(i) => {
					let a = &self.associated_types[*i];
					let ty_ident = a.ident();
					let (table_ty, table_wrap) = match a.trait_object_path(context, self) {
						Some(path) => {
							let path = context.module_path(scope).to(&path);
							(quote!(#path <'a, C>), quote!(#path::new(object)))
						}
						None => {
							let item_a = &self.associated_types[a.collection_item_type().unwrap()];
							let item_path = context
								.module_path(scope)
								.to(&item_a.trait_object_path(context, self).unwrap());
							(
								quote!(::treeldr_rust_prelude::BoxedDynIterator<#item_path <'a, C>>),
								quote!(::treeldr_rust_prelude::BoxedDynIterator::new(object.map(#item_path::new))),
							)
						}
					};
					(quote!(Self::#ty_ident<'a>), table_ty, table_wrap)
				}
				MethodType::Option(i) => {
					let a = &self.associated_types[*i];
					let ty_ident = a.ident();
					let path = context
						.module_path(scope)
						.to(&a.trait_object_path(context, self).unwrap());
					(
						quote!(Option<Self::#ty_ident<'a>>),
						quote!(Option<#path <'a, C>>),
						quote!(object.map(#path::new)),
					)
				}
			};
			never_methods.push(quote! {
				fn #m_ident <'a> (&'a self, _context: &'a C) -> #return_ty {
					unreachable!()
				}
			});
			ref_methods.push(quote! {
				fn #m_ident <'a> (&'a self, context: &'a C) -> #return_ty {
					T::#m_ident(*self, context)
				}
			});

			table_fields.push(quote! {
				pub #m_ident: unsafe fn (*const u8, context: ::treeldr_rust_prelude::ContravariantReference<'a, C>) -> #table_field_ty
			});
			table_fields_init.push(quote! {
				#m_ident: |ptr, context| unsafe {
					let subject = &*(ptr as *const T);
					let object = context.get(|context| subject.#m_ident(context));
					#wrap
				}
			})
		}

		let context_ident = self.context_ident();
		let dyn_table_ident = self.dyn_table_ident();
		let dyn_table_instance_ident = self.dyn_table_instance_ident();

		if table_fields.is_empty() {
			table_fields.push(quote! {
				_d: ::std::marker::PhantomData<&'a C>
			});

			table_fields_init.push(quote! {
				_d: ::std::marker::PhantomData
			})
		}

		tokens.extend(quote! {
			pub trait #ident #params: ::treeldr_rust_prelude::AsTraitObject<#dyn_table_ident<C>> #super_traits {
				#(#associated_types)*

				#(#methods)*
			}

			pub trait #context_ident <I: ?Sized>: ::treeldr_rust_prelude::Provider<I, Self::#ident> {
				type #ident: #ident <Self>;

				fn get(&self, id: &I) -> Option<&Self::#ident> {
					<Self as ::treeldr_rust_prelude::Provider<I, Self::#ident>>::get(self, id)
				}
			}

			impl <C: ?Sized> #ident <C> for ::std::convert::Infallible {
				#(#never_associated_types)*
				#(#never_methods)*
			}

			impl<'r, C: ?Sized, T: #ident<C>> #ident <C> for &'r T {
				#(#ref_associated_types)*
				#(#ref_methods)*
			}

			pub struct #dyn_table_ident <C: ?Sized>(std::marker::PhantomData<C>);

			impl<C: ?Sized> ::treeldr_rust_prelude::Table for #dyn_table_ident <C> {
				type Instance<'a> = #dyn_table_instance_ident <'a, C> where Self: 'a;
			}

			pub struct #dyn_table_instance_ident <'a, C: ?Sized> {
				#(#table_fields,)*
			}

			impl<'a, C: ?Sized> Clone for #dyn_table_instance_ident <'a, C> {
				fn clone(&self) -> Self {
					*self
				}
			}

			impl<'a, C: ?Sized> Copy for #dyn_table_instance_ident <'a, C> {}

			impl<'a, C: ?Sized> #dyn_table_instance_ident <'a, C> {
				pub fn new<T: 'a + #ident<C>>() -> Self {
					Self {
						#(#table_fields_init,)*
					}
				}
			}

			#(#associated_types_trait_objects)*
		});

		Ok(())
	}
}

impl<'a, 't, M> Generate<M> for contextual::Contextual<&'a Method, &'t Trait> {
	fn generate<
		V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>,
	>(
		&self,
		context: &crate::Context<V, M>,
		scope: Option<shelves::Ref<crate::Module>>,
		tokens: &mut proc_macro2::TokenStream,
	) -> Result<(), crate::Error> {
		let ident = &self.ident;
		let ty = self
			.ty
			.with(self.1)
			.generate_with(context, scope)
			.into_tokens()?;

		tokens.extend(quote! {
			fn #ident <'a> (&'a self, context: &'a C) -> #ty;
		});

		Ok(())
	}
}

impl<'a, 't, M> Generate<M> for contextual::Contextual<&'a MethodType, &'t Trait> {
	fn generate<
		V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>,
	>(
		&self,
		_context: &crate::Context<V, M>,
		_scope: Option<shelves::Ref<crate::Module>>,
		tokens: &mut proc_macro2::TokenStream,
	) -> Result<(), crate::Error> {
		match self.0 {
			MethodType::Required(i) => {
				let ty_expr = self.1.generate_associated_type_expr(quote!(Self), *i);
				tokens.extend(ty_expr);
				tokens.extend(quote!(<'a>));
			}
			MethodType::Option(i) => {
				let ty_expr = self.1.generate_associated_type_expr(quote!(Self), *i);
				tokens.extend(quote!(Option<#ty_expr<'a>>))
			}
		}

		Ok(())
	}
}

impl<'a, 't, M> GenerateIn<M> for contextual::Contextual<&'a AssociatedType, &'t Trait> {
	fn generate_in<
		V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>,
	>(
		&self,
		context: &crate::Context<V, M>,
		scope: Option<shelves::Ref<crate::Module>>,
		params_values: &ParametersValues,
		tokens: &mut proc_macro2::TokenStream,
	) -> Result<(), crate::Error> {
		let ident = &self.ident;
		let bound = self
			.bound
			.with(self.1)
			.generate_in_with(context, scope, params_values)
			.into_tokens()?;

		tokens.extend(quote! {
			type #ident <'a> : #bound;
		});

		Ok(())
	}
}

impl<'a, 't, M> GenerateIn<M> for contextual::Contextual<&'a AssociatedTypeBound, &'t Trait> {
	fn generate_in<
		V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>,
	>(
		&self,
		context: &crate::Context<V, M>,
		scope: Option<shelves::Ref<crate::Module>>,
		params_value: &ParametersValues,
		tokens: &mut proc_macro2::TokenStream,
	) -> Result<(), crate::Error> {
		match self.0 {
			AssociatedTypeBound::Types(refs) => {
				tokens.extend(quote!(::treeldr_rust_prelude::Reference<'a>));

				for type_ref in refs {
					tokens.extend(quote!(+));
					type_ref.generate_in(context, scope, params_value, tokens)?;
				}

				tokens.extend(quote!(where Self: 'a, C: 'a));

				Ok(())
			}
			AssociatedTypeBound::Collection(i) => {
				let ty_expr = self.1.generate_associated_type_expr(quote!(Self), *i);
				tokens.extend(quote!('a + Iterator<Item = #ty_expr<'a>> where Self: 'a, C: 'a));
				Ok(())
			}
		}
	}
}

impl<M> GenerateIn<M> for TId<treeldr::Type> {
	fn generate_in<
		V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>,
	>(
		&self,
		context: &crate::Context<V, M>,
		scope: Option<shelves::Ref<crate::Module>>,
		params_values: &ParametersValues,
		tokens: &mut TokenStream,
	) -> Result<(), crate::Error> {
		let tr = context.type_trait(*self).expect("trait not found");
		let path = tr.path(context).ok_or(Error::UnreachableTrait(*self))?;
		context.module_path(scope).to(&path).to_tokens(tokens);
		Parameters::context_parameter()
			.instantiate(params_values)
			.to_tokens(tokens);
		Ok(())
	}
}
