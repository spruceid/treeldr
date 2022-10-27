use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct ClassRef(usize);

#[derive(Clone)]
pub struct UnionFind<K, V> {
	nodes: Nodes<K>,
	classes: Classes<V>,
}

impl<K, V> Default for UnionFind<K, V> {
	fn default() -> Self {
		Self {
			nodes: Nodes::default(),
			classes: Classes::default(),
		}
	}
}

impl<K, V> UnionFind<K, V> {
	pub fn new() -> Self {
		Self::default()
	}
}

impl<K: Eq + Hash, V> UnionFind<K, V> {
	pub fn insert(&mut self, key: K, value: V) {
		assert!(self.nodes.class_of(&key).is_none());
		let class_ref = self.classes.new_class();
		self.classes.insert(class_ref, value);
		self.nodes.insert(key, class_ref);
	}

	pub fn merge(&mut self, a: &K, b: &K, f: impl FnOnce(V, V) -> V) {
		let a_class = self.nodes.class_of(a).unwrap();
		let b_class = self.nodes.class_of(b).unwrap();

		if a_class != b_class {
			let a_value = self.classes.take(a_class).unwrap();
			let b_value = self.classes.take(b_class).unwrap();
			let merged = f(a_value, b_value);

			self.classes.insert(b_class, merged);
			self.nodes.merge(a, b);
		}
	}

	pub fn merge_with(&mut self, k: &K, value: V, f: impl FnOnce(V, V) -> V) {
		let k_class = self.nodes.class_of(k).unwrap();
		let k_value = self.classes.take(k_class).unwrap();
		let merged = f(k_value, value);
		self.classes.insert(k_class, merged);
	}

	pub fn try_merge<E>(
		&mut self,
		a: &K,
		b: &K,
		f: impl FnOnce(V, V) -> Result<V, E>,
	) -> Result<(), E> {
		let a_class = self.nodes.class_of(a).unwrap();
		let b_class = self.nodes.class_of(b).unwrap();

		if a_class != b_class {
			let a_value = self.classes.take(a_class).unwrap();
			let b_value = self.classes.take(b_class).unwrap();
			let merged = f(a_value, b_value)?;

			self.classes.insert(b_class, merged);
			self.nodes.merge(a, b);
		}

		Ok(())
	}

	pub fn try_merge_with<E>(
		&mut self,
		k: &K,
		value: V,
		f: impl FnOnce(V, V) -> Result<V, E>,
	) -> Result<(), E> {
		let k_class = self.nodes.class_of(k).unwrap();
		let k_value = self.classes.take(k_class).unwrap();
		let merged = f(k_value, value)?;
		self.classes.insert(k_class, merged);
		Ok(())
	}

	pub fn get(&self, k: &K) -> Option<&V> {
		self.nodes.class_of(k).and_then(|c| self.classes.get(c))
	}

	pub fn get_or_insert_with<F>(&mut self, k: K, f: F) -> &V
	where
		F: FnOnce() -> V,
	{
		let class_ref = self.class_of_or_insert_with(k, f);
		self.classes.get(class_ref).unwrap()
	}

	pub fn class_of(&self, k: &K) -> Option<ClassRef> {
		self.nodes.class_of(k)
	}

	pub fn class_of_or_insert_with<F>(&mut self, k: K, f: F) -> ClassRef
	where
		F: FnOnce() -> V,
	{
		self.nodes.class_of_or_insert_with(k, || {
			let class_ref = self.classes.new_class();
			self.classes.insert(class_ref, f());
			class_ref
		})
	}

	pub fn map<W>(self, f: impl FnMut(V) -> W) -> UnionFind<K, W> {
		UnionFind {
			nodes: self.nodes,
			classes: self.classes.map(f),
		}
	}

	pub fn try_map<E, W>(self, f: impl FnMut(V) -> Result<W, E>) -> Result<UnionFind<K, W>, E> {
		Ok(UnionFind {
			nodes: self.nodes,
			classes: self.classes.try_map(f)?,
		})
	}

	pub fn class(&self, class: ClassRef) -> Option<&V> {
		self.classes.get(class)
	}

	pub fn classes(&self) -> impl Iterator<Item = (ClassRef, &V)> {
		self.classes.iter()
	}
}

#[derive(Clone)]
struct Nodes<K> {
	indexes: HashMap<K, usize>,
	list: Vec<Node>,
}

impl<K> Default for Nodes<K> {
	fn default() -> Self {
		Self {
			indexes: HashMap::default(),
			list: Vec::default(),
		}
	}
}

impl<K: Eq + Hash> Nodes<K> {
	fn index_of(&self, k: &K) -> Option<usize> {
		self.indexes.get(k).cloned()
	}

	fn index_of_or_insert_with<F>(&mut self, k: K, f: F) -> usize
	where
		F: FnOnce() -> ClassRef,
	{
		*self.indexes.entry(k).or_insert_with(|| {
			let index = self.list.len();
			self.list.push(Node::Class(f()));
			index
		})
	}

	fn class_of(&self, k: &K) -> Option<ClassRef> {
		self.index_of(k).map(|i| self.class_of_index(i))
	}

	fn class_of_index(&self, index: usize) -> ClassRef {
		match self.list[index] {
			Node::Class(c) => c,
			Node::SameAs(other) => self.class_of_index(other),
		}
	}

	fn class_of_or_insert_with<F>(&mut self, k: K, f: F) -> ClassRef
	where
		F: FnOnce() -> ClassRef,
	{
		let class_ref = self.index_of_or_insert_with(k, f);
		self.class_of_index(class_ref)
	}

	fn insert(&mut self, key: K, class: ClassRef) {
		let index = self.list.len();
		self.indexes.insert(key, index);
		self.list.push(Node::Class(class));
	}

	/// Merges a into b.
	fn merge(&mut self, a: &K, b: &K) {
		let a = self.index_of(a).unwrap();
		let b = self.index_of(b).unwrap();
		self.merge_indexes(a, b)
	}

	fn merge_indexes(&mut self, a: usize, b: usize) {
		match self.list[a] {
			Node::Class(_) => self.list[a] = Node::SameAs(b),
			Node::SameAs(other) => self.merge_indexes(other, b),
		}
	}
}

#[derive(Clone)]
enum Node {
	Class(ClassRef),
	SameAs(usize),
}

#[derive(Clone)]
struct Classes<V> {
	count: usize,
	map: BTreeMap<ClassRef, V>,
}

impl<V> Default for Classes<V> {
	fn default() -> Self {
		Self {
			count: 0,
			map: BTreeMap::new(),
		}
	}
}

impl<V> Classes<V> {
	pub fn get(&self, class: ClassRef) -> Option<&V> {
		self.map.get(&class)
	}

	pub fn new_class(&mut self) -> ClassRef {
		let c = ClassRef(self.count);
		self.count += 1;
		c
	}

	pub fn take(&mut self, class: ClassRef) -> Option<V> {
		self.map.remove(&class)
	}

	pub fn insert(&mut self, class: ClassRef, value: V) {
		self.map.insert(class, value);
	}

	pub fn map<W>(self, mut f: impl FnMut(V) -> W) -> Classes<W> {
		let mut map = BTreeMap::new();

		for (class, value) in self.map {
			map.insert(class, f(value));
		}

		Classes {
			count: self.count,
			map,
		}
	}

	pub fn try_map<E, W>(self, mut f: impl FnMut(V) -> Result<W, E>) -> Result<Classes<W>, E> {
		let mut map = BTreeMap::new();

		for (class, value) in self.map {
			map.insert(class, f(value)?);
		}

		Ok(Classes {
			count: self.count,
			map,
		})
	}

	pub fn iter(&self) -> impl Iterator<Item = (ClassRef, &V)> {
		self.map.iter().map(|(c, v)| (*c, v))
	}
}
