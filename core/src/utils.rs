pub mod automaton;

pub use automaton::{Automaton, DetAutomaton};

use btree_range_map::RangeSet;

pub fn charset_intersection(a: &RangeSet<char>, b: &RangeSet<char>) -> RangeSet<char> {
	let mut result = a.clone();

	for r in b.gaps() {
		result.remove(r.cloned());
	}

	result
}
