use std::collections::HashMap;

use crate::{syntax::ast, ItemPath, ItemPathSegment, ModuleRef};

use super::{BuildContext, BuildEnvironment, BuildingPackage, ScopeItemRef, TypeVariableRef};

pub struct Scope<'a, P> {
	pub module: Option<ModuleRef<P>>,
	pub expr: ExprScope<'a>,
	pub const_: bool,
}

impl<P> Default for Scope<'_, P> {
	fn default() -> Self {
		Self {
			module: None,
			expr: ExprScope::new(None),
			const_: false,
		}
	}
}

impl<P> Scope<'_, P> {
	pub fn new() -> Self {
		Self::default()
	}
}

impl<'a, P: Copy + PartialEq> Scope<'a, P> {
	pub fn get<'c, E, D>(
		&self,
		context: &'c BuildContext<'c, E, D>,
		namespace: &'c BuildingPackage<P>,
		path: &ItemPath,
	) -> Option<ScopeItemRef<'c, P>>
	where
		E: BuildEnvironment<PackageId = P>,
	{
		match path.split_first() {
			Some((first, rest)) => self
				.get_by_segment(context, namespace, first)
				.and_then(|i| {
					i.get(
						context.environment.registry(),
						namespace,
						ItemPath::from_slice(rest),
					)
				}),
			None => self.module.map(ScopeItemRef::Module),
		}
	}

	pub fn get_by_segment<'c, E, D>(
		&self,
		context: &'c BuildContext<'c, E, D>,
		namespace: &'c BuildingPackage<P>,
		segment: &ItemPathSegment,
	) -> Option<ScopeItemRef<'c, P>>
	where
		E: BuildEnvironment<PackageId = P>,
	{
		if let ItemPathSegment::Item(name) = segment {
			if let Some(x) = self.expr.get_by_name(name) {
				return Some(ScopeItemRef::Variable(x));
			}
		}

		if let Some(m) = self.module {
			if let Some(item) = ScopeItemRef::Module(m).get_by_segment(
				context.environment.registry(),
				namespace,
				segment,
			) {
				return Some(item);
			}
		}

		match segment {
			ItemPathSegment::Super => None,
			ItemPathSegment::Item(name) => context.environment.get_by_name(name).map(Into::into),
		}
	}

	pub fn get_by_name<'c, E, D>(
		&self,
		context: &'c BuildContext<'c, E, D>,
		namespace: &'c BuildingPackage<P>,
		name: &str,
	) -> Option<ScopeItemRef<'c, P>>
	where
		E: BuildEnvironment<PackageId = P>,
	{
		if let Some(x) = self.expr.get_by_name(name) {
			return Some(ScopeItemRef::Variable(x));
		}

		if let Some(m) = self.module {
			if let Some(item) =
				ScopeItemRef::Module(m).get_by_name(context.environment.registry(), namespace, name)
			{
				return Some(item);
			}
		}

		context.environment.get_by_name(name).map(Into::into)
	}

	pub fn begin(&'a self) -> Scope<'a, P> {
		Self {
			module: self.module,
			expr: ExprScope::new(Some(&self.expr)),
			const_: self.const_,
		}
	}

	pub fn begin_const(&'a self) -> Scope<'a, P> {
		Self {
			module: self.module,
			expr: ExprScope::new(Some(&self.expr)),
			const_: true,
		}
	}

	pub fn begin_sub_module(&'a self, self_: ModuleRef<P>) -> Scope<'a, P> {
		Self {
			module: Some(self_),
			expr: ExprScope::new(None),
			const_: self.const_,
		}
	}
}

pub struct ExprScope<'a> {
	parent: Option<&'a ExprScope<'a>>,
	variables: Vec<(String, TypeVariableRef)>,
	bindings: HashMap<String, u32>,
	len: u32,
}

impl<'a> ExprScope<'a> {
	pub fn new(parent: Option<&'a ExprScope<'a>>) -> Self {
		Self {
			parent,
			variables: Vec::new(),
			bindings: HashMap::new(),
			len: parent.map(|p| p.len).unwrap_or_default(),
		}
	}

	pub fn offset(&self) -> u32 {
		self.parent.map(|p| p.len).unwrap_or_default()
	}

	pub fn is_empty(&self) -> bool {
		self.len == 0
	}

	pub fn len(&self) -> u32 {
		self.len
	}

	pub fn variable(&self, i: u32) -> Option<(&str, TypeVariableRef)> {
		let j = match self.parent {
			Some(p) => {
				if i < p.len {
					return p.variable(i);
				} else {
					i - p.len
				}
			}
			None => i,
		};

		self.variables
			.get(j as usize)
			.map(|(n, t)| (n.as_str(), *t))
	}

	pub fn get_by_name(&self, name: &str) -> Option<u32> {
		match self.bindings.get(name) {
			Some(&i) => Some(i),
			None => self.parent.and_then(|p| p.get_by_name(name)),
		}
	}

	pub fn type_of(&self, i: u32) -> Option<TypeVariableRef> {
		self.variable(i).map(|(_, ty)| ty)
	}

	pub fn insert(&mut self, ident: &ast::Ident, ty: TypeVariableRef) -> u32 {
		let i = self.len;
		self.len += 1;
		self.variables.push((ident.as_str().to_owned(), ty));
		self.bindings.insert(ident.as_str().to_owned(), i);
		i
	}
}
