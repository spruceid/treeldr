use super::Vocabulary;
use fmt::Display as StdDisplay;
use std::fmt;

pub trait Display {
	fn fmt(&self, namespace: &Vocabulary, f: &mut fmt::Formatter) -> fmt::Result;

	fn display<'n>(&self, namespace: &'n Vocabulary) -> Displayed<'n, '_, Self> {
		Displayed(namespace, self)
	}
}

pub struct Displayed<'n, 'a, T: ?Sized>(&'n Vocabulary, &'a T);

impl<'n, 'a, T: Display> fmt::Display for Displayed<'n, 'a, T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.1.fmt(self.0, f)
	}
}

impl Display for super::Id {
	fn fmt(&self, namespace: &Vocabulary, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Iri(id) => id.fmt(namespace, f),
			Self::Blank(id) => id.fmt(f),
		}
	}
}

impl<F> Display for super::Object<F> {
	fn fmt(&self, namespace: &Vocabulary, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Iri(id) => id.fmt(namespace, f),
			Self::Blank(id) => id.fmt(f),
			Self::Literal(lit) => lit.fmt(f),
		}
	}
}

impl Display for super::StrippedObject {
	fn fmt(&self, namespace: &Vocabulary, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Iri(id) => id.fmt(namespace, f),
			Self::Blank(id) => id.fmt(f),
			Self::Literal(lit) => lit.fmt(f),
		}
	}
}

impl Display for super::Name {
	fn fmt(&self, namespace: &Vocabulary, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "<{}>", self.iri(namespace).unwrap())
	}
}

impl<S: Display, P: Display, O: Display, G: Display> Display for super::Quad<S, P, O, G> {
	fn fmt(&self, namespace: &Vocabulary, f: &mut fmt::Formatter) -> fmt::Result {
		match self.graph() {
			Some(graph) => write!(
				f,
				"{} {} {} {}",
				self.subject().display(namespace),
				self.predicate().display(namespace),
				self.object().display(namespace),
				graph.display(namespace)
			),
			None => write!(
				f,
				"{} {} {}",
				self.subject().display(namespace),
				self.predicate().display(namespace),
				self.object().display(namespace)
			),
		}
	}
}

impl<T: Display, F> Display for locspan::Loc<T, F> {
	fn fmt(&self, namespace: &Vocabulary, f: &mut fmt::Formatter) -> fmt::Result {
		self.value().fmt(namespace, f)
	}
}

impl<'a, T: Display> Display for &'a T {
	fn fmt(&self, namespace: &Vocabulary, f: &mut fmt::Formatter) -> fmt::Result {
		(*self).fmt(namespace, f)
	}
}
