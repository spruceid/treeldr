use std::{borrow::Cow, collections::{BTreeMap, HashMap}, hash::Hash, rc::Rc, sync::Arc};

use educe::Educe;
use iref::{InvalidIri, Iri, IriBuf};
use langtag::{InvalidLangTag, LangTagBuf};
use ll1::{Span, Spanned};
use rdf_types::{Quad, RDF_FIRST, RDF_NIL, RDF_REST, XSD_STRING};

use crate::{eval::RdfContextMut, expr::{self, Bound, Expr, InnerExpr, ListComponents}, function::{self, FunctionRef, Signature}, ty, utils, value::{Number, NumberParseError, TypedMap, TypedValue, TypedValueDesc, TypedValueInnerDesc}, DatasetPattern, Function, Literal, Module, TermPattern, Type, TypeRef};

use super::ast::{self, Ident, ListDescription};

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("invalid number: {1}")]
	Number(Span, NumberParseError),

	#[error("invalid string")]
	String(Span),

	#[error("invalid IRI")]
	Iri(Span, String),

	#[error("invalid language tag `{1}`")]
	Language(Span, String),

	#[error("unknown IRI prefix `{1}`")]
	UnknownPrefix(Span, String),

	#[error("unknown variable `{0}`")]
	UnknownVariable(Ident),

	#[error("unknown type `{0}`")]
	UnknownType(Ident),

	#[error("unknown function `{0}`")]
	UnknownFunction(Ident),

	#[error("unknown field")]
	UnknownField(Span),

	#[error("unknown variant `{0}`")]
	UnknownVariant(Ident),

	#[error("invalid number of arguments (expected {expected}, found {found})")]
	ArgumentCount {
		expected: usize,
		found: usize
	},

	#[error("type mismatch")]
	TypeMismatch(Span),

	#[error("expected RDF resource")]
	ExpectedResource(Span),

	#[error("expected exactly one binding")]
	ExpectedExactlyOneBinding(Span)
}

pub trait Context<R> {
	fn get_prefix(&self, name: &str) -> Result<Option<&Iri>, Error>;

	fn get_type(&self, id: &str) -> Option<&TypePatternRef>;

	fn get_function(&self, id: &str) -> Option<(&FunctionRef<R>, &Signature<TypePatternRef>)>;

	fn new_type(&self, pattern: Option<TypePattern<R>>) -> TypePatternRef;

	fn set_type(&mut self, ty: TypePatternRef, pattern: TypePattern<R>);

	fn set_type_min(&mut self, ty: TypePatternRef, pattern: TypePattern<R>);
}

struct TypeConstraints<R> {
	max: Vec<TypePattern<R>>,
	min: Vec<TypePattern<R>>
}

pub struct Scope<'a> {
	parent: Option<&'a Self>,
	variables: Vec<(String, TypePatternRef)>,
	bindings: HashMap<String, u32>,
	count: u32
}

impl<'a> Scope<'a> {
	pub fn new(parent: Option<&'a Self>) -> Self {
		Self {
			parent,
			variables: Vec::new(),
			bindings: HashMap::new(),
			count: 0
		}
	}

	pub fn offset(&self) -> u32 {
		match self.parent {
			Some(p) => p.len(),
			None => 0
		}
	}

	pub fn len(&self) -> u32 {
		self.offset() + self.count
	}

	pub fn insert(&mut self, ident: &ast::Ident, ty: TypePatternRef) -> u32 {
		let i = self.len();
		self.variables.push((ident.as_str().to_owned(), ty));
		self.bindings.insert(ident.as_str().to_owned(), i);
		i
	}

	pub fn get(&self, i: u32) -> Option<(&str, TypePatternRef)> {
		match self.parent {
			Some(p) => {
				let offset = p.len();

				if i < offset {
					p.get(i)
				} else {
					self.variables.get((i - offset) as usize).map(|(n, t)| (n.as_str(), *t))
				}
			}
			None => {
				self.variables.get(i as usize).map(|(n, t)| (n.as_str(), *t))
			}
		}
	}

	pub fn index_of(&self, ident: &ast::Ident) -> Result<u32, Error> {
		match self
			.bindings
			.get(ident.as_str()) {
				Some(i) => Ok(*i),
				None => match self.parent {
					Some(p) => p.index_of(ident),
					None => Err(Error::UnknownVariable(ident.clone()))
				}
			}
	}

	pub fn type_of(&self, i: u32) -> Option<TypePatternRef> {
		self.get(i).map(|(_, ty)| ty)
	}

	pub fn begin<R>(&self, context: &mut impl Context<R>, bindings: &ast::Bindings) -> Scope {
		let mut sub_scope = Scope::new(Some(self));
		
		for ident in &bindings.variables {
			let ty = context.new_type(Some(TypePattern::resource()));
			sub_scope.insert(ident, ty);
		}

		sub_scope
	}
}

enum Building<T, U> {
	Unbuilt(T),
	Building,
	Built(U)
}

pub type TypePatternRef = u32;

/// Type pattern.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Educe)]
#[educe(PartialOrd(bound = "R: 'static + PartialOrd"))]
#[educe(Ord(bound = "R: 'static + Ord"))]
pub enum TypePattern<R> {
	Var(u32),
	Type(Type<R, TypePatternRef>)
}

impl<R> TypePattern<R> {
	pub fn resource() -> Self {
		Self::Type(Type::resource())
	}

	pub fn resolve(&self, f: impl Fn(u32) -> TypeRef<R>) -> TypeRef<R> {
		match self {
			Self::Var(x) => f(*x),
			Self::Type(ty) => {
				todo!()
			}
		}
	}
}

struct ModuleWithPrefix<'a, R, C> {
	parent: &'a C,
	prefixes: HashMap<String, IriBuf>,
	functions: HashMap<String, (FunctionRef<R>, Signature<TypePatternRef>)>,
	types: HashMap<String, TypePatternRef>
}

impl<'a, R, C> ModuleWithPrefix<'a, R, C> {
	fn new(
		parent: &'a C,
	) -> Self {
		Self {
			parent,
			prefixes: HashMap::new(),
			functions: HashMap::new(),
			types: HashMap::new()
		}
	}

	fn declare_prefix(&mut self, ident: &Ident) {
		todo!()
	}
}

impl<'a, R, C: Context<R>> Context<R> for ModuleWithPrefix<'a, R, C> {
	fn get_prefix(&self, name: &str) -> Result<Option<&Iri>, Error> {
		todo!()
	}

	fn get_type(&self, id: &str) -> Option<&TypePatternRef> {
		todo!()
	}

	fn get_function(&self, id: &str) -> Option<(&FunctionRef<R>, &Signature<TypePatternRef>)> {
		todo!()
	}

	fn new_type(&self, min: Option<TypePattern<R>>) -> TypePatternRef {
		todo!()
	}

	fn set_type_min(&mut self, ty: TypePatternRef, pattern: TypePattern<R>) {
		todo!()
	}

	fn set_type(&mut self, ty: TypePatternRef, pattern: TypePattern<R>) {
		todo!()
	}
}

pub trait Build<R> {
	type Target;

	fn build(
		&self,
		rdf: &mut impl RdfContextMut<R>,
		context: &mut impl Context<R>,
		scope: &Scope
	) -> Result<Self::Target, Error>;
}

impl<R: 'static + Clone + Ord + Hash> Build<R> for ast::Module {
	type Target = Module<R>;

	fn build(
		&self,
		rdf: &mut impl RdfContextMut<R>,
		context: &mut impl Context<R>,
		scope: &Scope
	) -> Result<Self::Target, Error> {
		// let mut sub_context = ModuleWithPrefix::new(context);

		// let type_variables = HashMap::new();

		// // Declare.
		// for item in &self.items {
		// 	match item {
		// 		ast::Item::Prefix(prefix_ast) => {
		// 			sub_context.declare_prefix(&prefix_ast.ident);
		// 		},
		// 		ast::Item::Function(f_ast) => {
		// 			// sub_context.declare_function(f_ast.)
		// 			todo!()
		// 		}
		// 	}
		// }

		// // Define.
		// for item in &self.items {
		// 	match item {
		// 		ast::Item::Prefix(prefix_ast) => {
		// 			// ...
		// 		},
		// 		ast::Item::Function(f_ast) => {
		// 			let return_type = match &f_ast.type_ {
		// 				Some(ty) => ty.expr.build(rdf, context, scope)?,
		// 				None => f_ast.body.preferred_type(rdf, context, scope)?
		// 			};

		// 			let mut sub_scope = Scope::new(Some(scope));

		// 			let args = f_ast.args.iter().map(|a| {
		// 				Ok(match a {
		// 					ast::ArgDef::Typed(_, ident, ty, _) => {
		// 						let type_ref = ty.expr.build(rdf, context, scope)?;
		// 						sub_scope.insert(ident, type_ref.clone());
		// 						type_ref
		// 					}
		// 					ast::ArgDef::Untyped(ident) => {
		// 						let type_ref = Type::resource().into_ref();
		// 						sub_scope.insert(ident, type_ref.clone());
		// 						type_ref
		// 					}
		// 				})
		// 			}).collect::<Result<_, _>>()?;

		// 			let body = f_ast.body.build_typed(
		// 				rdf,
		// 				context,
		// 				&sub_scope,
		// 				&return_type
		// 			)?;

		// 			let f = Function {
		// 				signature: Signature {
		// 					args,
		// 					return_type: return_type.clone(),
		// 					dataset: DatasetPattern::default()
		// 				},
		// 				body: function::Body::Expr(Box::new(body))
		// 			};

		// 			result.functions.insert(f_ast.ident.as_str().to_owned(), Arc::new(f));
		// 		}
		// 	}
		// }

		// Ok(result)
		todo!()
	}
}

impl<R: 'static + Clone + Ord + Hash> Build<R> for ast::TypeExpr {
	type Target = TypePatternRef;

	fn build(
		&self,
		rdf: &mut impl RdfContextMut<R>,
		context: &mut impl Context<R>,
		scope: &Scope
	) -> Result<Self::Target, Error> {
		match self {
			Self::Var(ident) => context
				.get_type(ident.as_str())
				.cloned()
				.ok_or_else(|| Error::UnknownType(ident.clone())),
			Self::Literal(expr) => {
				let (_, ty) = expr.build(rdf, context, scope)?;
				Ok(context.new_type(Some(TypePattern::Type(Type::Literal(ty)))))
			},
			Self::List(expr) => {
				let ty = expr.build(rdf, context, scope)?;
				Ok(context.new_type(Some(TypePattern::Type(Type::List(ty)))))
			},
			Self::Map(expr) => {
				let ty = expr.build(rdf, context, scope)?;
				Ok(context.new_type(Some(TypePattern::Type(Type::Struct(ty)))))
			}
			Self::Enum(expr) => {
				let ty = expr.build(rdf, context, scope)?;
				Ok(context.new_type(Some(TypePattern::Type(Type::Enum(ty)))))
			}
		}
	}
}

impl<R: 'static + Clone + Ord + Hash> Build<R> for ast::ListTypeExpr {
	type Target = ty::List<TypePatternRef>;

	fn build(
		&self,
		rdf: &mut impl RdfContextMut<R>,
		context: &mut impl Context<R>,
		scope: &Scope
	) -> Result<Self::Target, Error> {
		Ok(ty::List {
			prefix: self.prefix.iter().map(|e| e.build(rdf, context, scope)).collect::<Result<_, _>>()?,
			rest: self.rest.as_ref().map(|r| r.type_.build(rdf, context, scope)).transpose()?
		})
	}
}

impl<R: 'static + Clone + Ord + Hash> Build<R> for ast::MapTypeExpr {
	type Target = ty::Struct<TypePatternRef>;

	fn build(
		&self,
		rdf: &mut impl RdfContextMut<R>,
		context: &mut impl Context<R>,
		scope: &Scope
	) -> Result<Self::Target, Error> {
		Ok(ty::Struct {
			fields: self.entries.iter().map(|f| {
				Ok((
					f.key.build(rdf, context, scope)?,
					ty::Field {
						type_: f.value.build(rdf, context, scope)?,
						required: false
					}
				))
			}).collect::<Result<_, _>>()?
		})
	}
}

impl<R: 'static + Clone + Ord + Hash> Build<R> for ast::EnumTypeExpr {
	type Target = ty::Enum<TypePatternRef>;

	fn build(
		&self,
		rdf: &mut impl RdfContextMut<R>,
		context: &mut impl Context<R>,
		scope: &Scope
	) -> Result<Self::Target, Error> {
		Ok(ty::Enum {
			variants: self.variants.iter().map(|v| {
				Ok((v.ident.as_str().to_owned(), v.type_.build(rdf, context, scope)?))
			}).collect::<Result<_, _>>()?
		})
	}
}

impl<R: 'static + Clone + Ord + Hash> Build<R> for ast::Expr {
	type Target = Expr<R, TypePatternRef>;

	fn build(&self, rdf: &mut impl RdfContextMut<R>, context: &mut impl Context<R>, scope: &Scope) -> Result<Self::Target, Error> {
		let (bound, inner, type_) = match self {
			Self::Let(_, bindings, _, inner) => {
				let (bound, sub_scope) = bindings.build(rdf, context, scope)?;
				let (inner, type_) = inner.build(rdf, context, &sub_scope)?;
				(bound, inner, type_)
			}
			Self::Inner(inner) => {
				let (inner, type_) = inner.build(rdf, context, scope)?;
				(expr::Bound::default(), inner, type_)
			}
		};

		Ok(Expr {
			type_,
			bound,
			inner
		})
	}
}

impl ast::Bindings {
	fn build<'a, R: 'static + Clone + Ord + Hash>(&self, rdf: &mut impl RdfContextMut<R>, context: &mut impl Context<R>, scope: &'a Scope) -> Result<(expr::Bound<R, TypePatternRef>, Scope<'a>), Error> {
		let mut intro = Vec::with_capacity(self.variables.len());
		let dataset = DatasetPattern::new();
		let mut sub_scope = Scope::new(Some(scope));

		for ident in &self.variables {
			let ty_ref = context.new_type(Some(TypePattern::resource()));
			intro.push(ty_ref);
			sub_scope.insert(ident, ty_ref);
		}
		
		if let Some(b) = &self.bound {
			b.dataset.build(rdf, context, &sub_scope)?;
		}

		Ok((
			expr::Bound {
				intro,
				dataset
			},
			sub_scope
		))
	}
}

impl<R: Clone + Ord> Build<R> for ast::RdfDataset {
	type Target = DatasetPattern<R>;

	fn build(
		&self,
		rdf: &mut impl RdfContextMut<R>,
		context: &mut impl Context<R>,
		scope: &Scope
	) -> Result<Self::Target, Error> {
		let mut result = DatasetPattern::new();

		for statement in &self.statements {
			statement.build(rdf, context, scope, &mut result)?;
		}

		Ok(result)
	}
}

impl ast::RdfStatement {
	fn build<R: Clone + Ord>(
		&self,
		rdf: &mut impl RdfContextMut<R>,
		context: &mut impl Context<R>,
		scope: &Scope,
		result: &mut DatasetPattern<R>
	) -> Result<(), Error> {
		let subject = self.subject.build(rdf, context, scope)?;
		
		for po in &self.predicates_objects {
			po.build(rdf, context, scope, result, &subject)?;
		}

		Ok(())
	}
}

impl ast::RdfPredicateObjects {
	fn build<R: Clone + Ord>(
		&self,
		rdf: &mut impl RdfContextMut<R>,
		context: &mut impl Context<R>,
		scope: &Scope,
		result: &mut DatasetPattern<R>,
		subject: &TermPattern<R>
	) -> Result<(), Error> {
		let predicate = self.predicate.build(rdf, context, scope)?;
		
		for o in &self.objects {
			let object = o.build(rdf, context, scope)?;
			result.insert(Quad(
				subject.clone(),
				predicate.clone(),
				object,
				None
			));
		}

		Ok(())
	}
}

impl ast::RdfTerm {
	fn build<R>(
		&self,
		rdf: &mut impl RdfContextMut<R>,
		context: &mut impl Context<R>,
		scope: &Scope
	) -> Result<TermPattern<R>, Error> {
		match self {
			Self::Var(ident) => {
				scope.index_of(ident).map(TermPattern::Var)
			}
			Self::Iri(i) => {
				let iri = i.build(context)?;
				Ok(TermPattern::Resource(rdf.insert_iri(Cow::Owned(iri))))
			}
			Self::Literal(l) => {
				l.build(rdf, context).map(TermPattern::Resource)
			}
		}
	}
}

impl ast::RdfLiteral {
	fn build<R>(&self, rdf: &mut impl RdfContextMut<R>, context: &impl Context<R>) -> Result<R, Error> {
		let value = self.value.decode()?;
		match &self.annotation {
			None => {
				Ok(rdf.insert_literal(value, Cow::Borrowed(XSD_STRING)))
			}
			Some(ast::RdfLiteralAnnotation::Type(_, iri)) => {
				let iri = iri.build(context)?;
				Ok(rdf.insert_literal(value, Cow::Owned(iri)))
			}
			Some(ast::RdfLiteralAnnotation::Language(_, lang_literal)) => {
				let lang = lang_literal.decode()?;
				let tag = LangTagBuf::new(lang)
					.map_err(|InvalidLangTag(t)| Error::Language(lang_literal.span(), t))?;
				Ok(rdf.insert_lang_string(value, tag))
			}
		}
	}
}

impl ast::Iri {
	fn build<R>(&self, context: &impl Context<R>) -> Result<IriBuf, Error> {
		match self {
			Self::Full(iri) => {
				iri.decode()
			}
			Self::Compact(compact_iri) => {
				compact_iri.build(context)
			}
		}
	}
}

impl ast::FullIri {
	fn decode(&self) -> Result<IriBuf, Error> {
		let len = self.as_str().len();
		let content = &self.as_str()[1..(len-1)];
		IriBuf::new(content.to_owned()).map_err(|InvalidIri(i)| Error::Iri(self.span(), i))
	}
}

impl ast::CompactIri {
	fn build<R>(&self, context: &impl Context<R>) -> Result<IriBuf, Error> {
		let (prefix_name, suffix) = self
			.as_str()
			.split_once(':')
			.unwrap();
		let prefix = context
			.get_prefix(prefix_name)?
			.ok_or_else(|| Error::UnknownPrefix(self.span(), prefix_name.to_owned()))?;
		IriBuf::new(format!("{prefix}{suffix}"))
			.map_err(|InvalidIri(i)| Error::Iri(self.span(), i))
	}
}

impl<R: 'static + Clone + Ord + Hash> Build<R> for ast::InnerExpr {
	type Target = (InnerExpr<R, TypePatternRef>, TypePatternRef);

	fn build(&self, rdf: &mut impl RdfContextMut<R>, context: &mut impl Context<R>, scope: &Scope) -> Result<Self::Target, Error> {
		match self {
			Self::Var(ident) => {
				let x = scope.index_of(ident)?;
				let type_ = scope.type_of(x).unwrap();
				Ok((InnerExpr::Var(x), type_))
			},
			Self::Literal(expr) => {
				let (e, ty) = expr.build(rdf, context, scope)?;
				let ty = context.new_type(Some(TypePattern::Type(Type::Literal(ty))));
				Ok((InnerExpr::Literal(e), ty))
			},
			Self::List(expr) => {
				let (e, ty) = expr.build(rdf, context, scope)?;
				let ty = context.new_type(Some(TypePattern::Type(Type::List(ty))));
				Ok((InnerExpr::List(e), ty))
			},
			Self::Map(expr) => {
				let (e, ty) = expr.build(rdf, context, scope)?;
				let ty = context.new_type(Some(TypePattern::Type(Type::Struct(ty))));
				Ok((InnerExpr::Map(e), ty))
			},
			Self::Match(expr) => {
				let (e, ty) = expr.build(rdf, context, scope)?;
				let ty = context.new_type(Some(TypePattern::Type(Type::Enum(ty))));
				Ok((InnerExpr::Match(e), ty))
			},
			Self::Call(expr) => {
				let (e, ty) = expr.build(rdf, context, scope)?;
				Ok((InnerExpr::Call(e), ty))
			}
		}
	}
}

impl<R: 'static + Clone + Ord + Hash> Build<R> for ast::ListExpr {
	type Target = (expr::List<R, TypePatternRef>, ty::List<TypePatternRef>);

	fn build(&self, rdf: &mut impl RdfContextMut<R>, context: &mut impl Context<R>, scope: &Scope) -> Result<Self::Target, Error> {
		match &self.desc {
			ListDescription::Given(list) => {
				let mut items = Vec::with_capacity(list.len());
				let mut types = Vec::with_capacity(list.len());

				for item in list {
					let e = item.build(rdf, context, scope)?;
					types.push(e.type_);
					items.push(e);
				}

				Ok((expr::List::Explicit(expr::ExplicitList::new(items)), ty::List::from_prefix(types)))
			}
			ListDescription::All(_, bindings, mapping) => {
				let (bound, sub_scope) = bindings.build(rdf, context, scope)?;

				match bound.intro.as_slice() {
					&[item_ty] => {
						let body = Box::new(match mapping {
							Some(m) => {
								m.expr.build(rdf, context, &sub_scope)?
							}
							None => {
								Expr {
									type_: item_ty,
									bound: Bound::default(),
									inner: InnerExpr::Var(sub_scope.offset())
								}
							}
						});

						context.set_type(item_ty, TypePattern::Var(body.type_));

						let e = expr::List::Implicit(expr::ImplicitList::Unordered(expr::UnorderedList {
							bound,
							body
						}));
		
						Ok((e, ty::List::uniform(item_ty)))
					}
					_ => Err(Error::ExpectedExactlyOneBinding(self.span()))
				}
			}
			ListDescription::List(_, bindings, _, head, mapping) => {
				let (bound, sub_scope) = bindings.build(rdf, context, scope)?;

				match bound.intro.as_slice() {
					&[item_ty] => {
						let body = Box::new(match mapping {
							Some(m) => {
								m.expr.build(rdf, context, &sub_scope)?
							}
							None => {
								Expr {
									type_: item_ty,
									bound: Bound::default(),
									inner: InnerExpr::Var(sub_scope.offset())
								}
							}
						});

						context.set_type(item_ty, TypePattern::Var(body.type_));

						let e = expr::List::Implicit(expr::ImplicitList::Ordered(expr::OrderedList {
							components: ListComponents {
								first: rdf.insert_iri(Cow::Borrowed(RDF_FIRST)),
								rest: rdf.insert_iri(Cow::Borrowed(RDF_REST)),
								nil: rdf.insert_iri(Cow::Borrowed(RDF_NIL))
							},
							head: Box::new(head.build(rdf, context, scope)?),
							bound,
							body
						}));
		
						Ok((e, ty::List::uniform(item_ty)))
					}
					_ => Err(Error::ExpectedExactlyOneBinding(self.span()))
				}
			}
		}
	}
}

impl<R: 'static + Clone + Ord + Hash> Build<R> for ast::MapExpr {
	type Target = (expr::Map<R, TypePatternRef>, ty::Struct<TypePatternRef>);

	fn build(&self, rdf: &mut impl RdfContextMut<R>, context: &mut impl Context<R>, scope: &Scope) -> Result<Self::Target, Error> {
		let mut fields = BTreeMap::new();
		let mut entries = TypedMap::new();

		for entry in &self.entries {
			let key = entry.key.build(rdf, context, scope)?;
			let value = entry.value.build(rdf, context, scope)?;

			fields.insert(key.type_, ty::Field {
				type_: value.type_,
				required: true
			});

			entries.insert(key, value);
		}
		
		let e = expr::Map {
			entries
		};

		let t = ty::Struct {
			fields
		};
		
		Ok((e, t))
	}
}

impl<R: 'static + Clone + Ord + Hash> Build<R> for ast::ConstValue {
	type Target = TypedValue<R, TypePatternRef>;

	fn build(
		&self,
		rdf: &mut impl RdfContextMut<R>,
		context: &mut impl Context<R>,
		scope: &Scope
	) -> Result<Self::Target, Error> {
		let (inner, type_) = match self {
			Self::Literal(expr) => {
				let (e, ty) = expr.build(rdf, context, scope)?;
				let ty = context.new_type(Some(TypePattern::Type(Type::Literal(ty))));
				(TypedValueInnerDesc::Literal(e), ty)
			},
			Self::List(expr) => {
				let (e, ty) = expr.build(rdf, context, scope)?;
				let ty = context.new_type(Some(TypePattern::Type(Type::List(ty))));
				(TypedValueInnerDesc::List(e), ty)
			},
			Self::Map(expr) => {
				let (e, ty) = expr.build(rdf, context, scope)?;
				let ty = context.new_type(Some(TypePattern::Type(Type::Struct(ty))));
				(TypedValueInnerDesc::Map(e), ty)
			}
		};

		Ok(TypedValue {
			type_,
			desc: TypedValueDesc::Constant(inner)
		})
	}
}

impl<R: 'static + Clone + Ord + Hash> Build<R> for ast::ConstList {
	type Target = (Vec<TypedValue<R, TypePatternRef>>, ty::List<TypePatternRef>);

	fn build(&self, rdf: &mut impl RdfContextMut<R>, context: &mut impl Context<R>, scope: &Scope) -> Result<Self::Target, Error> {
		let mut items = Vec::with_capacity(self.items.len());
		let mut types = Vec::with_capacity(self.items.len());

		for item in &self.items {
			let e = item.build(rdf, context, scope)?;
			types.push(e.type_);
			items.push(e);
		}

		Ok((items, ty::List::from_prefix(types)))
	}
}

impl<R: 'static + Clone + Ord + Hash> Build<R> for ast::ConstMap {
	type Target = (TypedMap<R, TypedValue<R, TypePatternRef>, TypePatternRef>, ty::Struct<TypePatternRef>);

	fn build(&self, rdf: &mut impl RdfContextMut<R>, context: &mut impl Context<R>, scope: &Scope) -> Result<Self::Target, Error> {
		let mut fields = BTreeMap::new();
		let mut entries = TypedMap::new();

		for entry in &self.entries {
			let key = entry.key.build(rdf, context, scope)?;
			let value = entry.value.build(rdf, context, scope)?;

			fields.insert(key.type_, ty::Field {
				type_: value.type_,
				required: true
			});

			entries.insert(key, value);
		}

		let t = ty::Struct {
			fields
		};
		
		Ok((entries, t))
	}
}

impl<R: 'static + Clone + Ord + Hash> Build<R> for ast::MatchExpr {
	type Target = (expr::Match<R, TypePatternRef>, ty::Enum<TypePatternRef>);

	fn build(&self, rdf: &mut impl RdfContextMut<R>, context: &mut impl Context<R>, scope: &Scope) -> Result<Self::Target, Error> {
		let mut t = ty::Enum {
			variants: BTreeMap::new()
		};

		let mut e = expr::Match {
			cases: BTreeMap::new(),
			order: Vec::with_capacity(self.cases.len())
		};

		for case in &self.cases {
			let value = case.value.build(rdf, context, scope)?;
			t.variants.insert(case.name.as_str().to_owned(), value.type_);
			e.cases.insert(case.name.as_str().to_owned(), value);
			e.order.push(case.name.as_str().to_owned());
		}

		Ok((e, t))
	}
}

impl<R: 'static + Clone + Ord + Hash> Build<R> for ast::CallExpr {
	type Target = (expr::Call<R, TypePatternRef>, TypePatternRef);
	
	fn build(&self, rdf: &mut impl RdfContextMut<R>, context: &mut impl Context<R>, scope: &Scope) -> Result<Self::Target, Error> {
		let (function, signature) = context
			.get_function(self.function.as_str())
			.map(|(f, s)| (f.clone(), s.clone()))
			.ok_or_else(|| Error::UnknownFunction(self.function.clone()))?;

		if self.args.len() != signature.args.len() {
			return Err(Error::ArgumentCount {
				expected: signature.args.len(),
				found: self.args.len()
			})
		}

		let args = self
			.args
			.iter()
			.zip(&signature.args)
			.map(|(a, ty)| {
				let e = a.build(rdf, context, scope)?;
				context.set_type(e.type_, TypePattern::Var(*ty));
				Ok(e)
			})
			.collect::<Result<_, _>>()?;
		
		let e = expr::Call {
			function,
			args
		};

		let t = context.new_type(None);
		context.set_type_min(t, TypePattern::Var(signature.return_type));

		signature.return_type;

		Ok((e, t))
	}
}

impl<R: 'static + Clone + Ord + Hash> Build<R> for ast::LiteralExpr {
	type Target = (Literal, ty::LiteralType);

	// fn preferred_type(&self, rdf: &mut impl RdfContextMut<R>, context: &mut impl Context<R>, scope: &Scope) -> Result<Self::Type, Error> {
	// 	match self {
	// 		Self::Unit(_) => Ok(ty::LiteralType::Unit),
	// 		Self::True(b) => b.preferred_type(rdf, context, scope).map(ty::LiteralType::Boolean),
	// 		Self::False(b) => b.preferred_type(rdf, context, scope).map(ty::LiteralType::Boolean),
	// 		Self::Number(n) => n.preferred_type(rdf, context, scope).map(ty::LiteralType::Number),
	// 		Self::TextString(s) => s.preferred_type(rdf, context, scope).map(ty::LiteralType::TextString)
	// 	}
	// }

	fn build(&self, rdf: &mut impl RdfContextMut<R>, context: &mut impl Context<R>, scope: &Scope) -> Result<Self::Target, Error> {
		// match (self, ty) {
		// 	(Self::Unit(_), ty::LiteralType::Unit) => Ok(Literal::Unit),
		// 	(Self::True(b), ty::LiteralType::Boolean(ty)) => b.build_typed(rdf, context, scope, ty).map(Literal::Boolean),
		// 	(Self::False(b), ty::LiteralType::Boolean(ty)) => b.build_typed(rdf, context, scope, ty).map(Literal::Boolean),
		// 	(Self::Number(n), ty::LiteralType::Number(ty)) => n.build_typed(rdf, context, scope, ty).map(Literal::Number),
		// 	(Self::TextString(s), ty::LiteralType::TextString(ty)) => s.build_typed(rdf, context, scope, ty).map(Literal::TextString),
		// 	_ => Err(Error::TypeMismatch(self.span()))
		// }

		// match self {
		// 	Self::Unit(_) => Ok(ty::LiteralType::Unit),
		// 	Self::True(b) => b.build(rdf, context, scope).map(ty::LiteralType::Boolean),
		// 	Self::False(b) => b.build(rdf, context, scope).map(ty::LiteralType::Boolean),
		// 	Self::Number(n) => n.build(rdf, context, scope).map(ty::LiteralType::Number),
		// 	Self::TextString(t) => t.build(rdf, context, scope).map(ty::LiteralType::TextString),
		// }

		todo!()
	}
}

impl<R: 'static + Clone + Ord + Hash> Build<R> for ast::True {
	type Target = (bool, ty::Boolean);

	// fn preferred_type(&self, rdf: &mut impl RdfContextMut<R>, context: &mut impl Context<R>, scope: &Scope) -> Result<Self::Type, Error> {
	// 	self.build(rdf, context, scope)
	// }

	fn build(
		&self,
		_rdf: &mut impl RdfContextMut<R>,
		_context: &impl Context<R>,
		_scope: &Scope
	) -> Result<Self::Target, Error> {
		// Ok(true)

		// Ok(ty::Boolean::singleton(true))

		todo!()
	}
}

impl<R: 'static + Clone + Ord + Hash> Build<R> for ast::False {
	type Target = (bool, ty::Boolean);

	// fn preferred_type(&self, rdf: &mut impl RdfContextMut<R>, context: &mut impl Context<R>, scope: &Scope) -> Result<Self::Type, Error> {
	// 	self.build(rdf, context, scope)
	// }

	fn build(
		&self,
		_rdf: &mut impl RdfContextMut<R>,
		_context: &impl Context<R>,
		_scope: &Scope
	) -> Result<Self::Target, Error> {
		// Ok(false)

		// Ok(ty::Boolean::singleton(false))

		todo!()
	}
}

impl<R: 'static + Clone + Ord + Hash> Build<R> for ast::Number {
	type Target = (Number, ty::Number);

	// fn preferred_type(&self, rdf: &mut impl RdfContextMut<R>, context: &mut impl Context<R>, scope: &Scope) -> Result<Self::Type, Error> {
	// 	self.build(rdf, context, scope)
	// }
	
	fn build(&self, _rdf: &mut impl RdfContextMut<R>, _context: &impl Context<R>, _scope: &Scope) -> Result<Self::Target, Error> {
		// self.as_str().parse().map_err(|e| Error::Number(self.span(), e))
		
		// Ok(ty::Number::singleton(
		// 	self
		// 		.as_str()
		// 		.parse()
		// 		.map_err(|e| Error::Number(self.span(), e))?
		// ))
		
		todo!()
	}
}

impl ast::TextString {
	pub fn decode(&self) -> Result<String, Error> {
		let s = self.as_str();
		let content = &s[1..(s.len() - 1)];
		let mut chars = content.chars();
		let mut value = String::with_capacity(content.len());

		while let Some(c) = chars.next() {
			let decoded_c = match c {
				'\\' => {
					match chars.next() {
						Some('x') => {
							let a = chars
								.next()
								.and_then(|c| c.to_digit(16))
								.ok_or_else(|| Error::String(self.span()))?;

							let b = chars
								.next()
								.and_then(|c| c.to_digit(16))
								.ok_or_else(|| Error::String(self.span()))?;

							char::from_u32(a << 4 | b)
								.ok_or_else(|| Error::String(self.span()))?
						}
						Some('u' | 'U') => {
							match chars.next() {
								Some('{') => {
									let mut value = 0;
									let mut i = 0;

									loop {
										match chars.next() {
											Some('}') if i >= 2 => {
												break
											}
											Some(d) if i < 6 => {
												value = value << 4 | d
													.to_digit(16)
													.ok_or_else(|| Error::String(self.span()))?;
												i += 1
											}
											_ => return Err(Error::String(self.span()))
										}
									}

									char::from_u32(value)
										.ok_or_else(|| Error::String(self.span()))?
								}
								_ => return Err(Error::String(self.span()))
							}
						}
						Some('t') => '\t',
						Some('r') => '\r',
						Some('n') => '\n',
						Some('\'') => '\'',
						Some('"') => '"',
						Some('\\') => '\\',
						_ => return Err(Error::String(self.span()))
					}
				},
				c => c
			};

			value.push(decoded_c);
		}

		Ok(value)
	}
}

impl<R: 'static + Clone + Ord + Hash> Build<R> for ast::TextString {
	type Target = (String, ty::TextString);

	// fn preferred_type(&self, rdf: &mut impl RdfContextMut<R>, context: &mut impl Context<R>, scope: &Scope) -> Result<Self::Type, Error> {
	// 	self.build(rdf, context, scope)
	// }

	fn build(&self, _rdf: &mut impl RdfContextMut<R>, _context: &impl Context<R>, _scope: &Scope) -> Result<Self::Target, Error> {
		// self.decode()
		
		// self
		// 	.decode()
		// 	.map(ty::TextString::singleton)
		
		todo!()
	}
}