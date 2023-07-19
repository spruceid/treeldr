use crate::{path::Segment, syntax, Context, Error, GenerateSyntax, Path};
use proc_macro2::TokenStream;
use rdf_types::Vocabulary;
use shelves::Ref;
use std::collections::HashSet;
use treeldr::{BlankIdIndex, IriIndex, TId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TraitId {
	FromRdf,
	QuadsAndValues,
	AsJsonLd,
	IntoJsonLd,
	IntoJsonLdSyntax,
	Class(TId<treeldr::Type>),
}

impl TraitId {
	pub fn impl_for(self, ty: TId<treeldr::Layout>) -> TraitImpl {
		TraitImpl::new(ty, self)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TraitImpl {
	pub ty: TId<treeldr::Layout>,
	pub tr: TraitId,
}

impl TraitImpl {
	pub fn new(ty: TId<treeldr::Layout>, tr: TraitId) -> Self {
		Self { ty, tr }
	}
}

/// Module visibility.
pub enum Visibility {
	Public,
	Inherited,
}

impl From<Visibility> for syn::Visibility {
	fn from(vis: Visibility) -> syn::Visibility {
		match vis {
			Visibility::Public => Self::Public(syn::token::Pub::default()),
			Visibility::Inherited => Self::Inherited,
		}
	}
}

/// Module extern path.
///
/// ```text
/// path::to::extern::module
/// ```
#[derive(Clone)]
pub struct ExternPath {
	root: proc_macro2::Ident,
	rest: Vec<(syn::token::PathSep, proc_macro2::Ident)>,
}

impl ExternPath {
	pub fn new(root: proc_macro2::Ident) -> Self {
		Self {
			root,
			rest: Vec::new(),
		}
	}

	pub fn ident(&self) -> &proc_macro2::Ident {
		match self.rest.last() {
			Some((_, ident)) => ident,
			None => &self.root,
		}
	}

	pub fn push(&mut self, sep: syn::token::PathSep, ident: proc_macro2::Ident) {
		self.rest.push((sep, ident))
	}
}

/// Module extern path with optional renaming.
///
/// ```text
/// path::to::extern::module as renaming
/// ```
pub struct ExternPathWithRenaming {
	pub path: ExternPath,
	pub renaming: Option<(syn::token::As, proc_macro2::Ident)>,
}

impl ExternPathWithRenaming {
	pub fn into_path_and_ident(self) -> (ExternPath, proc_macro2::Ident) {
		match self.renaming {
			Some((_, ident)) => (self.path, ident),
			None => {
				let ident = self.path.ident().clone();
				(self.path, ident)
			}
		}
	}
}

/// Error raised when parsing an invalid module extern path.
#[derive(Debug, thiserror::Error)]
#[error("invalid extern module path")]
pub struct InvalidExternPath;

impl TryFrom<syn::UsePath> for ExternPathWithRenaming {
	type Error = InvalidExternPath;

	fn try_from(p: syn::UsePath) -> Result<Self, Self::Error> {
		let root = p.ident;
		let mut rest = Vec::new();
		let mut sep = p.colon2_token;
		let mut tree = *p.tree;

		loop {
			match tree {
				syn::UseTree::Path(p) => {
					rest.push((sep, p.ident));
					sep = p.colon2_token;
					tree = *p.tree;
				}
				syn::UseTree::Name(n) => {
					rest.push((sep, n.ident));
					break Ok(Self {
						path: ExternPath { root, rest },
						renaming: None,
					});
				}
				syn::UseTree::Rename(r) => {
					rest.push((sep, r.ident));
					let renaming = Some((r.as_token, r.rename));
					break Ok(Self {
						path: ExternPath { root, rest },
						renaming,
					});
				}
				_ => break Err(InvalidExternPath),
			}
		}
	}
}

impl quote::ToTokens for ExternPath {
	fn to_tokens(&self, tokens: &mut TokenStream) {
		self.root.to_tokens(tokens);
		for (sep, ident) in &self.rest {
			sep.to_tokens(tokens);
			ident.to_tokens(tokens);
		}
	}
}

/// TreeLDR generated module.
pub struct Module {
	parent: Option<Ref<Self>>,
	extern_path: Option<ExternPath>,
	ident: proc_macro2::Ident,
	visibility: syn::Visibility,
	sub_modules: HashSet<Ref<Self>>,
	layouts: HashSet<TId<treeldr::Layout>>,
	types: HashSet<TId<treeldr::Type>>,
	types_providers: HashSet<crate::tr::ProviderOf>,
	trait_impls: HashSet<TraitImpl>,
}

impl Module {
	pub fn new(
		parent: Option<Ref<Self>>,
		extern_path: Option<ExternPath>,
		ident: proc_macro2::Ident,
		visibility: impl Into<syn::Visibility>,
	) -> Self {
		Self {
			parent,
			extern_path,
			ident,
			visibility: visibility.into(),
			sub_modules: HashSet::new(),
			layouts: HashSet::new(),
			types: HashSet::new(),
			types_providers: HashSet::new(),
			trait_impls: HashSet::new(),
		}
	}

	pub fn ident(&self) -> &proc_macro2::Ident {
		&self.ident
	}

	pub fn visibility(&self) -> &syn::Visibility {
		&self.visibility
	}

	pub fn extern_path(&self) -> Option<&ExternPath> {
		self.extern_path.as_ref()
	}

	pub fn path<V, M>(&self, context: &Context<V, M>) -> Path {
		let mut path = context.module_path(self.parent);
		path.push(Segment::Ident(self.ident.clone()));
		path
	}

	pub fn sub_modules(&self) -> &HashSet<Ref<Self>> {
		&self.sub_modules
	}

	pub fn sub_modules_mut(&mut self) -> &mut HashSet<Ref<Self>> {
		&mut self.sub_modules
	}

	pub fn layouts(&self) -> &HashSet<TId<treeldr::Layout>> {
		&self.layouts
	}

	pub fn layouts_mut(&mut self) -> &mut HashSet<TId<treeldr::Layout>> {
		&mut self.layouts
	}

	pub fn types(&self) -> &HashSet<TId<treeldr::Type>> {
		&self.types
	}

	pub fn types_mut(&mut self) -> &mut HashSet<TId<treeldr::Type>> {
		&mut self.types
	}

	pub fn types_providers(&self) -> &HashSet<crate::tr::ProviderOf> {
		&self.types_providers
	}

	pub fn types_providers_mut(&mut self) -> &mut HashSet<crate::tr::ProviderOf> {
		&mut self.types_providers
	}

	pub fn trait_impls(&self) -> &HashSet<TraitImpl> {
		&self.trait_impls
	}

	pub fn trait_impls_mut(&mut self) -> &mut HashSet<TraitImpl> {
		&mut self.trait_impls
	}
}

impl<M> GenerateSyntax<M> for Module {
	type Output = syntax::ModuleContent;

	fn generate_syntax<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: &crate::Scope,
	) -> Result<Self::Output, Error> {
		let mut items = Vec::new();

		for module_ref in &self.sub_modules {
			items.push(syntax::ModuleItem::Module(
				module_ref.generate_syntax(context, scope)?,
			));
		}

		for type_ref in &self.types {
			let tr = context.type_trait(*type_ref).expect("undefined class");
			items.push(syntax::ModuleItem::Trait(syntax::TraitDefinition::Class(
				tr.generate_syntax(context, scope)?,
			)));
		}

		for provider in &self.types_providers {
			items.push(syntax::ModuleItem::Trait(
				syntax::TraitDefinition::ClassProvider(provider.generate_syntax(context, scope)?),
			));
		}

		for layout_ref in &self.layouts {
			let ty = context.layout_type(*layout_ref).expect("undefined layout");
			if let Some(def) = ty.generate_syntax(context, scope)? {
				items.push(syntax::ModuleItem::Type(syntax::TypeDefinition::Layout(
					def,
					doc_attributes(ty.label(), ty.documentation()),
				)));
			}
		}

		for trait_impl in &self.trait_impls {
			if let Some(i) = trait_impl.generate_syntax(context, scope)? {
				items.push(syntax::ModuleItem::TraitImpl(i));
			}
		}

		Ok(syntax::ModuleContent { items })
	}
}

pub fn doc_attributes(label: Option<&str>, doc: &treeldr::StrippedDocumentation) -> Vec<String> {
	let mut content = String::new();

	if let Some(label) = label {
		content.push_str(label)
	}

	if let Some(short) = doc.short_description() {
		if !content.is_empty() {
			content.push_str("\n\n");
		}

		content.push_str(short)
	}

	if let Some(long) = doc.long_description() {
		if !content.is_empty() {
			content.push_str("\n\n");
		}

		content.push_str(long)
	}

	content.lines().map(str::to_string).collect()
}

impl<M> GenerateSyntax<M> for Ref<Module> {
	type Output = syntax::ModuleOrUse;

	fn generate_syntax<V: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>>(
		&self,
		context: &Context<V, M>,
		scope: &crate::Scope,
	) -> Result<Self::Output, Error> {
		let module = context.module(*self).expect("undefined module");
		let ident = module.ident();
		let vis = module.visibility();

		match module.extern_path() {
			Some(path) => {
				let ident = if path.ident() == ident {
					None
				} else {
					Some(ident.clone())
				};

				Ok(syntax::ModuleOrUse::Use(syntax::Use {
					vis: vis.clone(),
					path: path.clone(),
					ident,
				}))
			}
			None => {
				let mut scope = scope.clone();
				scope.module = Some(*self);

				Ok(syntax::ModuleOrUse::Module(syntax::Module {
					vis: vis.clone(),
					ident: ident.clone(),
					content: module.generate_syntax(context, &scope)?,
				}))
			}
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub enum Parent {
	/// The parent module is unreachable.
	Extern,
	Ref(Ref<Module>),
}

impl Parent {
	pub fn into_ref(self) -> Option<Ref<Module>> {
		match self {
			Self::Extern => None,
			Self::Ref(r) => Some(r),
		}
	}
}
