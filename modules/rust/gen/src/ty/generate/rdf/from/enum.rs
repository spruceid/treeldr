use std::collections::{BTreeSet, HashSet};

use locspan::Meta;
use proc_macro2::TokenStream;
use quote::quote;
use rdf_types::Vocabulary;
use treeldr::{vocab, BlankIdIndex, Id, IriIndex, TId};

use crate::{
	ty::{
		enumeration::{Enum, Variant},
		generate::GenerateFor,
		params::ParametersValues,
	},
	Context, Generate, GenerateList,
};

use super::{collect_bounds, FromRdfImpl};

impl<M> GenerateFor<Enum, M> for FromRdfImpl {
	fn generate<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: Option<shelves::Ref<crate::Module>>,
		ty: &Enum,
		tokens: &mut TokenStream,
	) -> Result<(), crate::Error> {
		let ident = ty.ident();
		let params_values = ParametersValues::new_for_type(quote!(N::Id));
		let params = ty.params().instantiate(&params_values);
		let mut bounds = BTreeSet::new();

		let mut id_set = BTreeSet::new();
		let id_signatures: Vec<_> = ty
			.variants()
			.iter()
			.enumerate()
			.map(
				|(i, variant)| match VariantSignature::build_for_id(context.model(), variant) {
					Some(sig) => {
						id_set.insert(i);

						if let Some(variant_layout_ref) = variant.ty() {
							collect_bounds(context, variant_layout_ref, |b| {
								bounds.insert(b);
							});
						}

						sig
					}
					None => VariantSignature::default(),
				},
			)
			.collect();

		let mut literal_set = BTreeSet::new();
		let literal_signatures: Vec<_> = ty
			.variants()
			.iter()
			.enumerate()
			.map(|(i, variant)| {
				match VariantSignature::build_for_literal(context.model(), variant) {
					Some(sig) => {
						literal_set.insert(i);

						if let Some(variant_layout_ref) = variant.ty() {
							collect_bounds(context, variant_layout_ref, |b| {
								bounds.insert(b);
							});
						}

						sig
					}
					None => VariantSignature::default(),
				}
			})
			.collect();

		let id_tree = DecisionTree::build(&id_signatures, id_set);
		let literal_tree = DecisionTree::build(&literal_signatures, literal_set);

		let id_branch = id_tree.generate(
			context,
			ty.variants(),
			&quote!(::treeldr_rust_prelude::FromRdfError::ExpectedLiteralValue),
		);
		let literal_branch = literal_tree.generate(
			context,
			ty.variants(),
			&quote!(::treeldr_rust_prelude::FromRdfError::UnexpectedLiteralValue),
		);

		let bounds = bounds
			.separated_by(&quote!(,))
			.generate_with(context, scope)
			.into_tokens()?;

		tokens.extend(quote! {
			impl<N: ::treeldr_rust_prelude::rdf_types::Namespace, V> ::treeldr_rust_prelude::FromRdf<N, V> for #ident #params
			where
				N: ::treeldr_rust_prelude::rdf_types::IriVocabularyMut,
				N::Id: Clone + Ord + ::treeldr_rust_prelude::rdf_types::FromIri<Iri=N::Iri>,
				V: ::treeldr_rust_prelude::rdf::TypeCheck<N::Id>,
				#bounds
			{
				fn from_rdf<G>(
					namespace: &mut N,
					value: &::treeldr_rust_prelude::rdf_types::Object<N::Id, V>,
					graph: &G
				) -> Result<Self, ::treeldr_rust_prelude::FromRdfError>
				where
					G: ::treeldr_rust_prelude::grdf::Graph<
						Subject=N::Id,
						Predicate=N::Id,
						Object=::treeldr_rust_prelude::rdf_types::Object<N::Id, V>
					>
				{
					match value {
						::treeldr_rust_prelude::rdf_types::Object::Id(id) => {
							#id_branch
						}
						::treeldr_rust_prelude::rdf_types::Object::Literal(literal) => {
							#literal_branch
						}
					}
				}
			}
		});

		Ok(())
	}
}

struct VariantSignature<C>(HashSet<C>);

impl<C> Default for VariantSignature<C> {
	fn default() -> Self {
		Self(HashSet::new())
	}
}

impl<C: Condition> VariantSignature<C> {
	pub fn insert(&mut self, condition: C) {
		self.0.insert(condition);
	}

	pub fn contains(&self, condition: C) -> bool {
		self.0.contains(&condition)
	}
}

impl VariantSignature<IdCondition> {
	pub fn build_for_id<M>(model: &treeldr::Model<M>, variant: &Variant) -> Option<Self> {
		let mut result = Self::default();
		let mut can_be_id = false;

		if let Some(layout_ref) = variant.ty() {
			can_be_id |= Self::build_for_id_from_layout(model, layout_ref, &mut result)
		}

		if can_be_id {
			Some(result)
		} else {
			None
		}
	}

	pub fn build_for_id_from_layout<M>(
		model: &treeldr::Model<M>,
		layout_ref: TId<treeldr::Layout>,
		result: &mut VariantSignature<IdCondition>,
	) -> bool {
		use treeldr::layout::Description;
		let layout = model.get(layout_ref).unwrap();
		match layout.as_layout().description() {
			Description::Alias(target) => Self::build_for_id_from_layout(model, **target, result),
			Description::Array(_) => {
				result.insert(IdCondition::Type(TId::new(Id::Iri(IriIndex::Iri(
					vocab::Term::Rdf(vocab::Rdf::List),
				)))));
				result.insert(IdCondition::Property(TId::new(Id::Iri(IriIndex::Iri(
					vocab::Term::Rdf(vocab::Rdf::First),
				)))));
				result.insert(IdCondition::Property(TId::new(Id::Iri(IriIndex::Iri(
					vocab::Term::Rdf(vocab::Rdf::Nil),
				)))));
				true
			}
			Description::Derived(_) => true,
			Description::Enum(e) => {
				let mut can_be_id = false;

				for Meta(variant_ref, _) in e.variants() {
					let variant = model.get(*variant_ref).unwrap();
					if let Some(layout_ref) = variant.as_formatted().format() {
						can_be_id |= Self::build_for_id_from_layout(model, layout_ref, result)
					}
				}

				can_be_id
			}
			Description::Never => false,
			Description::OneOrMany(_) => true,
			Description::Option(_) => true,
			Description::Primitive(_) => false,
			Description::Reference(_) => true,
			Description::Required(item) => {
				Self::build_for_id_from_layout(model, **item.item_layout(), result)
			}
			Description::Set(_) => false,
			Description::Struct(s) => {
				for Meta(field_ref, _) in s.fields() {
					let field = model.get(*field_ref).unwrap();
					if let Some(prop_ref) = field.as_layout_field().property() {
						result.insert(IdCondition::Property(*prop_ref));
					}
				}

				true
			}
		}
	}
}

impl VariantSignature<LiteralCondition> {
	pub fn build_for_literal<M>(model: &treeldr::Model<M>, variant: &Variant) -> Option<Self> {
		let mut result = Self::default();
		let mut can_be_literal = false;

		if let Some(layout_ref) = variant.ty() {
			can_be_literal |= Self::build_for_literal_from_layout(model, layout_ref, &mut result)
		}

		if can_be_literal {
			Some(result)
		} else {
			None
		}
	}

	pub fn build_for_literal_from_layout<M>(
		model: &treeldr::Model<M>,
		layout_ref: TId<treeldr::Layout>,
		result: &mut VariantSignature<LiteralCondition>,
	) -> bool {
		use treeldr::layout::Description;
		let layout = model.get(layout_ref).unwrap();
		match layout.as_layout().description() {
			Description::Alias(target) => {
				Self::build_for_literal_from_layout(model, **target, result)
			}
			Description::Array(_) => false,
			Description::Derived(_) => {
				for ty in layout.as_layout().ty() {
					result.insert(LiteralCondition::Type(**ty.value));
				}

				true
			}
			Description::Enum(e) => {
				let mut can_be_literal = false;

				for Meta(variant_ref, _) in e.variants() {
					let variant = model.get(*variant_ref).unwrap();
					if let Some(layout_ref) = variant.as_formatted().format() {
						can_be_literal |=
							Self::build_for_literal_from_layout(model, layout_ref, result)
					}
				}
				can_be_literal
			}
			Description::Never => false,
			Description::OneOrMany(_) => false,
			Description::Option(_) => false,
			Description::Primitive(_) => {
				for ty in layout.as_layout().ty() {
					result.insert(LiteralCondition::Type(**ty.value));
				}

				true
			}
			Description::Reference(r) => {
				let id_layout = model.get(**r.id_layout()).unwrap();
				for ty in id_layout.as_layout().ty() {
					result.insert(LiteralCondition::Type(**ty.value));
				}

				true
			}
			Description::Required(item) => {
				Self::build_for_literal_from_layout(model, **item.item_layout(), result)
			}
			Description::Set(_) => false,
			Description::Struct(_) => false,
		}
	}
}

trait Condition: Copy + Ord + std::hash::Hash {
	fn generate<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>, M>(
		&self,
		context: &Context<V, M>,
	) -> TokenStream;
}

enum DecisionTree<C> {
	Node {
		condition: C,
		then: Box<Self>,
		else_: Box<Self>,
	},
	Leaf(BTreeSet<usize>),
}

impl<C: Condition> DecisionTree<C> {
	fn build(signatures: &[VariantSignature<C>], set: BTreeSet<usize>) -> Self {
		match Self::best_split(signatures, &set) {
			Some(condition) => {
				let mut then = BTreeSet::new();
				let mut else_ = BTreeSet::new();

				for i in set {
					if signatures[i].contains(condition) {
						then.insert(i);
					} else {
						else_.insert(i);
					}
				}

				Self::Node {
					condition,
					then: Box::new(Self::build(signatures, then)),
					else_: Box::new(Self::build(signatures, else_)),
				}
			}
			None => Self::Leaf(set),
		}
	}

	fn split_score(
		signatures: &[VariantSignature<C>],
		set: &BTreeSet<usize>,
		condition: C,
	) -> usize {
		let mut then = 0;
		let mut else_ = 0;

		for &i in set {
			if signatures[i].contains(condition) {
				then += 1
			} else {
				else_ += 1
			}
		}

		std::cmp::min(then, else_)
	}

	fn best_split(signatures: &[VariantSignature<C>], set: &BTreeSet<usize>) -> Option<C> {
		if set.len() > 1 {
			let mut common_conditions = BTreeSet::new();

			for &i in set {
				common_conditions.extend(signatures[i].0.iter().copied());
			}

			let mut best_split_score = 0;
			let mut best_split = None;
			for condition in common_conditions {
				let score = Self::split_score(signatures, set, condition);
				if score > best_split_score {
					best_split = Some(condition);
					best_split_score = score
				}
			}

			best_split
		} else {
			None
		}
	}

	fn generate<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>, M>(
		&self,
		context: &Context<V, M>,
		variants: &[Variant],
		error: &TokenStream,
	) -> TokenStream {
		match self {
			Self::Node {
				condition,
				then,
				else_,
			} => {
				let rust_condition = condition.generate(context);
				let rust_then = then.generate(context, variants, error);
				let rust_else = else_.generate(context, variants, error);

				quote! {
					if #rust_condition {
						#rust_then
					} else {
						#rust_else
					}
				}
			}
			Self::Leaf(set) => match set.first() {
				Some(i) => {
					let variant = &variants[*i];
					let v_ident = variant.ident();

					match variant.ty() {
						Some(_) => {
							quote! {
								Ok(Self::#v_ident(::treeldr_rust_prelude::rdf::FromRdf::from_rdf(namespace, value, graph)?))
							}
						}
						None => {
							quote! {
								Ok(Self::#v_ident)
							}
						}
					}
				}
				None => {
					quote! {
						Err(#error)
					}
				}
			},
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IdCondition {
	Type(TId<treeldr::Type>),
	Property(TId<treeldr::Property>),
}

impl Condition for IdCondition {
	fn generate<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>, M>(
		&self,
		context: &Context<V, M>,
	) -> TokenStream {
		match *self {
			Self::Type(ty_ref) => {
				let property = quote! {
					::treeldr_rust_prelude::rdf_types::FromIri::from_iri(
						namespace.insert(::treeldr_rust_prelude::static_iref::iri!(
							"http://www.w3.org/1999/02/22-rdf-syntax-ns#type"
						))
					)
				};

				let ty_iri_index = ty_ref.id().into_iri().unwrap();
				let ty_iri = context.vocabulary().iri(&ty_iri_index).unwrap().into_str();
				let ty_id = quote! {
					::treeldr_rust_prelude::rdf_types::FromIri::from_iri(
						namespace.insert(::treeldr_rust_prelude::static_iref::iri!(
							#ty_iri
						))
					)
				};

				quote! {
					graph.any_match(::treeldr_rust_prelude::rdf_types::Triple(Some(id), Some(&#property), Some(&#ty_id))).is_some()
				}
			}
			Self::Property(prop_ref) => {
				let iri_index = prop_ref.id().into_iri().unwrap();
				let iri = context.vocabulary().iri(&iri_index).unwrap().into_str();
				let property = quote! {
					::treeldr_rust_prelude::rdf_types::FromIri::from_iri(
						namespace.insert(::treeldr_rust_prelude::static_iref::iri!(
							#iri
						))
					)
				};

				quote! {
					graph.any_match(::treeldr_rust_prelude::rdf_types::Triple(Some(id), Some(&#property), None)).is_some()
				}
			}
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LiteralCondition {
	Type(TId<treeldr::Type>),
}

impl Condition for LiteralCondition {
	fn generate<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>, M>(
		&self,
		context: &Context<V, M>,
	) -> TokenStream {
		match self {
			Self::Type(type_ref) => {
				let iri_index = type_ref.id().into_iri().unwrap();
				let iri = context.vocabulary().iri(&iri_index).unwrap().into_str();
				quote! {
					::treeldr_rust_prelude::TypeCheck::<N::Id>::has_type(literal, &::treeldr_rust_prelude::rdf_types::FromIri::from_iri(
						namespace.insert(::treeldr_rust_prelude::static_iref::iri!(
							#iri
						))
					))
				}
			}
		}
	}
}
