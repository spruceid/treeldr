pub trait Automaton {
	fn initial_state(&self) -> usize;

	fn next_state(&self, state: usize, c: char) -> Option<usize>;

	fn is_final_state(&self, state: usize) -> bool;
}

pub trait Pattern<A> {
	fn check(&self, value: &A) -> bool;
}

impl<A: Automaton> Pattern<A> for String {
	fn check(&self, automaton: &A) -> bool {
		let mut state = automaton.initial_state();

		for c in self.chars() {
			match automaton.next_state(state, c) {
				Some(next) => state = next,
				None => return false,
			}
		}

		automaton.is_final_state(state)
	}
}
