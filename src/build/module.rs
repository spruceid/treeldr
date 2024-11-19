use iref::{Iri, IriBuf};
use ll1::Spanned;
use std::{collections::HashMap, hash::Hash};

use crate::{
	function::FunctionRef, module::Item, rdf::RdfContextMut, syntax::ast, ItemPath,
	ItemPathSegment, ItemRef, Module, ModuleRef, Registry, TypeRef,
};

use super::{
	Build, BuildContext, BuildSignature, BuildingNamespace, Error, ExprScope, Scope, TypeValue,
	TypeVariableRef, TypeVariables,
};

pub struct BuildModule<'ast> {
	self_: ModuleRef,

	metadata: Option<&'ast ast::Metadata>,

	parent: Option<ModuleRef>,

	/// Items.
	items: HashMap<String, BuildItem>,
}

impl<'ast> BuildModule<'ast> {
	pub fn get<'a, R>(
		&'a self,
		registry: &'a Registry<R>,
		namespace: &'a BuildingNamespace<'ast>,
		path: &ItemPath,
	) -> Option<ScopeItemRef<'a>> {
		match path.split_first() {
			Some((first, rest)) => self
				.get_by_segment(first)
				.and_then(|item| item.get(registry, namespace, ItemPath::from_slice(rest))),
			None => Some(ScopeItemRef::Module(self.self_)),
		}
	}

	fn get_by_segment(&self, segment: &ItemPathSegment) -> Option<ScopeItemRef<'_>> {
		match segment {
			ItemPathSegment::Super => self.parent.map(ScopeItemRef::Module),
			ItemPathSegment::Item(name) => self.get_by_name(name),
		}
	}

	fn get_by_name(&self, name: &str) -> Option<ScopeItemRef<'_>> {
		self.items.get(name).map(BuildItem::as_ref)
	}
}

impl<'ast> BuildModule<'ast> {
	pub fn build<R, D>(
		&self,
		context: &mut BuildContext<R, D>,
		namespace: &BuildingNamespace,
	) -> Result<Module<R, TypeVariableRef>, Error>
	where
		R: Clone + Ord,
		D: RdfContextMut<R>,
	{
		let mut result = Module::default();

		let scope = Scope {
			module: self.parent,
			expr: ExprScope::new(None),
			const_: false,
		};

		result.attributes = self
			.metadata
			.map(|m| m.build(context, namespace, &scope))
			.transpose()?
			.unwrap_or_default();

		for (name, item) in &self.items {
			match item {
				BuildItem::Prefix(i) => {
					result.insert(name, Item::Prefix(i.clone()));
				}
				BuildItem::Type(x) => {
					result.insert(name, Item::Type(*x));
				}
				BuildItem::Function(f_ref) => {
					result.insert(name, Item::Function(*f_ref));
				}
				BuildItem::Module(m) => {
					result.insert(name, Item::Module(*m));
				}
			}
		}

		Ok(result)
	}
}

#[derive(Debug)]
pub enum BuildItem {
	Prefix(IriBuf),
	Type(TypeVariableRef),
	Function(FunctionRef),
	Module(ModuleRef),
}

impl BuildItem {
	fn as_ref(&self) -> ScopeItemRef<'_> {
		match self {
			Self::Prefix(p) => ScopeItemRef::Prefix(p),
			Self::Type(r) => ScopeItemRef::TypeVariable(*r),
			Self::Function(r) => ScopeItemRef::Function(*r),
			Self::Module(m) => ScopeItemRef::Module(*m),
		}
	}
}

#[derive(Debug, Clone)]
pub enum ScopeItem {
	Prefix(IriBuf),
	Variable(u32),
	Type(TypeRef),
	TypeVariable(TypeVariableRef),
	Function(FunctionRef),
	Module(ModuleRef),
}

impl ScopeItem {
	pub fn import_type<R>(
		&self,
		registry: &mut Registry<R>,
		type_variables: &mut TypeVariables<R>,
	) -> Option<TypeVariableRef> {
		match self {
			Self::Type(t) => Some(type_variables.import_type_ref(registry, *t)),
			Self::TypeVariable(t) => Some(*t),
			_ => None,
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub enum ScopeItemRef<'a> {
	Prefix(&'a Iri),
	Variable(u32),
	Type(TypeRef),
	TypeVariable(TypeVariableRef),
	Function(FunctionRef),
	Module(ModuleRef),
}

impl<'a> ScopeItemRef<'a> {
	pub fn get<R>(
		&self,
		registry: &'a Registry<R>,
		namespace: &'a BuildingNamespace,
		path: &ItemPath,
	) -> Option<Self> {
		match path.split_first() {
			Some((first, rest)) => self
				.get_by_segment(registry, namespace, first)
				.and_then(|i| i.get(registry, namespace, ItemPath::from_slice(rest))),
			None => Some(*self),
		}
	}

	pub fn get_by_segment<R>(
		&self,
		registry: &'a Registry<R>,
		namespace: &'a BuildingNamespace,
		segment: &ItemPathSegment,
	) -> Option<Self> {
		match self {
			Self::Module(m) => {
				if m.namespace == namespace.id {
					namespace
						.get_module(m.index)
						.and_then(|m| m.get_by_segment(segment))
				} else {
					m.get_by_segment(registry, segment).map(Into::into)
				}
			}
			_ => None,
		}
	}

	pub fn get_by_name<R>(
		&self,
		registry: &'a Registry<R>,
		namespace: &'a BuildingNamespace,
		name: &str,
	) -> Option<Self> {
		match self {
			Self::Module(m) => {
				if m.namespace == namespace.id {
					namespace
						.get_module(m.index)
						.and_then(|m| m.get_by_name(name))
				} else {
					m.get_by_name(registry, name).map(Into::into)
				}
			}
			_ => None,
		}
	}

	pub fn as_prefix(&self) -> Option<&'a str> {
		match self {
			Self::Prefix(p) => Some(p),
			_ => None,
		}
	}

	pub fn as_variable(&self) -> Option<u32> {
		match self {
			Self::Variable(x) => Some(*x),
			_ => None,
		}
	}

	pub fn as_function(&self) -> Option<FunctionRef> {
		match self {
			Self::Function(f) => Some(*f),
			_ => None,
		}
	}

	pub fn into_owned(self) -> ScopeItem {
		match self {
			Self::Prefix(v) => ScopeItem::Prefix(v.to_owned()),
			Self::Type(v) => ScopeItem::Type(v),
			Self::TypeVariable(v) => ScopeItem::TypeVariable(v),
			Self::Function(v) => ScopeItem::Function(v),
			Self::Module(v) => ScopeItem::Module(v),
			Self::Variable(v) => ScopeItem::Variable(v),
		}
	}
}

impl<'a> From<ItemRef<'a>> for ScopeItemRef<'a> {
	fn from(value: ItemRef<'a>) -> Self {
		match value {
			ItemRef::Prefix(i) => ScopeItemRef::Prefix(i),
			ItemRef::Type(t) => ScopeItemRef::Type(t),
			ItemRef::Function(f) => ScopeItemRef::Function(f),
			ItemRef::Module(m) => ScopeItemRef::Module(m),
		}
	}
}

impl<'ast> ast::Module {
	pub fn declare<R, D>(
		&'ast self,
		context: &mut BuildContext<'_, R, D>,
		namespace: &mut BuildingNamespace<'ast>,
		scope: &Scope,
		metadata: Option<&'ast ast::Metadata>,
	) -> Result<ModuleRef, Error>
	where
		R: Clone + Ord + Hash,
		D: RdfContextMut<R>,
	{
		namespace.begin_module(|namespace, module_ref| {
			let mut result = BuildModule {
				self_: module_ref,
				metadata,
				parent: scope.module,
				items: HashMap::new(),
			};

			let scope = Scope {
				module: Some(module_ref),
				expr: ExprScope::new(None),
				const_: false,
			};

			for item in &self.items {
				match &item.value {
					ast::Item::Prefix(ast) => {
						let iri = ast.value.decode()?;
						result
							.items
							.insert(ast.ident.as_str().to_owned(), BuildItem::Prefix(iri));
					}
					ast::Item::Type(t_ast) => {
						let x = context
							.type_variables
							.fresh(TypeValue::Undefined, Some(t_ast.ident.span()));
						namespace.declare_type(module_ref, &item.metadata, t_ast, x);
						if let Some(item) = result
							.items
							.insert(t_ast.ident.as_str().to_owned(), BuildItem::Type(x))
						{
							return Err(Error::ItemRedefinition(
								t_ast.ident.clone(),
								item,
								BuildItem::Type(x),
							));
						}
					}
					ast::Item::Function(ast) => {
						let args = ast
							.args
							.iter()
							.map(|a| {
								context
									.type_variables
									.fresh(TypeValue::Undefined, Some(a.span()))
							})
							.collect();

						let signature = BuildSignature {
							const_args: vec![],
							args,
							return_type: context
								.type_variables
								.fresh(TypeValue::Undefined, Some(ast.ident.span())),
						};

						let f_ref =
							namespace.declare_function(module_ref, &item.metadata, ast, signature);
						if let Some(item) = result
							.items
							.insert(ast.ident.as_str().to_owned(), BuildItem::Function(f_ref))
						{
							return Err(Error::ItemRedefinition(
								ast.ident.clone(),
								item,
								BuildItem::Function(f_ref),
							));
						}
					}
					ast::Item::Module(m) => {
						let m_scope = scope.begin_sub_module(module_ref);
						let m_ref =
							m.value
								.declare(context, namespace, &m_scope, Some(&item.metadata))?;
						if let Some(item) = result
							.items
							.insert(m.ident.as_str().to_owned(), BuildItem::Module(m_ref))
						{
							return Err(Error::ItemRedefinition(
								m.ident.clone(),
								item,
								BuildItem::Module(m_ref),
							));
						}
					}
				}
			}

			Ok(result)
		})
	}
}
