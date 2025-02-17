use iref::IriBuf;
use rdf_types::Literal;

pub struct ClassResources<'a, R> {
	pub parent_resources: &'a [R],
	pub local_resources: &'a [R],
}

impl<'a, R> ClassResources<'a, R> {
	pub fn iter(&self) -> std::iter::Chain<std::slice::Iter<'a, R>, std::slice::Iter<'a, R>> {
		self.parent_resources.iter().chain(self.local_resources)
	}
}

impl<'a, R> IntoIterator for ClassResources<'a, R> {
	type Item = &'a R;
	type IntoIter = std::iter::Chain<std::slice::Iter<'a, R>, std::slice::Iter<'a, R>>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

pub struct Class<R> {
	pub canonical_resource: R,

	pub local_resources: Vec<R>,
	pub parent_resources: Vec<R>,

	pub local_iris: Vec<IriBuf>,
	pub parent_iris: Vec<IriBuf>,

	pub local_literals: Vec<Literal>,
	pub parent_literals: Vec<Literal>,
}

impl<R> Class<R> {
	pub fn new_local(
		canonical_resource: R,
		iris: impl IntoIterator<Item = IriBuf>,
		literals: impl IntoIterator<Item = Literal>,
	) -> Self
	where
		R: Clone,
	{
		Self {
			canonical_resource: canonical_resource.clone(),
			local_resources: vec![canonical_resource],
			parent_resources: Vec::new(),
			local_iris: iris.into_iter().collect(),
			parent_iris: Vec::new(),
			local_literals: literals.into_iter().collect(),
			parent_literals: Vec::new(),
		}
	}

	pub fn from_parent(
		canonical_resource: R,
		parent_resources: Vec<R>,
		iris: impl IntoIterator<Item = IriBuf>,
		literals: impl IntoIterator<Item = Literal>,
	) -> Self {
		Self {
			canonical_resource,
			parent_resources,
			local_resources: Vec::new(),
			parent_iris: iris.into_iter().collect(),
			local_iris: Vec::new(),
			parent_literals: literals.into_iter().collect(),
			local_literals: Vec::new(),
		}
	}

	pub fn iris(
		&self,
	) -> std::iter::Chain<std::slice::Iter<'_, IriBuf>, std::slice::Iter<'_, IriBuf>> {
		self.local_iris.iter().chain(&self.parent_iris)
	}

	pub fn literals(
		&self,
	) -> std::iter::Chain<std::slice::Iter<'_, Literal>, std::slice::Iter<'_, Literal>> {
		self.local_literals.iter().chain(&self.parent_literals)
	}

	pub fn resources(&self) -> ClassResources<R> {
		ClassResources {
			parent_resources: &self.parent_resources,
			local_resources: &self.local_resources,
		}
	}

	pub fn unify(mut self, other: Self) -> Self
	where
		R: Ord,
	{
		self.local_resources.extend(other.local_resources);
		self.parent_resources.extend(other.parent_resources);
		self.local_iris.extend(other.local_iris);
		self.parent_iris.extend(other.parent_iris);
		self.local_literals.extend(other.local_literals);
		self.parent_literals.extend(other.parent_literals);
		Self {
			canonical_resource: std::cmp::min(self.canonical_resource, other.canonical_resource),
			local_resources: self.local_resources,
			parent_resources: self.parent_resources,
			local_iris: self.local_iris,
			parent_iris: self.parent_iris,
			local_literals: self.local_literals,
			parent_literals: self.parent_literals,
		}
	}
}
