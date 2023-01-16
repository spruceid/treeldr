use std::{
	collections::{HashMap, HashSet},
	hash::Hash,
};

// Solve dependencies using Tarjan's SCC algorithm.
struct Data {
	index: u32,
	lowlink: u32,
	on_stack: bool,
	component: usize,
}

pub trait SccGraph {
	/// Vertex reference.
	type Vertex: Copy + Eq + Hash;

	type Vertices<'a>: 'a + IntoIterator<Item = Self::Vertex>
	where
		Self: 'a;

	type Successors<'a>: 'a + IntoIterator<Item = Self::Vertex>
	where
		Self: 'a;

	fn vertices(&self) -> Self::Vertices<'_>;

	fn successors(&self, v: Self::Vertex) -> Self::Successors<'_>;

	fn strongly_connected_components(&self) -> Components<Self::Vertex> {
		let mut map: HashMap<Self::Vertex, Data> = HashMap::new();
		let mut stack = Vec::new();
		let mut components = Vec::new();

		for v in self.vertices() {
			if !map.contains_key(&v) {
				strong_connect(self, v, &mut stack, &mut map, &mut components);
			}
		}

		let vertex_to_component: HashMap<_, _> = map
			.into_iter()
			.map(|(v, data)| (v, data.component))
			.collect();

		let successors: Vec<HashSet<_>> = components
			.iter()
			.map(|component| {
				component
					.iter()
					.flat_map(|v| {
						self.successors(*v)
							.into_iter()
							.map(|sc| *vertex_to_component.get(&sc).unwrap())
					})
					.collect()
			})
			.collect();

		Components {
			vertex_to_component,
			components,
			successors,
		}
	}
}

pub struct Components<V> {
	vertex_to_component: HashMap<V, usize>,
	components: Vec<Vec<V>>,
	successors: Vec<HashSet<usize>>,
}

impl<V> Components<V> {
	pub fn len(&self) -> usize {
		self.components.len()
	}

	pub fn is_empty(&self) -> bool {
		self.components.is_empty()
	}

	pub fn iter(&self) -> impl Iterator<Item = &[V]> {
		self.components.iter().map(Vec::as_slice)
	}

	pub fn vertex_component(&self, v: &V) -> Option<usize>
	where
		V: Eq + Hash,
	{
		self.vertex_to_component.get(v).cloned()
	}

	pub fn get(&self, i: usize) -> Option<&[V]> {
		self.components.get(i).map(Vec::as_slice)
	}

	pub fn successors(&self, i: usize) -> Option<impl '_ + Iterator<Item = usize>> {
		self.successors.get(i).map(|s| s.iter().cloned())
	}

	fn remove_indirect_successors(&self, result: &mut HashSet<usize>, i: usize) {
		for j in self.successors(i).unwrap() {
			result.remove(&j);
			self.remove_indirect_successors(result, j);
		}
	}

	pub fn direct_successors(&self, i: usize) -> Option<HashSet<usize>> {
		let mut result: HashSet<_> = self.successors(i)?.collect();

		for j in self.successors(i).unwrap() {
			self.remove_indirect_successors(&mut result, j);
		}

		Some(result)
	}

	/// Returns the depth of each component.
	///
	/// The depth of a component is the maximum of the depth of its predecessors
	/// plus 1. A component with no predecessors has depth 0.
	pub fn depths(&self) -> Vec<usize> {
		let mut depth = Vec::new();
		depth.resize(self.components.len(), 0);
		let mut stack: Vec<_> = depth.iter().cloned().enumerate().collect();

		while let Some((i, new_depth)) = stack.pop() {
			if depth[i] == 0 || new_depth > depth[i] {
				depth[i] = new_depth;
				for c in self.successors(i).unwrap() {
					if c != i {
						stack.push((c, new_depth + 1))
					}
				}
			}
		}

		depth
	}

	pub fn predecessors(&self) -> Vec<HashSet<usize>> {
		let mut predecessors = Vec::new();
		predecessors.resize_with(self.components.len(), HashSet::default);

		for (i, successors) in self.successors.iter().enumerate() {
			for &j in successors {
				predecessors[j].insert(i);
			}
		}

		predecessors
	}

	/// Order components by depth.
	///
	/// The depth of a component is the maximum of the depth of its predecessors
	/// plus 1. A component with no predecessors has depth 0.
	pub fn order_by_depth(&self) -> Vec<usize> {
		let depth = self.depths();
		let mut ordered_components: Vec<_> = (0..self.components.len()).collect();
		ordered_components.sort_unstable_by_key(|i| depth[*i]);
		ordered_components
	}
}

/// Returns the depth of each component.
///
/// The depth of a component is the maximum of the depth of its predecessors
/// plus 1. A component with no predecessors has depth 0.
pub fn depths(predecessors: &[HashSet<usize>]) -> Vec<usize> {
	let mut depth = Vec::new();
	depth.resize(predecessors.len(), 0);
	let mut stack: Vec<_> = depth.iter().cloned().enumerate().collect();

	while let Some((i, new_depth)) = stack.pop() {
		if depth[i] == 0 || new_depth > depth[i] {
			depth[i] = new_depth;
			for &c in &predecessors[i] {
				if c != i {
					stack.push((c, new_depth + 1))
				}
			}
		}
	}

	depth
}

fn strong_connect<G: ?Sized + SccGraph>(
	graph: &G,
	v: G::Vertex,
	stack: &mut Vec<G::Vertex>,
	map: &mut HashMap<G::Vertex, Data>,
	components: &mut Vec<Vec<G::Vertex>>,
) -> u32 {
	let index = map.len() as u32;
	stack.push(v);
	map.insert(
		v,
		Data {
			index,
			lowlink: index,
			on_stack: true,
			component: 0,
		},
	);

	// Consider successors of v
	for w in graph.successors(v) {
		let new_v_lowlink = match map.get(&w) {
			None => {
				// Successor w has not yet been visited; recurse on it
				let w_lowlink = strong_connect(graph, w, stack, map, components);
				Some(std::cmp::min(map[&v].lowlink, w_lowlink))
			}
			Some(w_data) => {
				if w_data.on_stack {
					// Successor w is in stack S and hence in the current SCC
					// If w is not on stack, then (v, w) is an edge pointing to an SCC already found and must be ignored
					// Note: The next line may look odd - but is correct.
					// It says w.index not w.lowlink; that is deliberate and from the original paper
					Some(std::cmp::min(map[&v].lowlink, w_data.index))
				} else {
					None
				}
			}
		};

		if let Some(new_v_lowlink) = new_v_lowlink {
			map.get_mut(&v).unwrap().lowlink = new_v_lowlink;
		}
	}

	let lowlink = map[&v].lowlink;

	// If v is a root node, pop the stack and generate an SCC
	if lowlink == map[&v].index {
		// Start a new strongly connected component
		let mut component = Vec::new();

		loop {
			let w = stack.pop().unwrap();
			let w_data = map.get_mut(&w).unwrap();
			w_data.on_stack = false;
			w_data.component = components.len();

			// Add w to current strongly connected component
			component.push(w);

			if w == v {
				break;
			}
		}

		// Output the current strongly connected component
		components.push(component)
	}

	lowlink
}
