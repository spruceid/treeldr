use std::collections::{HashMap, BTreeMap};

use iref::{IriRefBuf, IriBuf, Iri};
use proc_macro2::{Span, TokenTree, TokenStream};
use rdf_types::BlankIdBuf;
use syn::spanned::Spanned;
use treeldr_layouts::abs::syntax::{Pattern, CompactIri, Layout, LiteralLayout, IdLayout, LayoutHeader, Quad, Dataset, DataLayout, UnitLayout, BooleanLayout, NumberLayout, TextStringLayout, ByteStringLayout, ProductLayout, Field, ValueFormatOrLayout, ValueFormat, Variant, VariantFormatOrLayout, VariantFormat, SumLayout, ListLayout, UnorderedListLayout, ListItem, OrderedListLayout, ListNodeOrLayout, ListNode, SizedListLayout};

#[derive(Default)]
pub struct TypeMap(HashMap<IriBuf, syn::Type>);

impl TypeMap {
	pub fn get(&self, iri: &Iri) -> Option<&syn::Type> {
		self.0.get(iri)
	}

	pub fn insert(&mut self, ty: syn::Type) -> IriBuf {
		let i = self.0.len();
		let iri = IriBuf::new(format!("rust:/#Type{i}")).unwrap();
		self.0.insert(iri.clone(), ty);
		iri
	}
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("invalid `tldr` attribute")]
	InvalidMeta(Span),

	#[error("unexpected token")]
	UnexpectedToken(TokenTree),

	#[error("unexpected literal")]
	UnexpectedLiteral(Span),

	#[error("missing token")]
	MissingToken(Span),

	#[error("invalid compact IRI `{0}`")]
	InvalidCompactIri(String, Span),

	#[error("invalid pattern")]
	InvalidPattern(String, Span),

	#[error("conflicting value")]
	ConflictingValue(Span),

	#[error("`union` type not supported")]
	UnionType(Span),

	#[error("missing datatype")]
	MissingDatatype(Span),

	#[error("expected `struct`")]
	ExpectedStruct(Span),

	#[error("expected named fields")]
	ExpectedNamedFields(Span),

	#[error("expected unnamed fields")]
	ExpectedUnnamedFields(Span),

	#[error("expected one field")]
	ExpectedOneField(Span),

	#[error("expected `Vec<_>` type")]
	ExpectedVec(Span)
}

impl Error {
	pub fn span(&self) -> Span {
		match self {
			Self::InvalidMeta(span) => *span,
			Self::UnexpectedToken(token) => token.span(),
			Self::UnexpectedLiteral(l) => l.span(),
			Self::MissingToken(span) => *span,
			Self::InvalidPattern(_, span) => *span,
			Self::InvalidCompactIri(_, span) => *span,
			Self::ConflictingValue(span) => *span,
			Self::UnionType(span) => *span,
			Self::MissingDatatype(span) => *span,
			Self::ExpectedStruct(span) => *span,
			Self::ExpectedNamedFields(span) => *span,
			Self::ExpectedUnnamedFields(span) => *span,
			Self::ExpectedOneField(span) => *span,
			Self::ExpectedVec(span) => *span
		}
	}
}

pub struct ParsedInput {
	pub ident: syn::Ident,
	pub type_map: TypeMap,
	pub layout: Layout
}

pub fn parse(
	input: syn::DeriveInput
) -> Result<ParsedInput, Error> {
	let type_attrs = TypeAttributes::parse(input.attrs)?;
	let kind = type_attrs.kind.map(Ok).unwrap_or_else(|| Kind::from_data(&input.data, input.ident.span()))?;
	let mut type_map = TypeMap::default();
	let layout = match kind {
		Kind::Id => {
			Layout::Literal(LiteralLayout::Id(IdLayout {
				type_: Default::default(),
				header: LayoutHeader {
					base: type_attrs.base,
					prefixes: type_attrs.prefixes,
					id: type_attrs.id,
					input: type_attrs.input.map(Into::into).unwrap_or_default(),
					intro: type_attrs.intro.map(Into::into).unwrap_or_default(),
					dataset: type_attrs.dataset.unwrap_or_default()
				},
				pattern: None, // TODO
				resource: type_attrs.resource
			}))
		}
		Kind::Unit => {
			Layout::Literal(LiteralLayout::Data(DataLayout::Unit(UnitLayout {
				type_: Default::default(),
				header: LayoutHeader {
					base: type_attrs.base,
					prefixes: type_attrs.prefixes,
					id: type_attrs.id,
					input: type_attrs.input.map(Into::into).unwrap_or_default(),
					intro: type_attrs.intro.map(Into::into).unwrap_or_default(),
					dataset: type_attrs.dataset.unwrap_or_default()
				},
				const_: treeldr_layouts::Value::default()
			})))
		}
		Kind::Boolean => {
			Layout::Literal(LiteralLayout::Data(DataLayout::Boolean(BooleanLayout {
				type_: Default::default(),
				header: LayoutHeader {
					base: type_attrs.base,
					prefixes: type_attrs.prefixes,
					id: type_attrs.id,
					input: type_attrs.input.map(Into::into).unwrap_or_default(),
					intro: type_attrs.intro.map(Into::into).unwrap_or_default(),
					dataset: type_attrs.dataset.unwrap_or_default()
				},
				datatype: type_attrs.datatype,
				resource: type_attrs.resource
			})))
		},
		Kind::Number => {
			Layout::Literal(LiteralLayout::Data(DataLayout::Number(NumberLayout {
				type_: Default::default(),
				header: LayoutHeader {
					base: type_attrs.base,
					prefixes: type_attrs.prefixes,
					id: type_attrs.id,
					input: type_attrs.input.map(Into::into).unwrap_or_default(),
					intro: type_attrs.intro.map(Into::into).unwrap_or_default(),
					dataset: type_attrs.dataset.unwrap_or_default()
				},
				datatype: type_attrs.datatype.ok_or_else(|| Error::MissingDatatype(input.ident.span()))?,
				resource: type_attrs.resource
			})))
		},
		Kind::String => {
			Layout::Literal(LiteralLayout::Data(DataLayout::TextString(TextStringLayout {
				type_: Default::default(),
				header: LayoutHeader {
					base: type_attrs.base,
					prefixes: type_attrs.prefixes,
					id: type_attrs.id,
					input: type_attrs.input.map(Into::into).unwrap_or_default(),
					intro: type_attrs.intro.map(Into::into).unwrap_or_default(),
					dataset: type_attrs.dataset.unwrap_or_default()
				},
				pattern: None, // TODO
				datatype: type_attrs.datatype,
				resource: type_attrs.resource
			})))
		}
		Kind::Bytes => {
			Layout::Literal(LiteralLayout::Data(DataLayout::ByteString(ByteStringLayout {
				type_: Default::default(),
				header: LayoutHeader {
					base: type_attrs.base,
					prefixes: type_attrs.prefixes,
					id: type_attrs.id,
					input: type_attrs.input.map(Into::into).unwrap_or_default(),
					intro: type_attrs.intro.map(Into::into).unwrap_or_default(),
					dataset: type_attrs.dataset.unwrap_or_default()
				},
				datatype: type_attrs.datatype.ok_or_else(|| Error::MissingDatatype(input.ident.span()))?,
				resource: type_attrs.resource
			})))
		}
		Kind::Record => {
			let fields = match input.data {
				syn::Data::Struct(s) => {
					match s.fields {
						syn::Fields::Named(fields) => {
							fields.named.into_iter().map(|f| {
								let name = f.ident.unwrap().to_string();
								let attrs = ComponentAttributes::parse(f.attrs)?;
								let field = Field {
									intro: attrs.intro.map(Into::into).unwrap_or_default(),
									dataset: attrs.dataset.unwrap_or_default(),
									property: attrs.property,
									value: ValueFormatOrLayout::Format(ValueFormat {
										layout: type_map.insert(f.ty).into(),
										input: attrs.input.map(Into::into).unwrap_or_default(),
										graph: attrs.graph.unwrap_or_default().into()
									})
								};

								Ok((name, field))
							}).collect::<Result<BTreeMap<_, _>, _>>()?
						}
						f => return Err(Error::ExpectedNamedFields(f.span()))
					}
				}
				_ => return Err(Error::ExpectedStruct(input.ident.span()))
			};

			Layout::Product(ProductLayout {
				type_: Default::default(),
				header: LayoutHeader {
					base: type_attrs.base,
					prefixes: type_attrs.prefixes,
					id: type_attrs.id,
					input: type_attrs.input.map(Into::into).unwrap_or_default(),
					intro: type_attrs.intro.map(Into::into).unwrap_or_default(),
					dataset: type_attrs.dataset.unwrap_or_default()
				},
				fields
			})
		}
		Kind::Sum => {
			let variants = match input.data {
				syn::Data::Enum(e) => {
					e.variants.into_iter().map(|v| {
						let name = v.ident.to_string();
						let attrs = ComponentAttributes::parse(v.attrs)?;

						match v.fields {
							syn::Fields::Unnamed(fields) => {
								if fields.unnamed.len() == 1 {
									let f = fields.unnamed.into_iter().next().unwrap();
									let field = Variant {
										intro: attrs.intro.map(Into::into).unwrap_or_default(),
										dataset: attrs.dataset.unwrap_or_default(),
										value: VariantFormatOrLayout::Format(VariantFormat {
											layout: type_map.insert(f.ty).into(),
											input: attrs.input.map(Into::into).unwrap_or_default(),
											graph: attrs.graph.unwrap_or_default().into()
										})
									};
			
									Ok((name, field))
								} else {
									Err(Error::ExpectedOneField(fields.span()))
								}
							}
							f => Err(Error::ExpectedUnnamedFields(f.span()))
						}
					}).collect::<Result<BTreeMap<_, _>, _>>()?
				}
				_ => return Err(Error::ExpectedStruct(input.ident.span()))
			};

			Layout::Sum(SumLayout {
				type_: Default::default(),
				header: LayoutHeader {
					base: type_attrs.base,
					prefixes: type_attrs.prefixes,
					id: type_attrs.id,
					input: type_attrs.input.map(Into::into).unwrap_or_default(),
					intro: type_attrs.intro.map(Into::into).unwrap_or_default(),
					dataset: type_attrs.dataset.unwrap_or_default()
				},
				variants
			})
		}
		Kind::Set => {
			match input.data {
				syn::Data::Struct(s) => {
					match s.fields {
						syn::Fields::Unnamed(fields) => {
							if fields.unnamed.len() == 1 {
								let f = fields.unnamed.into_iter().next().unwrap();
								let item_attrs = ComponentAttributes::parse(f.attrs)?;
		
								Layout::List(ListLayout::Unordered(UnorderedListLayout {
									type_: Default::default(),
									header: LayoutHeader {
										base: type_attrs.base,
										prefixes: type_attrs.prefixes,
										id: type_attrs.id,
										input: type_attrs.input.map(Into::into).unwrap_or_default(),
										intro: type_attrs.intro.map(Into::into).unwrap_or_default(),
										dataset: type_attrs.dataset.unwrap_or_default()
									},
									item: ListItem {
										intro: item_attrs.intro.map(Into::into).unwrap_or_default(),
										dataset: item_attrs.dataset.unwrap_or_default(),
										property: item_attrs.property,
										value: ValueFormatOrLayout::Format(ValueFormat {
											layout: type_map.insert(extract_vec_item(f.ty)?).into(),
											input: item_attrs.input.map(Into::into).unwrap_or_default(),
											graph: item_attrs.graph.unwrap_or_default().into()
										})
									}
								}))
							} else {
								return Err(Error::ExpectedOneField(fields.span()))
							}
						}
						f => return Err(Error::ExpectedUnnamedFields(f.span()))
					}
				},
				_ => return Err(Error::ExpectedStruct(input.ident.span()))
			}
		}
		Kind::List => {
			match input.data {
				syn::Data::Struct(s) => {
					match s.fields {
						syn::Fields::Unnamed(fields) => {
							if fields.unnamed.len() == 1 {
								let f = fields.unnamed.into_iter().next().unwrap();
								let node_attrs = ComponentAttributes::parse(f.attrs)?;
		
								Layout::List(ListLayout::Ordered(OrderedListLayout {
									type_: Default::default(),
									header: LayoutHeader {
										base: type_attrs.base,
										prefixes: type_attrs.prefixes,
										id: type_attrs.id,
										input: type_attrs.input.map(Into::into).unwrap_or_default(),
										intro: type_attrs.intro.map(Into::into).unwrap_or_default(),
										dataset: type_attrs.dataset.unwrap_or_default()
									},
									node: ListNodeOrLayout::ListNode(ListNode {
										head: node_attrs.head.unwrap_or_else(ListNode::default_head),
										rest: node_attrs.rest.unwrap_or_else(ListNode::default_rest),
										intro: node_attrs.intro.map(Into::into).unwrap_or_default(),
										dataset: node_attrs.dataset,
										value: ValueFormatOrLayout::Format(ValueFormat {
											layout: type_map.insert(extract_vec_item(f.ty)?).into(),
											input: node_attrs.input.map(Into::into).unwrap_or_default(),
											graph: node_attrs.graph.unwrap_or_default().into()
										})
									}),
									head: type_attrs.head.unwrap_or_else(Pattern::default_head),
									tail: type_attrs.tail.unwrap_or_else(Pattern::default_tail)
								}))
							} else {
								return Err(Error::ExpectedOneField(fields.span()))
							}
						}
						f => return Err(Error::ExpectedUnnamedFields(f.span()))
					}
				},
				_ => return Err(Error::ExpectedStruct(input.ident.span()))
			}
		}
		Kind::Tuple => {
			match input.data {
				syn::Data::Struct(s) => {
					match s.fields {
						syn::Fields::Unnamed(fields) => {
							let items = fields.unnamed.into_iter().map(|f| {
								let item_attrs = ComponentAttributes::parse(f.attrs)?;
								Ok(ListItem {
									intro: item_attrs.intro.map(Into::into).unwrap_or_default(),
									dataset: item_attrs.dataset.unwrap_or_default(),
									property: item_attrs.property,
									value: ValueFormatOrLayout::Format(ValueFormat {
										layout: type_map.insert(f.ty).into(),
										input: item_attrs.input.map(Into::into).unwrap_or_default(),
										graph: item_attrs.graph.unwrap_or_default().into()
									})
								})
							}).collect::<Result<Vec<_>, _>>()?;

							Layout::List(ListLayout::Sized(SizedListLayout {
								type_: Default::default(),
								header: LayoutHeader {
									base: type_attrs.base,
									prefixes: type_attrs.prefixes,
									id: type_attrs.id,
									input: type_attrs.input.map(Into::into).unwrap_or_default(),
									intro: type_attrs.intro.map(Into::into).unwrap_or_default(),
									dataset: type_attrs.dataset.unwrap_or_default()
								},
								items
							}))
						}
						f => return Err(Error::ExpectedUnnamedFields(f.span()))
					}
				},
				_ => return Err(Error::ExpectedStruct(input.ident.span()))
			}
		}
	};

	Ok(ParsedInput { ident: input.ident, type_map, layout })
}

#[derive(Default)]
pub struct TypeAttributes {
	base: Option<CompactIri>,
	prefixes: HashMap<String, CompactIri>,
	id: Option<CompactIri>,
	kind: Option<Kind>,
	input: Option<Vec<String>>,
	intro: Option<Vec<String>>,
	head: Option<Pattern>,
	tail: Option<Pattern>,
	dataset: Option<Dataset>,
	resource: Option<Pattern>,
	datatype: Option<CompactIri>
}

impl TypeAttributes {
	pub fn parse(attrs: Vec<syn::Attribute>) -> Result<Self, Error> {
		let mut result = TypeAttributes::default();
		
		for attr in attrs {
			if attr.path().is_ident("tldr") {
				match attr.meta {
					syn::Meta::List(meta) => {
						let mut tokens = meta.tokens.into_iter();
					
						while let Some(token) = tokens.next() {
							match token {
								TokenTree::Ident(ident) => {
									match Kind::from_ident(&ident) {
										Some(k) => {
											replace(&mut result.kind, k, ident.span())?
										}
										None => {
											if ident == "base" {
												let (value, span) = expect_compact_iri_argument(&mut tokens, ident.span())?;
												replace(&mut result.base, value, span)?;
											}

											if ident == "prefix" {
												let values = expect_binding_list_argument(&mut tokens, ident.span())?.0;
												result.prefixes.extend(values)
											}

											if ident == "id" {
												let (value, span) = expect_compact_iri_argument(&mut tokens, ident.span())?;
												replace(&mut result.id, value, span)?;
											}

											if ident == "head" {
												let (value, span) = expect_pattern_argument(&mut tokens, ident.span())?;
												replace(&mut result.head, value, span)?;
											}

											if ident == "tail" {
												let (value, span) = expect_pattern_argument(&mut tokens, ident.span())?;
												replace(&mut result.tail, value, span)?;
											}

											if ident == "resource" {
												let (value, span) = expect_pattern_argument(&mut tokens, ident.span())?;
												replace(&mut result.resource, value, span)?;
											}

											if ident == "datatype" {
												let (value, span) = expect_compact_iri_argument(&mut tokens, ident.span())?;
												replace(&mut result.datatype, value, span)?;
											}

											if ident == "input" {
												let (value, span) = expect_string_list_argument(&mut tokens, ident.span())?;
												replace(&mut result.input, value, span)?;
											}

											if ident == "intro" {
												let (value, span) = expect_string_list_argument(&mut tokens, ident.span())?;
												replace(&mut result.intro, value, span)?;
											}

											if ident == "dataset" {
												let (value, span) = expect_quad_list_argument(&mut tokens, ident.span())?;
												replace(&mut result.dataset, value, span)?;
											}
										}
									}
								}
								other => return Err(Error::UnexpectedToken(other))
							}

							match tokens.next() {
								Some(TokenTree::Punct(p)) if p.as_char() == ',' => (),
								Some(other) => return Err(Error::UnexpectedToken(other)),
								None => ()
							}
						}
					}
					other => return Err(Error::InvalidMeta(other.span()))
				}
			}
		}

		Ok(result)
	}
}

#[derive(Default)]
pub struct ComponentAttributes {
	head: Option<String>,
	rest: Option<String>,
	intro: Option<Vec<String>>,
	property: Option<Pattern>,
	dataset: Option<Dataset>,
	input: Option<Vec<Pattern>>,
	graph: Option<GraphValue>,
}

impl ComponentAttributes {
	pub fn parse(attrs: Vec<syn::Attribute>) -> Result<Self, Error> {
		let mut result = ComponentAttributes::default();
		
		for attr in attrs {
			if attr.path().is_ident("tldr") {
				match attr.meta {
					syn::Meta::List(meta) => {
						let mut tokens = meta.tokens.into_iter();
					
						while let Some(token) = tokens.next() {
							match token {
								TokenTree::Literal(lit) => {
									match syn::Lit::new(lit) {
										syn::Lit::Str(s) => {
											let value = parse_pattern(s.value(), s.span())?;
											replace(&mut result.property, value, s.span())?;
										}
										other => return Err(Error::UnexpectedLiteral(other.span()))
									}
								}
								TokenTree::Ident(ident) => {
									if ident == "head" {
										let (value, span) = expect_string_argument(&mut tokens, ident.span())?;
										replace(&mut result.head, value, span)?;
									}

									if ident == "rest" {
										let (value, span) = expect_string_argument(&mut tokens, ident.span())?;
										replace(&mut result.rest, value, span)?;
									}

									if ident == "intro" {
										let (value, span) = expect_string_list_argument(&mut tokens, ident.span())?;
										replace(&mut result.intro, value, span)?;
									}

									if ident == "dataset" {
										let (value, span) = expect_quad_list_argument(&mut tokens, ident.span())?;
										replace(&mut result.dataset, value, span)?;
									}

									if ident == "input" {
										let (value, span) = expect_pattern_list_argument(&mut tokens, ident.span())?;
										replace(&mut result.input, value, span)?;
									}

									if ident == "graph" {
										let (value, span) = expect_argument(&mut tokens, ident.span(), parse_graph_value)?;
										replace(&mut result.graph, value, span)?;
									}
								}
								other => return Err(Error::UnexpectedToken(other))
							}

							match tokens.next() {
								Some(TokenTree::Punct(p)) if p.as_char() == ',' => (),
								Some(other) => return Err(Error::UnexpectedToken(other)),
								None => ()
							}
						}
					}
					other => return Err(Error::InvalidMeta(other.span()))
				}
			}
		}

		Ok(result)
	}
}

// pub enum FieldType {
// 	Optional(syn::Type),
// 	Required(syn::Type)
// }

// impl FieldType {
// 	pub fn new(ty: syn::Type) -> Self {
// 		if is_option_type(&ty) {
// 			let syn::Type::Path(path) = ty else { unreachable!() };
// 			let syn::PathArguments::AngleBracketed(args) = path.path.segments.into_iter().next().unwrap().arguments else { unreachable!() };
// 			let syn::GenericArgument::Type(item) = args.args.into_iter().next().unwrap() else { unreachable!() };
// 			Self::Optional(item)
// 		} else {
// 			Self::Required(ty)
// 		}
// 	}

// 	pub fn is_required(&self) -> bool {
// 		matches!(self, Self::Required(_))
// 	}

// 	pub fn into_type(self) -> syn::Type {
// 		match self {
// 			Self::Required(ty) => ty,
// 			Self::Optional(ty) => ty
// 		}
// 	}
// }

// fn is_option_type(ty: &syn::Type) -> bool {
// 	if let syn::Type::Path(path) = ty {
// 		if path.qself.is_none() {
// 			if path.path.segments.len() == 1 {
// 				let segment = path.path.segments.iter().next().unwrap();
// 				if segment.ident == "Option" {
// 					if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
// 						if args.args.len() == 1 {
// 							if let syn::GenericArgument::Type(_) = args.args.iter().next().unwrap() {
// 								return true
// 							}
// 						}
// 					}
// 				}
// 			}
// 		}
// 	}

// 	false
// }

fn extract_vec_item(ty: syn::Type) -> Result<syn::Type, Error> {
	let span = ty.span();

	match ty {
		syn::Type::Path(path) => {
			match path.qself {
				Some(_) => Err(Error::ExpectedVec(span)),
				None => {
					if path.path.segments.len() == 1 {
						let segment = path.path.segments.into_iter().next().unwrap();
						if segment.ident == "Vec" {
							match segment.arguments {
								syn::PathArguments::AngleBracketed(args) => {
									if args.args.len() == 1 {
										match args.args.into_iter().next().unwrap() {
											syn::GenericArgument::Type(item) => Ok(item),
											_ => Err(Error::ExpectedVec(span))
										}
									} else {
										Err(Error::ExpectedVec(span))
									}
								}
								_ => Err(Error::ExpectedVec(span))
							}
						} else {
							Err(Error::ExpectedVec(span))
						}
					} else {
						Err(Error::ExpectedVec(span))
					}
				}
			}
		}
		_ => Err(Error::ExpectedVec(span))
	}
}

#[derive(Default, PartialEq, Eq)]
pub enum GraphValue {
	#[default]
	Keep,
	None,
	Some(Pattern)
}

impl From<GraphValue> for Option<Option<Pattern>> {
	fn from(value: GraphValue) -> Self {
		match value {
			GraphValue::Keep => None,
			GraphValue::None => Some(None),
			GraphValue::Some(g) => Some(Some(g))
		}
	}
}

fn replace<T: PartialEq>(target: &mut Option<T>, value: T, span: Span) -> Result<(), Error> {
	if let Some(old) = target.replace(value) {
		if old != *target.as_ref().unwrap() {
			return Err(Error::ConflictingValue(span));
		}
	}

	Ok(())
}

fn expect_argument<T>(
	tokens: &mut impl Iterator<Item = TokenTree>,
	span: Span,
	f: impl FnOnce(TokenStream, Span) -> Result<T, Error>
) -> Result<T, Error> {
	match tokens.next() {
		Some(TokenTree::Group(g)) => {
			f(g.stream(), g.span())
		}
		Some(other) => Err(Error::UnexpectedToken(other)),
		None => Err(Error::MissingToken(span))
	}
}

fn parse_string(
	tokens: TokenStream,
	span: Span
) -> Result<(String, Span), Error> {
	let mut tokens = tokens.into_iter();
	match tokens.next() {
		Some(token) => {
			if let Some(t) = tokens.next() {
				return Err(Error::UnexpectedToken(t))
			}

			string_token(token)
		},
		None => Err(Error::MissingToken(span))
	}
}

fn parse_compact_iri(
	tokens: TokenStream,
	span: Span
) -> Result<(CompactIri, Span), Error> {
	let (s, span) = parse_string(tokens, span)?;
	Ok((CompactIri(IriRefBuf::new(s).map_err(|e| Error::InvalidCompactIri(e.0, span))?), span))
}

fn string_token(token: TokenTree) -> Result<(String, Span), Error> {
	match token {
		TokenTree::Literal(l) => {
			match syn::Lit::new(l) {
				syn::Lit::Str(s) => Ok((s.value(), s.span())),
				l => Err(Error::UnexpectedLiteral(l.span())),
			}
		}
		other => Err(Error::UnexpectedToken(other))
	}
}

fn quad_token(token: TokenTree) -> Result<(Quad, Span), Error> {
	match token {
		TokenTree::Group(group) => {
			let span = group.span();
			let mut tokens = group.stream().into_iter();
			match tokens.next() {
				Some(t) => {
					let (s, s_span) = pattern_token(t)?;
					parse_comma(&mut tokens, s_span)?;
					match tokens.next() {
						Some(t) => {
							let (p, p_span) = pattern_token(t)?;
							parse_comma(&mut tokens, p_span)?;
							match tokens.next() {
								Some(t) => {
									let o = pattern_token(t)?.0;
									let g = if parse_opt_comma(&mut tokens)? {
										match tokens.next() {
											Some(t) => {
												let g = pattern_token(t)?.0;
												Some(g)
											}
											None => None
										}
									} else {
										if let Some(t) = tokens.next() {
											return Err(Error::UnexpectedToken(t))
										}

										None
									};

									Ok((Quad(s, p, o, g), span))
								}
								None => Err(Error::MissingToken(group.span()))
							}
						}
						None => Err(Error::MissingToken(group.span()))
					}
				}
				None => Err(Error::MissingToken(group.span()))
			}
		},
		other => Err(Error::UnexpectedToken(other))
	}
}

fn parse_opt_comma(tokens: &mut impl Iterator<Item = TokenTree>) -> Result<bool, Error> {
	match tokens.next() {
		Some(TokenTree::Punct(p)) if p.as_char() == ',' => Ok(true),
		Some(other) => Err(Error::UnexpectedToken(other)),
		None => Ok(false)
	}
}

fn parse_comma(tokens: &mut impl Iterator<Item = TokenTree>, span: Span) -> Result<(), Error> {
	if parse_opt_comma(tokens)? {
		Ok(())
	} else {
		Err(Error::MissingToken(span))
	}
}

fn parse_list<T>(
	tokens: TokenStream,
	span: Span,
	f: impl Fn(TokenTree) -> Result<(T, Span), Error>
) -> Result<(Vec<T>, Span), Error> {
	let mut result = Vec::new();
	let mut tokens = tokens.into_iter();
	while let Some(token) = tokens.next() {
		let (value, _) = f(token)?;
		parse_opt_comma(&mut tokens)?;
		result.push(value)
	}

	Ok((result, span))
}

fn parse_string_list(
	tokens: TokenStream,
	span: Span
) -> Result<(Vec<String>, Span), Error> {
	parse_list(tokens, span, string_token)
}

fn parse_pattern_list(
	tokens: TokenStream,
	span: Span
) -> Result<(Vec<Pattern>, Span), Error> {
	parse_list(tokens, span, pattern_token)
}

fn parse_quad_list(
	tokens: TokenStream,
	span: Span
) -> Result<(Vec<Quad>, Span), Error> {
	parse_list(tokens, span, quad_token)
}

fn expect_string_argument(
	tokens: &mut impl Iterator<Item = TokenTree>,
	span: Span
) -> Result<(String, Span), Error> {
	expect_argument(tokens, span, parse_string)
}

fn expect_compact_iri_argument(
	tokens: &mut impl Iterator<Item = TokenTree>,
	span: Span
) -> Result<(CompactIri, Span), Error> {
	expect_argument(tokens, span, parse_compact_iri)
}

fn expect_string_list_argument(
	tokens: &mut impl Iterator<Item = TokenTree>,
	span: Span
) -> Result<(Vec<String>, Span), Error> {
	expect_argument(tokens, span, parse_string_list)
}

fn expect_pattern_list_argument(
	tokens: &mut impl Iterator<Item = TokenTree>,
	span: Span
) -> Result<(Vec<Pattern>, Span), Error> {
	expect_argument(tokens, span, parse_pattern_list)
}

fn expect_pattern_argument(
	tokens: &mut impl Iterator<Item = TokenTree>,
	span: Span
) -> Result<(Pattern, Span), Error> {
	let (value, v_span) = expect_string_argument(tokens, span)?;
	Ok((parse_pattern(value, span)?, v_span))
}

fn expect_quad_list_argument(
	tokens: &mut impl Iterator<Item = TokenTree>,
	span: Span
) -> Result<(Dataset, Span), Error> {
	expect_argument(tokens, span, parse_quad_list).map(|(q, s)| (q.into(), s))
}

fn parse_pattern(value: String, span: Span) -> Result<Pattern, Error> {
	match BlankIdBuf::new(value) {
		Ok(blank_id) => Ok(Pattern::Var(blank_id.suffix().to_owned())),
		Err(e) => match IriRefBuf::new(e.0) {
			Ok(iri_ref) => Ok(Pattern::Iri(CompactIri(iri_ref))),
			Err(e) => Err(Error::InvalidPattern(e.0, span)),
		},
	}
}

fn pattern_token(token: TokenTree) -> Result<(Pattern, Span), Error> {
	let (value, span) = string_token(token)?;
	Ok((parse_pattern(value, span)?, span))
}

fn parse_graph_value(
	tokens: TokenStream,
	span: Span
) -> Result<(GraphValue, Span), Error> {
	let mut tokens = tokens.into_iter();
	match tokens.next() {
		Some(TokenTree::Ident(id)) => {
			if id == "_" {
				match tokens.next() {
					Some(t) => Err(Error::UnexpectedToken(t)),
					None => Ok((GraphValue::Keep, id.span()))
				}
			} else if id == "None" {
				match tokens.next() {
					Some(t) => Err(Error::UnexpectedToken(t)),
					None => Ok((GraphValue::None, id.span()))
				}
			} else if id == "Some" {
				match tokens.next() {
					Some(TokenTree::Group(g)) => {
						let span = g.span();
						let mut tokens = g.stream().into_iter();
						match tokens.next() {
							Some(t) => {
								let (pattern, span) = pattern_token(t)?;
								Ok((GraphValue::Some(pattern), span))
							}
							None => Err(Error::MissingToken(span))
						}
					}
					Some(other) => Err(Error::UnexpectedToken(other)),
					None => Err(Error::MissingToken(id.span()))
				}
			} else {
				Err(Error::UnexpectedToken(TokenTree::Ident(id)))
			}
		},
		Some(other) => Err(Error::UnexpectedToken(other)),
		None => Err(Error::MissingToken(span))
	}
}

fn expect_binding_list_argument(
	tokens: &mut impl Iterator<Item = TokenTree>,
	span: Span
) -> Result<(Vec<(String, CompactIri)>, Span), Error> {
	expect_argument(tokens, span, parse_binding_list)
}

fn parse_binding_list(
	tokens: TokenStream,
	span: Span
) -> Result<(Vec<(String, CompactIri)>, Span), Error> {
	let mut tokens = tokens.into_iter();
	let mut result = Vec::new();
	while let Some(token) = tokens.next() {
		let prefix = string_token(token)?.0;
		match tokens.next() {
			Some(TokenTree::Punct(p)) if p.as_char() == '=' => {
				match tokens.next() {
					Some(token) => {
						let value = CompactIri(IriRefBuf::new(string_token(token)?.0).map_err(|e| Error::InvalidCompactIri(e.0, span))?);
						result.push((prefix, value));
						parse_opt_comma(&mut tokens)?;
					}
					None => return Err(Error::MissingToken(span))
				}
			}
			Some(token) => return Err(Error::UnexpectedToken(token)),
			None => return Err(Error::MissingToken(span))
		}
	}

	Ok((result, span))
}

#[derive(PartialEq, Eq)]
pub enum Kind {
	Id,
	Unit,
	Boolean,
	Number,
	String,
	Bytes,
	Record,
	Sum,
	Set,
	List,
	Tuple
}

impl Kind {
	pub fn from_ident(ident: &syn::Ident) -> Option<Self> {
		if ident == "id" {
			Some(Self::Id)
		} else if ident == "unit" {
			Some(Self::Unit)
		} else if ident == "boolean" {
			Some(Self::Boolean)
		} else if ident == "number" {
			Some(Self::Number)
		} else if ident == "string" {
			Some(Self::String)
		} else if ident == "bytes" {
			Some(Self::Bytes)
		} else if ident == "record" {
			Some(Self::Record)
		} else if ident == "sum" {
			Some(Self::Sum)
		} else if ident == "set" {
			Some(Self::Set)
		} else if ident == "list" {
			Some(Self::List)
		} else if ident == "tuple" {
			Some(Self::Tuple)
		} else {
			None
		}
	}

	pub fn from_data(data: &syn::Data, span: Span) -> Result<Self, Error> {
		match data {
			syn::Data::Struct(s) => {
				match s.fields {
					syn::Fields::Named(_) => Ok(Self::Record),
					syn::Fields::Unnamed(_) => Ok(Self::Tuple),
					syn::Fields::Unit => Ok(Self::Unit)
				}
			}
			syn::Data::Enum(_) => Ok(Self::Sum),
			syn::Data::Union(_) => Err(Error::UnionType(span))
		}
	}
}