use std::collections::{HashMap, HashSet};

use crate::{
	component,
	metadata::Merge,
	node::{self, BindingValueRef},
	prop::{self, PropertyName, UnknownProperty},
	property_values,
	vocab::{self, Rdfs, Term},
	BlankIdIndex, Id, IriIndex, Multiple, MutableModel, PropertyValueRef, PropertyValues, Ref,
	RequiredFunctionalPropertyValue, ResourceType, TId,
};

pub mod data;
mod intersection;
pub mod normal;
pub mod properties;
pub mod restriction;
mod r#union;

use contextual::DisplayWithContext;
pub use data::DataType;
use derivative::Derivative;
pub use intersection::Intersection;
use locspan::Meta;
pub use normal::Normal;
use once_cell::unsync::OnceCell;
pub use properties::{Properties, PseudoProperty};
use rdf_types::Vocabulary;
pub use restriction::{Restriction, Restrictions};
pub use union::Union;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct OtherTypeId(TId<Type>);

impl OtherTypeId {
	pub fn id(&self) -> TId<Type> {
		self.0
	}

	pub fn raw_id(&self) -> Id {
		self.0.id()
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Type {
	Resource(Option<node::Type>),
	Other(OtherTypeId),
}

impl Type {
	pub fn id(&self) -> TId<Type> {
		match self {
			Self::Resource(None) => {
				TId::new(Id::Iri(IriIndex::Iri(Term::Rdfs(vocab::Rdfs::Resource))))
			}
			Self::Resource(Some(ty)) => TId::new(Id::Iri(IriIndex::Iri(ty.term()))),
			Self::Other(ty) => ty.id(),
		}
	}

	pub fn into_id(self) -> TId<Type> {
		self.id()
	}

	pub fn raw_id(&self) -> Id {
		self.id().id()
	}

	pub fn into_raw_id(self) -> Id {
		self.into_id().into_id()
	}
}

impl ResourceType for Type {
	const TYPE: Type = Type::Resource(Some(node::Type::Class(None)));

	fn check<M>(resource: &crate::node::Definition<M>) -> bool {
		resource.is_type()
	}
}

impl<'a, M> Ref<'a, Type, M> {
	pub fn as_type(&self) -> &'a Meta<Definition<M>, M> {
		self.as_resource().as_type().unwrap()
	}
}

impl From<node::Type> for Type {
	fn from(ty: node::Type) -> Self {
		Self::Resource(Some(ty))
	}
}

impl From<SubClass> for Type {
	fn from(ty: SubClass) -> Self {
		Self::Resource(Some(node::Type::Class(Some(ty))))
	}
}

impl From<prop::Type> for Type {
	fn from(ty: prop::Type) -> Self {
		Self::Resource(Some(node::Type::Property(Some(ty))))
	}
}

impl From<component::Type> for Type {
	fn from(ty: component::Type) -> Self {
		Self::Resource(Some(node::Type::Component(Some(ty))))
	}
}

impl From<component::formatted::Type> for Type {
	fn from(ty: component::formatted::Type) -> Self {
		Self::Resource(Some(node::Type::Component(Some(
			component::Type::Formatted(Some(ty)),
		))))
	}
}

impl From<Term> for Type {
	fn from(t: Term) -> Self {
		match t {
			Term::Rdfs(vocab::Rdfs::Resource) => Self::Resource(None),
			Term::Rdfs(vocab::Rdfs::Class) => node::Type::Class(None).into(),
			Term::Rdfs(vocab::Rdfs::Datatype) => SubClass::DataType.into(),
			Term::Rdf(vocab::Rdf::Property) => node::Type::Property(None).into(),
			Term::Rdf(vocab::Rdf::List) => node::Type::List.into(),
			Term::Owl(vocab::Owl::Restriction) => SubClass::Restriction.into(),
			Term::Owl(vocab::Owl::FunctionalProperty) => prop::Type::FunctionalProperty.into(),
			Term::TreeLdr(vocab::TreeLdr::Component) => node::Type::Component(None).into(),
			Term::TreeLdr(vocab::TreeLdr::Layout) => component::Type::Layout.into(),
			Term::TreeLdr(vocab::TreeLdr::Formatted) => component::Type::Formatted(None).into(),
			Term::TreeLdr(vocab::TreeLdr::Field) => component::formatted::Type::LayoutField.into(),
			Term::TreeLdr(vocab::TreeLdr::Variant) => {
				component::formatted::Type::LayoutVariant.into()
			}
			t => Self::Other(OtherTypeId(TId::new(Id::Iri(IriIndex::Iri(t))))),
		}
	}
}

impl From<Id> for Type {
	fn from(id: Id) -> Self {
		match id {
			Id::Iri(IriIndex::Iri(t)) => t.into(),
			id => Self::Other(OtherTypeId(TId::new(id))),
		}
	}
}

impl From<TId<Type>> for Type {
	fn from(t: TId<Type>) -> Self {
		t.id().into()
	}
}

impl<C: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>> DisplayWithContext<C> for OtherTypeId {
	fn fmt_with(&self, context: &C, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		self.raw_id().fmt_with(context, f)
	}
}

impl<C: Vocabulary<Iri = IriIndex, BlankId = BlankIdIndex>> DisplayWithContext<C> for Type {
	fn fmt_with(&self, context: &C, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		self.raw_id().fmt_with(context, f)
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum SubClass {
	DataType,
	Restriction,
}

impl SubClass {
	/// Checks if this is a subclass of `other`.
	pub fn is_subclass_of(&self, _other: Self) -> bool {
		false
	}

	pub fn term(&self) -> Term {
		match self {
			Self::DataType => Term::Rdfs(vocab::Rdfs::Datatype),
			Self::Restriction => Term::Owl(vocab::Owl::Restriction),
		}
	}
}

/// Type definition.
#[derive(Debug)]
pub struct Definition<M> {
	/// Type description.
	desc: Description<M>,
}

/// Type definition.
#[derive(Debug)]
pub enum Description<M> {
	Empty,
	Data(data::Definition<M>),
	Normal(Normal<M>),
	Union(Union<M>),
	Intersection(Intersection<M>),
	Restriction(restriction::Definition<M>),
}

impl<M> Description<M> {
	pub fn kind(&self) -> Kind {
		match self {
			Self::Empty => Kind::Empty,
			Self::Data(_) => Kind::Data,
			Self::Normal(_) => Kind::Normal,
			Self::Union(_) => Kind::Union,
			Self::Intersection(_) => Kind::Intersection,
			Self::Restriction(_) => Kind::Restriction,
		}
	}

	pub fn is_datatype(&self, model: &MutableModel<M>) -> bool {
		match self {
			Self::Data(_) => true,
			Self::Union(u) => u.is_datatype(model),
			Self::Intersection(i) => i.is_datatype(model),
			_ => false,
		}
	}

	pub fn dependencies(&self) -> Multiple<TId<crate::Type>, M>
	where
		M: Clone,
	{
		let mut result = Multiple::default();

		if let Self::Intersection(i) = self {
			for id in i.types() {
				result.insert_unique(id.cloned());
			}
		}

		result
	}

	pub(crate) fn compute_properties(
		&self,
		class_properties: &HashMap<TId<crate::Type>, Properties<M>>,
		result: &mut Properties<M>,
	) -> Result<(), restriction::Contradiction>
	where
		M: Clone + Merge,
	{
		match self {
			Self::Intersection(i) => {
				*result = Properties::all();

				for Meta(id, _) in i.types() {
					result.intersect_with(class_properties.get(id).unwrap())?;
				}
			}
			Self::Restriction(r) => {
				*result = Properties::all();

				result.restrict(r.property(), r.restriction().clone())?;
			}
			_ => (),
		}

		Ok(())
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum Kind {
	Empty,
	Data,
	Normal,
	Union,
	Intersection,
	Restriction,
}

impl<M> Definition<M> {
	pub fn new(desc: Description<M>) -> Self {
		Self { desc }
	}

	pub fn description(&self) -> &Description<M> {
		&self.desc
	}

	pub fn is_datatype(&self, model: &MutableModel<M>) -> bool {
		self.desc.is_datatype(model)
	}

	pub fn union_of(
		&self,
	) -> Option<&RequiredFunctionalPropertyValue<Multiple<TId<crate::Type>, M>, M>> {
		match &self.desc {
			Description::Union(u) => Some(u.union_of()),
			_ => None,
		}
	}

	pub fn intersection_of(
		&self,
	) -> Option<&RequiredFunctionalPropertyValue<Multiple<TId<crate::Type>, M>, M>> {
		match &self.desc {
			Description::Intersection(i) => Some(i.intersection_of()),
			_ => None,
		}
	}

	pub fn sub_class_of(&self) -> Option<&PropertyValues<TId<crate::Type>, M>> {
		match &self.desc {
			Description::Normal(n) => Some(n.sub_class_of()),
			_ => None,
		}
	}

	pub fn collect_all_superclasses(
		&self,
		model: &MutableModel<M>,
		result: &mut HashSet<TId<crate::Type>>,
	) {
		if let Some(super_classes) = self.sub_class_of() {
			for PropertyValueRef {
				value: Meta(&id, _),
				..
			} in super_classes
			{
				if result.insert(id) {
					let ty = model.get(id).unwrap();
					ty.as_type().collect_all_superclasses(model, result);
				}
			}
		}
	}

	pub fn all_superclasses(&self, model: &MutableModel<M>) -> HashSet<TId<crate::Type>> {
		let mut result = HashSet::new();
		self.collect_all_superclasses(model, &mut result);
		result
	}

	pub fn class_bindings(&self) -> ClassBindings<M> {
		ClassBindings {
			union_of: self.union_of().map(RequiredFunctionalPropertyValue::iter),
			intersection_of: self
				.intersection_of()
				.map(RequiredFunctionalPropertyValue::iter),
		}
	}

	pub fn datatype_bindings(&self) -> Option<data::Bindings<M>> {
		match &self.desc {
			Description::Data(dt) => Some(dt.bindings()),
			_ => None,
		}
	}

	pub fn restriction_bindings(&self) -> Option<restriction::Bindings<M>> {
		match &self.desc {
			Description::Restriction(r) => Some(r.bindings()),
			_ => None,
		}
	}

	pub fn bindings(&self) -> Bindings<M> {
		Bindings {
			data: self.class_bindings(),
			datatype: self.datatype_bindings().unwrap_or_default(),
			restriction: self.restriction_bindings().unwrap_or_default(),
		}
	}

	pub fn dependencies(&self) -> Multiple<TId<crate::Type>, M>
	where
		M: Clone,
	{
		self.desc.dependencies()
	}

	pub(crate) fn compute_properties(
		&self,
		class_properties: &HashMap<TId<crate::Type>, Properties<M>>,
		result: &mut Properties<M>,
	) -> Result<(), restriction::Contradiction>
	where
		M: Clone + Merge,
	{
		self.desc.compute_properties(class_properties, result)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Property {
	Datatype(data::Property),
	Restriction(restriction::Property),
	SubClassOf(Option<TId<UnknownProperty>>),
	UnionOf(Option<TId<UnknownProperty>>),
	IntersectionOf(Option<TId<UnknownProperty>>),
}

impl Property {
	pub fn id(&self) -> Id {
		use vocab::Owl;
		match self {
			Self::Datatype(p) => p.id(),
			Self::Restriction(p) => p.id(),
			Self::SubClassOf(None) => Id::Iri(IriIndex::Iri(Term::Rdfs(Rdfs::SubClassOf))),
			Self::SubClassOf(Some(p)) => p.id(),
			Self::UnionOf(None) => Id::Iri(IriIndex::Iri(Term::Owl(Owl::UnionOf))),
			Self::UnionOf(Some(p)) => p.id(),
			Self::IntersectionOf(None) => Id::Iri(IriIndex::Iri(Term::Owl(Owl::IntersectionOf))),
			Self::IntersectionOf(Some(p)) => p.id(),
		}
	}

	pub fn term(&self) -> Option<vocab::Term> {
		use vocab::Owl;
		match self {
			Self::Datatype(p) => p.term(),
			Self::Restriction(p) => p.term(),
			Self::SubClassOf(None) => Some(Term::Rdfs(Rdfs::SubClassOf)),
			Self::UnionOf(None) => Some(Term::Owl(Owl::UnionOf)),
			Self::IntersectionOf(None) => Some(Term::Owl(Owl::IntersectionOf)),
			_ => None,
		}
	}

	pub fn name(&self) -> PropertyName {
		match self {
			Self::Datatype(p) => p.name(),
			Self::Restriction(p) => p.name(),
			Self::SubClassOf(None) => PropertyName::Resource("super class"),
			Self::SubClassOf(Some(p)) => PropertyName::Other(*p),
			Self::UnionOf(None) => PropertyName::Resource("type union"),
			Self::UnionOf(Some(p)) => PropertyName::Other(*p),
			Self::IntersectionOf(None) => PropertyName::Resource("type intersection"),
			Self::IntersectionOf(Some(p)) => PropertyName::Other(*p),
		}
	}

	pub fn expect_type(&self) -> bool {
		match self {
			Self::Datatype(p) => p.expect_type(),
			Self::Restriction(p) => p.expect_type(),
			_ => false,
		}
	}

	pub fn expect_layout(&self) -> bool {
		match self {
			Self::Datatype(p) => p.expect_layout(),
			Self::Restriction(p) => p.expect_layout(),
			_ => false,
		}
	}
}

pub enum ClassBindingRef<'a, M> {
	UnionOf(
		Option<TId<UnknownProperty>>,
		&'a Multiple<TId<crate::Type>, M>,
	),
	IntersectionOf(
		Option<TId<UnknownProperty>>,
		&'a Multiple<TId<crate::Type>, M>,
	),
}

impl<'a, M> ClassBindingRef<'a, M> {
	pub fn into_binding_ref(self) -> BindingRef<'a, M> {
		match self {
			Self::UnionOf(p, i) => BindingRef::UnionOf(p, i),
			Self::IntersectionOf(p, i) => BindingRef::IntersectionOf(p, i),
		}
	}
}

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct ClassBindings<'a, M> {
	union_of:
		Option<property_values::required_functional::Iter<'a, Multiple<TId<crate::Type>, M>, M>>,
	intersection_of:
		Option<property_values::required_functional::Iter<'a, Multiple<TId<crate::Type>, M>, M>>,
}

impl<'a, M> Iterator for ClassBindings<'a, M> {
	type Item = Meta<ClassBindingRef<'a, M>, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.union_of
			.as_mut()
			.and_then(|v| {
				v.next()
					.map(|v| v.into_class_binding(ClassBindingRef::UnionOf))
			})
			.or_else(|| {
				self.intersection_of.as_mut().and_then(|v| {
					v.next()
						.map(|v| v.into_class_binding(ClassBindingRef::IntersectionOf))
				})
			})
	}
}

pub enum BindingRef<'a, M> {
	UnionOf(
		Option<TId<UnknownProperty>>,
		&'a Multiple<TId<crate::Type>, M>,
	),
	IntersectionOf(
		Option<TId<UnknownProperty>>,
		&'a Multiple<TId<crate::Type>, M>,
	),
	Datatype(data::BindingRef<'a, M>),
	Restriction(restriction::BindingRef<'a>),
}

impl<'a, M> BindingRef<'a, M> {
	pub fn domain(&self) -> Option<SubClass> {
		match self {
			Self::Datatype(_) => Some(SubClass::DataType),
			Self::Restriction(_) => Some(SubClass::Restriction),
			_ => None,
		}
	}

	pub fn property(&self) -> Property {
		match self {
			Self::UnionOf(p, _) => Property::UnionOf(*p),
			Self::IntersectionOf(p, _) => Property::IntersectionOf(*p),
			Self::Datatype(b) => Property::Datatype(b.property()),
			Self::Restriction(b) => Property::Restriction(b.property()),
		}
	}

	pub fn value(&self) -> BindingValueRef<'a, M> {
		match self {
			Self::UnionOf(_, v) => BindingValueRef::TypeList(node::MultipleIdValueRef::Multiple(v)),
			Self::IntersectionOf(_, v) => {
				BindingValueRef::TypeList(node::MultipleIdValueRef::Multiple(v))
			}
			Self::Datatype(b) => b.value(),
			Self::Restriction(b) => b.value(),
		}
	}
}

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct Bindings<'a, M> {
	data: ClassBindings<'a, M>,
	datatype: data::Bindings<'a, M>,
	restriction: restriction::Bindings<'a, M>,
}

impl<'a, M> Iterator for Bindings<'a, M> {
	type Item = Meta<BindingRef<'a, M>, &'a M>;

	fn next(&mut self) -> Option<Self::Item> {
		self.data
			.next()
			.map(|m| m.map(ClassBindingRef::into_binding_ref))
			.or_else(|| {
				self.datatype
					.next()
					.map(|m| m.map(BindingRef::Datatype))
					.or_else(|| {
						self.restriction
							.next()
							.map(|m| m.map(BindingRef::Restriction))
					})
			})
	}
}

pub struct Hierarchy<M> {
	sub_classes: HashMap<TId<crate::Type>, Multiple<TId<crate::Type>, M>>,
	depths: OnceCell<HashMap<TId<crate::Type>, usize>>,
}

impl<M> Hierarchy<M> {
	pub fn new(model: &MutableModel<M>) -> Self
	where
		M: Clone,
	{
		let mut result: HashMap<TId<crate::Type>, Multiple<TId<crate::Type>, M>> = HashMap::new();

		for (id, node) in model.nodes() {
			if let Some(ty) = node.as_type() {
				result.insert(TId::new(id), Multiple::default());

				if let Some(super_classes) = ty.sub_class_of() {
					for PropertyValueRef {
						value: Meta(s, meta),
						..
					} in super_classes
					{
						result
							.entry(*s)
							.or_default()
							.insert_unique(Meta(TId::new(id), meta.clone()));
					}
				}
			}
		}

		Self {
			sub_classes: result,
			depths: OnceCell::new(),
		}
	}

	/// Assigns an index to each declared type where sub classes has a greater index than their super classes.
	fn compute_type_depths(&self) -> HashMap<TId<crate::Type>, usize> {
		let mut stack: Vec<(TId<crate::Type>, usize)> =
			self.sub_classes.keys().copied().map(|id| (id, 0)).collect();

		let mut result = HashMap::new();

		while let Some((ty, depth)) = stack.pop() {
			if !result.contains_key(&ty) || result[&ty] < depth {
				result.insert(ty, depth);

				for Meta(&sub_class, _) in &self.sub_classes[&ty] {
					stack.push((sub_class, depth + 1));
				}
			}
		}

		result
	}

	/// Returns the "depth" of the given type.
	///
	/// The depth of a type is always grater that the depth of its super classes.
	pub fn depth(&self, ty: TId<crate::Type>) -> Option<usize> {
		self.depths
			.get_or_init(|| self.compute_type_depths())
			.get(&ty)
			.copied()
	}
}

/// Class dependency analysis.
///
/// A class is dependent from another class if it is required to compute
/// its set of properties.
///
/// The intersection `A & B` depends on `A` and `B`.
pub struct Dependencies<M> {
	map: HashMap<TId<crate::Type>, Multiple<TId<crate::Type>, M>>,
	depths: OnceCell<HashMap<TId<crate::Type>, usize>>,
}

impl<M> Dependencies<M> {
	pub fn new(model: &MutableModel<M>) -> Self
	where
		M: Clone,
	{
		let mut result: HashMap<TId<crate::Type>, Multiple<TId<crate::Type>, M>> = HashMap::new();

		for (id, node) in model.nodes() {
			if let Some(ty) = node.as_type() {
				result.insert(TId::new(id), ty.dependencies());
			}
		}

		Self {
			map: result,
			depths: OnceCell::new(),
		}
	}

	/// Assigns an index to each declared type where dependent classes have a greater index than their depended classes.
	fn compute_type_depths(&self) -> HashMap<TId<crate::Type>, usize> {
		let mut stack: Vec<(TId<crate::Type>, usize)> =
			self.map.keys().copied().map(|id| (id, 0)).collect();

		let mut result = HashMap::new();

		while let Some((ty, depth)) = stack.pop() {
			if !result.contains_key(&ty) || result[&ty] < depth {
				result.insert(ty, depth);

				for Meta(&dep, _) in &self.map[&ty] {
					stack.push((dep, depth + 1));
				}
			}
		}

		result
	}

	/// Returns the "depth" of the given type.
	///
	/// The depth of a type is always grater that the depth of its dependencies.
	pub fn depth(&self, ty: TId<crate::Type>) -> Option<usize> {
		self.depths
			.get_or_init(|| self.compute_type_depths())
			.get(&ty)
			.copied()
	}

	pub fn compute_class_properties(
		&self,
		model: &MutableModel<M>,
	) -> Result<HashMap<TId<Type>, Properties<M>>, restriction::Contradiction>
	where
		M: Clone + Merge,
	{
		let mut classes: Vec<_> = self.map.keys().copied().collect();
		classes.sort_unstable_by_key(|id| self.depth(*id));

		let mut direct_properties: HashMap<TId<Type>, Properties<M>> = HashMap::new();

		for (prop_id, prop) in model.properties() {
			for PropertyValueRef {
				value: Meta(&domain, meta),
				..
			} in prop.as_property().domain()
			{
				direct_properties
					.entry(domain)
					.or_default()
					.insert(prop_id, None, meta.clone());
			}
		}

		let mut result: HashMap<TId<Type>, Properties<M>> = HashMap::new();

		for id in classes.into_iter().rev() {
			let ty = model.get(id).unwrap();
			let mut properties = direct_properties.remove(&id).unwrap_or_default();
			ty.as_type().compute_properties(&result, &mut properties)?;
			result.insert(id, properties);
		}

		Ok(result)
	}
}
