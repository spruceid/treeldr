use std::collections::BTreeSet;

use proc_macro2::Ident;
use quote::quote;
use rdf_types::Vocabulary;
use treeldr::{BlankIdIndex, IriIndex, TId};

use crate::{
	syntax::{self, ClassAssociatedTypeTraitObject},
	Context, Error, GenerateSyntax, Scope,
};

use super::{AssociatedTypeBound, MethodType};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DynTableOf(pub TId<treeldr::Type>);

impl<M> GenerateSyntax<M> for DynTableOf {
	type Output = syntax::ClassDynTable;

	fn generate_syntax<
		V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>,
	>(
		&self,
		context: &crate::Context<V, M>,
		scope: &crate::Scope,
	) -> Result<Self::Output, crate::Error> {
		let mut scope = scope.clone();
		scope.params.context = Some(syn::parse2(quote!(C)).unwrap());
		scope.params.lifetime = Some(syn::Lifetime::new("'a", proc_macro2::Span::call_site()));

		let tr = context.type_trait(self.0).unwrap();
		let mut fields = Vec::with_capacity(tr.methods.len());
		for m in &tr.methods {
			let m_ident = m.ident();

			let ty;
			let wrap;

			match m.type_() {
				MethodType::Required(i) => {
					let a = &tr.associated_types[*i];
					match a.trait_object_path(context, tr) {
						Some(path) => {
							let path = context
								.module_path(scope.module)
								.to(&path)
								.generate_syntax(context, &scope)?;
							ty = syn::parse2(quote!(#path)).unwrap();
							wrap = quote!(#path::new(object));
						}
						None => {
							let item_a = &tr.associated_types[a.collection_item_type().unwrap()];
							let item_path = context
								.module_path(scope.module)
								.to(&item_a.trait_object_path(context, tr).unwrap())
								.generate_syntax(context, &scope)?;

							ty = syn::parse2(
								quote!(::treeldr_rust_prelude::BoxedDynIterator<#item_path>),
							)
							.unwrap();
							wrap = quote!(::treeldr_rust_prelude::BoxedDynIterator::new(object.map(#item_path::new)));
						}
					}
				}
				MethodType::Option(i) => {
					let a = &tr.associated_types[*i];
					let path = context
						.module_path(scope.module)
						.to(&a.trait_object_path(context, tr).unwrap())
						.generate_syntax(context, &scope)?;

					ty = syn::parse2(quote!(Option<#path>)).unwrap();
					wrap = quote!(object.map(#path::new));
				}
			};

			fields.push(syntax::ClassDynTableField {
				ident: m_ident.clone(),
				ty,
				initial_value: syn::parse2(quote! {
					|ptr, context| unsafe {
						let subject = &*(ptr as *const T);
						let object = context.get(|context| subject.#m_ident(context));
						#wrap
					}
				})
				.unwrap(),
			})
		}

		Ok(syntax::ClassDynTable {
			trait_path: self.0.generate_syntax(context, &scope)?,
			ident: tr.dyn_table_ident(),
			instance_ident: tr.dyn_table_instance_ident(),
			fields,
		})
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TraitObjectsOf(pub TId<treeldr::Type>);

impl<M> GenerateSyntax<M> for TraitObjectsOf {
	type Output = syntax::ClassDynTraitDefinition;

	fn generate_syntax<
		V: rdf_types::Vocabulary<Iri = treeldr::IriIndex, BlankId = treeldr::BlankIdIndex>,
	>(
		&self,
		context: &crate::Context<V, M>,
		scope: &crate::Scope,
	) -> Result<Self::Output, crate::Error> {
		let tr = context.type_trait(self.0).unwrap();

		let mut scope = scope.clone();
		scope.params.context = Some(syn::parse2(quote!(C)).unwrap());

		let mut associated_types_trait_objects = Vec::with_capacity(tr.associated_types.len());
		for ty in &tr.associated_types {
			if let AssociatedTypeBound::Types(classes) = ty.bound() {
				associated_types_trait_objects.push(associated_trait_object_type(
					context,
					&scope,
					ty.trait_object_ident(tr).unwrap(),
					classes,
				)?)
			};
		}

		Ok(syntax::ClassDynTraitDefinition {
			table: DynTableOf(self.0).generate_syntax(context, &scope)?,
			associated_types_trait_objects,
		})
	}
}

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
	scope: &Scope,
	ident: Ident,
	classes: &[TId<treeldr::Type>],
) -> Result<ClassAssociatedTypeTraitObject, Error> {
	let mut all_classes = BTreeSet::new();
	for &c in classes {
		collect_type_traits(context, c, |c| all_classes.insert(c))
	}

	let mut trait_bounds = Vec::with_capacity(classes.len());
	for ty in classes {
		trait_bounds.push(syn::TraitBound {
			paren_token: None,
			modifier: syn::TraitBoundModifier::None,
			lifetimes: None,
			path: ty.generate_syntax(context, scope)?,
		})
	}

	let mut tables = Vec::with_capacity(all_classes.len());
	for (i, ty) in all_classes.iter().enumerate() {
		let mut scope = scope.clone();
		scope.params.lifetime = Some(syn::Lifetime::new("'d", proc_macro2::Span::call_site()));

		let tr = context.type_trait(*ty).unwrap();
		let path = context
			.module_path(scope.module)
			.to(&tr.dyn_table_path(context).unwrap())
			.generate_syntax(context, &scope)?;
		let instance_path = context
			.module_path(scope.module)
			.to(&tr.dyn_table_instance_path(context).unwrap())
			.generate_syntax(context, &scope)?;
		let into_trait_object =
			quote!( ::treeldr_rust_prelude::AsTraitObject::<#path>::into_trait_object(value) );

		let initial_value = if i == 0 {
			quote!( {
				let (p, t) = #into_trait_object;
				ptr = p;
				t
			} )
		} else {
			quote!( #into_trait_object.1 )
		};

		tables.push(syntax::ClassAssociatedTypeTraitObjectTable {
			ty: syn::parse2(quote!(#instance_path)).unwrap(),
			initial_value,
		})
	}

	let mut trait_impls = Vec::with_capacity(all_classes.len());
	for (i, ty) in all_classes.iter().enumerate() {
		let tr = context.type_trait(*ty).unwrap();

		let mut associated_types = Vec::with_capacity(tr.associated_types().len());
		for a in tr.associated_types() {
			let mut scope = scope.clone();
			scope.params.lifetime = Some(syn::Lifetime::new("'a", proc_macro2::Span::call_site()));

			let ty = match a.trait_object_path(context, tr) {
				Some(path) => {
					let path = context
						.module_path(scope.module)
						.to(&path)
						.generate_syntax(context, &scope)?;
					syn::parse2(quote!(#path)).unwrap()
				}
				None => {
					let item_a = &tr.associated_types()[a.collection_item_type().unwrap()];
					let item_path = context
						.module_path(scope.module)
						.to(&item_a.trait_object_path(context, tr).unwrap())
						.generate_syntax(context, &scope)?;
					syn::parse2(quote!(::treeldr_rust_prelude::BoxedDynIterator<'a, #item_path>))
						.unwrap()
				}
			};

			associated_types.push((a.ident().clone(), ty))
		}

		let methods = tr
			.methods()
			.iter()
			.map(|m| syntax::ClassAssociatedTypeTraitObjectTraitImplMethod {
				ident: m.ident().clone(),
				return_ty: m.return_type_expr(tr),
				table_index: i,
			})
			.collect();

		let table_path = context
			.module_path(scope.module)
			.to(&tr.dyn_table_path(context).unwrap())
			.generate_syntax(context, scope)?;

		let mut scope = scope.clone();
		scope.params.lifetime = Some(syn::Lifetime::new("'r", proc_macro2::Span::call_site()));
		let table_instance_path = context
			.module_path(scope.module)
			.to(&tr.dyn_table_instance_path(context).unwrap())
			.generate_syntax(context, &scope)?;

		trait_impls.push(syntax::ClassAssociatedTypeTraitObjectTraitImpl {
			ident: ident.clone(),
			trait_path: context
				.module_path(scope.module)
				.to(&tr.path(context).unwrap())
				.generate_syntax(context, &scope)?,
			table_path,
			table_instance_path,
			table_index: i,
			associated_types,
			methods,
		})
	}

	Ok(syntax::ClassAssociatedTypeTraitObject {
		ident,
		tables,
		trait_bounds,
		trait_impls,
	})
}
