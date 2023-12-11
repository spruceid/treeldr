use btree_range_map::{AnyRange, RangeMap, RangeSet};
use std::{
	collections::{hash_map::Entry, BTreeMap, BTreeSet, HashMap, HashSet},
	hash::Hash,
	marker::PhantomData,
};

use super::charset_intersection;

/// Non deterministic state transitions.
pub type Transitions<Q> = BTreeMap<Option<RangeSet<char>>, BTreeSet<Q>>;

/// Non deterministic lexing automaton.
pub struct Automaton<Q> {
	transitions: BTreeMap<Q, Transitions<Q>>,
	initial_states: BTreeSet<Q>,
	final_states: BTreeSet<Q>,
}

impl<Q> Default for Automaton<Q> {
	fn default() -> Self {
		Self {
			transitions: BTreeMap::new(),
			initial_states: BTreeSet::new(),
			final_states: BTreeSet::new(),
		}
	}
}

impl<Q> Automaton<Q> {
	/// Create a new empty non deterministic automaton.
	pub fn new() -> Self {
		Self::default()
	}

	pub fn transitions(&self) -> std::collections::btree_map::Iter<Q, Transitions<Q>> {
		self.transitions.iter()
	}
}

impl<Q: Ord> Automaton<Q> {
	/// Get the successors of the given state.
	pub fn successors(&self, q: &Q) -> Successors<Q> {
		Successors::new(self.transitions.get(q))
	}

	pub fn add(&mut self, source: Q, label: Option<RangeSet<char>>, target: Q)
	where
		Q: Clone,
	{
		self.declare_state(target.clone());
		self.transitions
			.entry(source)
			.or_default()
			.entry(label)
			.or_default()
			.insert(target);
	}

	pub fn declare_state(&mut self, q: Q) {
		self.transitions.entry(q).or_default();
	}

	pub fn is_initial_state(&self, q: &Q) -> bool {
		self.initial_states.contains(q)
	}

	pub fn add_initial_state(&mut self, q: Q) -> bool {
		self.initial_states.insert(q)
	}

	pub fn is_final_state(&self, q: &Q) -> bool {
		self.final_states.contains(q)
	}

	pub fn final_states(&self) -> &BTreeSet<Q> {
		&self.final_states
	}

	pub fn add_final_state(&mut self, q: Q) -> bool {
		self.final_states.insert(q)
	}

	fn modulo_epsilon_state<'a>(&'a self, qs: impl IntoIterator<Item = &'a Q>) -> BTreeSet<&'a Q> {
		let mut states = BTreeSet::new();
		let mut stack: Vec<_> = qs.into_iter().collect();

		while let Some(q) = stack.pop() {
			if states.insert(q) {
				// add states reachable trough epsilon-transitions.
				if let Some(transitions) = self.transitions.get(q) {
					if let Some(epsilon_qs) = transitions.get(&None) {
						for t in epsilon_qs {
							stack.push(t)
						}
					}
				}
			}
		}

		states
	}

	fn determinize_transitions_for(
		&self,
		states: &BTreeSet<&Q>,
	) -> BTreeMap<AnyRange<char>, BTreeSet<&Q>> {
		let mut map = RangeMap::new();

		for q in states {
			if let Some(transitions) = self.transitions.get(q) {
				for (label, targets) in transitions {
					if let Some(label) = label {
						for range in label.iter() {
							debug_assert!(!range.is_empty());

							map.update(
								*range,
								|current_target_states_opt: Option<&BTreeSet<&Q>>| {
									let mut current_target_states = match current_target_states_opt
									{
										Some(current_target_states) => {
											current_target_states.clone()
										}
										None => BTreeSet::new(),
									};

									for q in targets {
										current_target_states
											.extend(self.modulo_epsilon_state(Some(q)));
									}

									Some(current_target_states)
								},
							);

							assert!(map.get(range.first().unwrap()).is_some());
						}
					}
				}
			}
		}

		let mut simplified_map = BTreeMap::new();

		for (range, set) in map {
			debug_assert!(!range.is_empty());
			simplified_map.insert(range, set);
		}

		simplified_map
	}

	pub fn determinize(&self) -> DetAutomaton<BTreeSet<&Q>>
	where
		Q: Hash,
	{
		let mut transitions = BTreeMap::new();

		// create the initial deterministic state.
		let initial_state = self.modulo_epsilon_state(&self.initial_states);
		let mut final_states = BTreeSet::new();

		let mut visited_states = HashSet::new();
		let mut stack = vec![initial_state.clone()];
		while let Some(det_q) = stack.pop() {
			if visited_states.insert(det_q.clone()) {
				if det_q.iter().any(|q| self.final_states.contains(q)) {
					final_states.insert(det_q.clone());
				}

				let map = self.determinize_transitions_for(&det_q);

				for next_det_q in map.values() {
					stack.push(next_det_q.clone())
				}

				transitions.insert(det_q, map);
			}
		}

		DetAutomaton {
			initial_state,
			final_states,
			transitions: DetTransitions(transitions),
		}
	}

	pub fn product<'a, 'b, R, S>(
		&'a self,
		other: &'b Automaton<R>,
		mut f: impl FnMut(&'a Q, &'b R) -> S,
	) -> Automaton<S>
	where
		R: Ord,
		S: Clone + Ord + Hash,
	{
		let mut result = Automaton::new();

		let mut stack = Vec::with_capacity(self.initial_states.len() * other.initial_states.len());
		for a in &self.initial_states {
			for b in &other.initial_states {
				let q = f(a, b);
				stack.push((q.clone(), a, b));
				result.add_initial_state(q);
			}
		}

		let mut visited = HashSet::new();
		while let Some((q, a, b)) = stack.pop() {
			if visited.insert(q.clone()) {
				if self.is_final_state(a) && other.is_final_state(b) {
					result.add_final_state(q.clone());
				}

				let transitions = result.transitions.entry(q).or_default();

				for (a_label, a_successors) in self.successors(a) {
					match a_label {
						Some(a_label) => {
							for (b_label, b_successors) in other.successors(b) {
								if let Some(b_label) = b_label {
									let label = charset_intersection(a_label, b_label);
									if !label.is_empty() {
										let successors =
											transitions.entry(Some(label)).or_default();

										for sa in a_successors {
											for sb in b_successors {
												let s = f(sa, sb);
												stack.push((s.clone(), sa, sb));
												successors.insert(s);
											}
										}
									}
								}
							}
						}
						None => {
							if let Some(b_successors) =
								other.transitions.get(b).and_then(|s| s.get(&None))
							{
								let successors = transitions.entry(None).or_default();

								for sa in a_successors {
									for sb in b_successors {
										let s = f(sa, sb);
										stack.push((s.clone(), sa, sb));
										successors.insert(s);
									}
								}
							}
						}
					}
				}
			}
		}

		result
	}
}

/// Deterministic epsilon-free automaton.
#[derive(
	Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(bound(deserialize = "Q: Ord + serde::Deserialize<'de>, L: Ord + serde::Deserialize<'de>"))]
pub struct DetAutomaton<Q, L = AnyRange<char>> {
	initial_state: Q,
	final_states: BTreeSet<Q>,
	transitions: DetTransitions<Q, L>,
}

impl<Q, L> DetAutomaton<Q, L> {
	pub fn new(initial_state: Q) -> Self {
		Self {
			initial_state,
			final_states: BTreeSet::new(),
			transitions: DetTransitions(BTreeMap::new()),
		}
	}

	pub fn from_parts(
		initial_state: Q,
		final_states: BTreeSet<Q>,
		transitions: BTreeMap<Q, BTreeMap<L, Q>>,
	) -> Self {
		Self {
			initial_state,
			final_states,
			transitions: DetTransitions(transitions),
		}
	}

	pub fn initial_state(&self) -> &Q {
		&self.initial_state
	}

	pub fn final_states(&self) -> &BTreeSet<Q> {
		&self.final_states
	}

	pub fn final_states_mut(&mut self) -> &mut BTreeSet<Q> {
		&mut self.final_states
	}

	pub fn transitions(&self) -> &BTreeMap<Q, BTreeMap<L, Q>> {
		&self.transitions.0
	}

	pub fn reachable_states_from<'a>(&'a self, q: &'a Q) -> ReachableStates<'a, Q, L> {
		ReachableStates::new(self, q)
	}
}

impl<Q: Ord, L: Ord> DetAutomaton<Q, L> {
	pub fn is_initial_state(&self, q: &Q) -> bool {
		self.initial_state == *q
	}

	pub fn is_final_state(&self, q: &Q) -> bool {
		self.final_states.contains(q)
	}

	pub fn add_final_state(&mut self, q: Q) -> bool {
		self.final_states.insert(q)
	}

	pub fn declare_state(&mut self, q: Q) {
		self.transitions.0.entry(q).or_default();
	}

	pub fn transitions_from(&self, q: &Q) -> impl '_ + Iterator<Item = (&'_ L, &'_ Q)> {
		self.transitions.0.get(q).into_iter().flatten()
	}

	pub fn successors(&self, q: &Q) -> DetSuccessors<Q, L> {
		DetSuccessors::new(self.transitions.0.get(q))
	}

	pub fn add(&mut self, source: Q, label: L, target: Q) {
		self.transitions
			.0
			.entry(source)
			.or_default()
			.insert(label, target);
	}

	pub fn select_states<F>(&self, f: F) -> BTreeSet<&Q>
	where
		Q: Hash + Eq,
		F: Fn(&Q) -> bool,
	{
		let mut set = BTreeSet::new();
		let mut visited = HashSet::new();
		self.select_states_from(&self.initial_state, &f, &mut visited, &mut set);
		set
	}

	pub fn states(&self) -> BTreeSet<&Q>
	where
		Q: Hash + Eq,
	{
		self.select_states(|_| true)
	}

	fn select_states_from<'a, F>(
		&'a self,
		q: &'a Q,
		f: &F,
		visited: &mut HashSet<&'a Q>,
		set: &mut BTreeSet<&'a Q>,
	) where
		Q: Hash + Eq,
		F: Fn(&Q) -> bool,
	{
		if visited.insert(q) {
			if f(q) {
				set.insert(q);
			}

			for (_, r) in self.successors(q) {
				self.select_states_from(r, f, visited, set)
			}
		}
	}

	pub fn partition<P, F>(&self, f: F) -> HashMap<P, BTreeSet<&Q>>
	where
		Q: Ord + Hash + Eq,
		P: Hash + Eq,
		F: Fn(&Q) -> P,
	{
		unsafe {
			self.try_partition::<P, _, std::convert::Infallible>(|q| Ok(f(q)))
				.unwrap_unchecked() // safe because infallible.
		}
	}

	pub fn try_partition<P, F, E>(&self, f: F) -> Result<HashMap<P, BTreeSet<&Q>>, E>
	where
		Q: Ord + Hash + Eq,
		P: Hash + Eq,
		F: Fn(&Q) -> Result<P, E>,
	{
		let mut partition = HashMap::new();
		let mut visited = HashSet::new();
		self.try_partition_from(&self.initial_state, &f, &mut visited, &mut partition)?;
		Ok(partition)
	}

	fn try_partition_from<'a, P, F, E>(
		&'a self,
		q: &'a Q,
		f: &F,
		visited: &mut HashSet<&'a Q>,
		partition: &mut HashMap<P, BTreeSet<&'a Q>>,
	) -> Result<(), E>
	where
		Q: Ord + Hash + Eq,
		P: Hash + Eq,
		F: Fn(&Q) -> Result<P, E>,
	{
		if visited.insert(q) {
			let p = f(q)?;

			partition.entry(p).or_default().insert(q);

			for (_, r) in self.successors(q) {
				self.try_partition_from(r, f, visited, partition)?;
			}
		}

		Ok(())
	}

	/// Minimizes the automaton.
	// Hopcroft's algorithm.
	// https://en.wikipedia.org/wiki/DFA_minimization
	pub fn minimize<'a, P>(&'a self, partition: P) -> DetAutomaton<BTreeSet<&Q>, &L>
	where
		Q: Hash,
		L: Hash,
		P: Iterator<Item = BTreeSet<&'a Q>>,
	{
		let mut partition: BTreeSet<_> = partition.collect();

		let mut working = partition.clone();

		while let Some(a) = working.pop_first() {
			let mut sources_by_label: HashMap<&L, BTreeSet<&Q>> = HashMap::new();

			for (source, targets) in &self.transitions.0 {
				for (label, target) in targets {
					if a.contains(target) {
						if sources_by_label.contains_key(label) {
							let sources = sources_by_label.get_mut(label).unwrap();
							sources.insert(source);
						} else {
							let mut sources = BTreeSet::new();
							sources.insert(source);
							sources_by_label.insert(label, sources);
						}
					}
				}
			}

			for sources in sources_by_label.values() {
				for y in partition.clone() {
					if y.intersection(sources).next().is_some()
						&& y.difference(sources).next().is_some()
					{
						let intersection: BTreeSet<&Q> = y.intersection(sources).cloned().collect();
						let difference: BTreeSet<&Q> = y.difference(sources).cloned().collect();

						if working.contains(&y) {
							working.remove(&y);
							working.insert(intersection.clone());
							working.insert(difference.clone());
						} else if intersection.len() <= difference.len() {
							working.insert(intersection.clone());
						} else {
							working.insert(difference.clone());
						}

						partition.remove(&y);
						partition.insert(intersection);
						partition.insert(difference);
					}
				}
			}
		}

		let mut map = HashMap::new();
		for member in partition {
			for q in &member {
				map.insert(*q, member.clone());
			}
		}

		let mut result = DetAutomaton::new(map[&self.initial_state].clone());
		for (source, transitions) in &self.transitions.0 {
			for (range, target) in transitions {
				result.add(map[source].clone(), range, map[target].clone());
			}
		}

		result
	}

	pub fn map<P, M>(
		&self,
		mut f: impl FnMut(&Q) -> P,
		mut g: impl FnMut(&L) -> M,
	) -> DetAutomaton<P, M>
	where
		Q: Hash,
		L: Hash,
		P: Clone + Ord + Hash,
		M: Clone + Ord + Hash,
	{
		let mut map = HashMap::new();
		let mapped_initial_state = f(&self.initial_state);
		map.insert(&self.initial_state, mapped_initial_state.clone());

		let mut label_map = HashMap::new();

		let mut result = DetAutomaton::new(mapped_initial_state);
		for (source, transitions) in &self.transitions.0 {
			for (range, target) in transitions {
				let source = map.entry(source).or_insert_with(|| f(source)).clone();
				let target = map.entry(target).or_insert_with(|| f(target)).clone();
				let range = label_map.entry(range).or_insert_with(|| g(range)).clone();
				result.add(source, range, target);
			}
		}

		for q in &self.final_states {
			let q = map.entry(q).or_insert_with(|| f(q)).clone();
			result.add_final_state(q);
		}

		result
	}

	pub fn try_map<P, M, E>(
		&self,
		mut f: impl FnMut(&Q) -> Result<P, E>,
		mut g: impl FnMut(&L) -> Result<M, E>,
	) -> Result<DetAutomaton<P, M>, E>
	where
		Q: Hash,
		L: Hash,
		P: Clone + Ord + Hash,
		M: Clone + Ord + Hash,
	{
		let mut map = HashMap::new();
		let mapped_initial_state = f(&self.initial_state)?;
		map.insert(&self.initial_state, mapped_initial_state.clone());

		let mut label_map: HashMap<&L, M> = HashMap::new();

		let mut result = DetAutomaton::new(mapped_initial_state);
		for (source, transitions) in &self.transitions.0 {
			for (label, target) in transitions {
				let source = match map.entry(source) {
					Entry::Occupied(entry) => entry.get().clone(),
					Entry::Vacant(entry) => entry.insert(f(source)?).clone(),
				};

				let target = match map.entry(target) {
					Entry::Occupied(entry) => entry.get().clone(),
					Entry::Vacant(entry) => entry.insert(f(target)?).clone(),
				};

				let label = match label_map.entry(label) {
					Entry::Occupied(entry) => entry.get().clone(),
					Entry::Vacant(entry) => entry.insert(g(label)?).clone(),
				};

				result.add(source, label, target);
			}
		}

		Ok(result)
	}

	pub fn product<'a, 'b, R, S, M, N>(
		&'a self,
		other: &'b DetAutomaton<R, M>,
		mut f: impl FnMut(&'a Q, &'b R) -> S,
		mut g: impl FnMut(&'a L, &'b M) -> Option<N>,
	) -> DetAutomaton<S, N>
	where
		R: Ord,
		S: Clone + Ord + Hash,
		M: Ord,
		N: Ord,
	{
		let mut stack = Vec::new();
		let initial_state = f(&self.initial_state, &other.initial_state);
		stack.push((
			initial_state.clone(),
			&self.initial_state,
			&other.initial_state,
		));
		let mut result = DetAutomaton::new(initial_state);

		let mut visited = HashSet::new();
		while let Some((q, a, b)) = stack.pop() {
			if visited.insert(q.clone()) {
				if self.is_final_state(a) && other.is_final_state(b) {
					result.add_final_state(q.clone());
				}

				let transitions = result.transitions.0.entry(q).or_default();

				for (a_label, sa) in self.successors(a) {
					for (b_label, sb) in other.successors(b) {
						if let Some(label) = g(a_label, b_label) {
							let s = f(sa, sb);
							stack.push((s.clone(), sa, sb));
							transitions.insert(label, s);
						}
					}
				}
			}
		}

		result
	}

	/// Returns the single transition that follows the state `q`.
	///
	/// Returns `None` if the state has no transitions, or multiple transitions.
	fn single_transition_of(&self, q: &Q) -> Option<(&L, &Q)> {
		let mut transitions = self.transitions().get(q)?.iter();
		let first = transitions.next()?;

		match transitions.next() {
			Some(_) => None,
			None => Some(first),
		}
	}

	pub fn compress<M>(&self, append: impl Fn(&mut M, &L)) -> DetAutomaton<Q, M>
	where
		Q: Clone,
		M: Default + Ord + Clone,
	{
		let mut transitions = BTreeMap::new();
		let mut stack = vec![&self.initial_state];

		while let Some(q) = stack.pop() {
			if !transitions.contains_key(q) {
				let mut q_transitions = BTreeMap::new();

				for (label, mut r) in self.transitions.0.get(q).into_iter().flatten() {
					let mut compact_label = M::default();
					append(&mut compact_label, label);

					while let Some((label, s)) = self.single_transition_of(r) {
						if self.is_final_state(r) {
							q_transitions.insert(compact_label.clone(), r.clone());
						}

						append(&mut compact_label, label);
						r = s;
					}

					q_transitions.insert(compact_label, r.clone());
				}

				transitions.insert(q.clone(), q_transitions);
			}
		}

		DetAutomaton::from_parts(
			self.initial_state.clone(),
			self.final_states.clone(),
			transitions,
		)
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DetTransitions<Q, L>(BTreeMap<Q, BTreeMap<L, Q>>);

impl<Q, L> DetTransitions<Q, L> {
	pub fn len(&self) -> usize {
		self.0.values().fold(0, |x, map| x + map.len())
	}

	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}
}

impl<Q, L> serde::Serialize for DetTransitions<Q, L>
where
	Q: serde::Serialize,
	L: serde::Serialize,
{
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		use serde::ser::SerializeSeq;
		let mut seq = serializer.serialize_seq(Some(self.len()))?;

		for (source, map) in &self.0 {
			for (label, target) in map {
				seq.serialize_element(&(source, label, target))?;
			}
		}

		seq.end()
	}
}

impl<'de, Q, L> serde::Deserialize<'de> for DetTransitions<Q, L>
where
	Q: Ord + serde::Deserialize<'de>,
	L: Ord + serde::Deserialize<'de>,
{
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		struct Visitor<Q, L>(PhantomData<(Q, L)>);

		impl<'de, Q, L> serde::de::Visitor<'de> for Visitor<Q, L>
		where
			Q: Ord + serde::Deserialize<'de>,
			L: Ord + serde::Deserialize<'de>,
		{
			type Value = DetTransitions<Q, L>;

			fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
				write!(formatter, "deterministic automaton transitions")
			}

			fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
			where
				A: serde::de::SeqAccess<'de>,
			{
				let mut result: BTreeMap<Q, BTreeMap<L, Q>> = BTreeMap::new();

				while let Some((source, label, target)) = seq.next_element()? {
					result.entry(source).or_default().insert(label, target);
				}

				Ok(DetTransitions(result))
			}
		}

		deserializer.deserialize_seq(Visitor(PhantomData))
	}
}

pub struct Successors<'a, Q> {
	inner: Option<std::collections::btree_map::Iter<'a, Option<RangeSet<char>>, BTreeSet<Q>>>,
}

impl<'a, Q> Successors<'a, Q> {
	pub fn new(map: Option<&'a BTreeMap<Option<RangeSet<char>>, BTreeSet<Q>>>) -> Self {
		Self {
			inner: map.map(|map| map.iter()),
		}
	}
}

impl<'a, Q> Iterator for Successors<'a, Q> {
	type Item = (&'a Option<RangeSet<char>>, &'a BTreeSet<Q>);

	fn next(&mut self) -> Option<Self::Item> {
		self.inner.as_mut().and_then(|inner| inner.next())
	}
}

pub struct DetSuccessors<'a, Q, L> {
	inner: Option<std::collections::btree_map::Iter<'a, L, Q>>,
}

impl<'a, Q, L> DetSuccessors<'a, Q, L> {
	pub fn new(map: Option<&'a BTreeMap<L, Q>>) -> Self {
		Self {
			inner: map.map(|map| map.iter()),
		}
	}
}

impl<'a, Q, L> Iterator for DetSuccessors<'a, Q, L> {
	type Item = (&'a L, &'a Q);

	fn next(&mut self) -> Option<Self::Item> {
		self.inner.as_mut().and_then(|inner| inner.next())
	}
}

pub struct ReachableStates<'a, Q, L = AnyRange<char>> {
	aut: &'a DetAutomaton<Q, L>,
	visited: HashSet<&'a Q>,
	stack: Vec<&'a Q>,
}

impl<'a, Q, L> ReachableStates<'a, Q, L> {
	fn new(aut: &'a DetAutomaton<Q, L>, q: &'a Q) -> Self {
		Self {
			aut,
			visited: HashSet::new(),
			stack: vec![q],
		}
	}
}

impl<'a, Q, L> Iterator for ReachableStates<'a, Q, L>
where
	Q: Ord + Eq + Hash,
{
	type Item = &'a Q;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			match self.stack.pop() {
				Some(q) => {
					if self.visited.insert(q) {
						if let Some(q_transitions) = self.aut.transitions.0.get(q) {
							for target in q_transitions.values() {
								self.stack.push(target)
							}
						}

						break Some(q);
					}
				}
				None => break None,
			}
		}
	}
}
