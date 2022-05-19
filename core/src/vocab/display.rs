use super::Vocabulary;
use fmt::Display as StdDisplay;
use locspan::Loc;
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

impl<F> Display for super::Literal<F> {
	fn fmt(&self, namespace: &Vocabulary, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::String(s) => s.fmt(f),
			Self::LangString(Loc(s, _), Loc(tag, _)) => write!(f, "{}@{}", s, tag),
			Self::TypedString(Loc(s, _), Loc(ty, _)) => {
				write!(f, "{}^^<{}>", s, ty.display(namespace))
			}
		}
	}
}

impl Display for super::StrippedLiteral {
	fn fmt(&self, namespace: &Vocabulary, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::String(s) => s.fmt(f),
			Self::LangString(s, tag) => write!(f, "{}@{}", s, tag),
			Self::TypedString(s, ty) => write!(f, "{}^^<{}>", s, ty.display(namespace)),
		}
	}
}

impl<F> Display for super::Object<F> {
	fn fmt(&self, namespace: &Vocabulary, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Iri(id) => id.fmt(namespace, f),
			Self::Blank(id) => id.fmt(f),
			Self::Literal(lit) => lit.fmt(namespace, f),
		}
	}
}

impl Display for super::StrippedObject {
	fn fmt(&self, namespace: &Vocabulary, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Iri(id) => id.fmt(namespace, f),
			Self::Blank(id) => id.fmt(f),
			Self::Literal(lit) => lit.fmt(namespace, f),
		}
	}
}

impl Display for super::Term {
	fn fmt(&self, namespace: &Vocabulary, f: &mut fmt::Formatter) -> fmt::Result {
		self.iri(namespace).unwrap().fmt(f)
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

pub trait RdfDisplay {
	fn rdf_fmt(&self, namespace: &Vocabulary, f: &mut fmt::Formatter) -> fmt::Result;

	fn rdf_display<'n>(&self, namespace: &'n Vocabulary) -> RdfDisplayed<'n, '_, Self> {
		RdfDisplayed(namespace, self)
	}
}

pub struct RdfDisplayed<'n, 'a, T: ?Sized>(&'n Vocabulary, &'a T);

impl<'n, 'a, T: RdfDisplay> fmt::Display for RdfDisplayed<'n, 'a, T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		self.1.rdf_fmt(self.0, f)
	}
}

impl RdfDisplay for super::Id {
	fn rdf_fmt(&self, namespace: &Vocabulary, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Iri(id) => id.rdf_fmt(namespace, f),
			Self::Blank(id) => id.fmt(f),
		}
	}
}

impl<F> RdfDisplay for super::Object<F> {
	fn rdf_fmt(&self, namespace: &Vocabulary, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Iri(id) => id.rdf_fmt(namespace, f),
			Self::Blank(id) => id.fmt(f),
			Self::Literal(lit) => lit.fmt(namespace, f),
		}
	}
}

impl RdfDisplay for super::StrippedObject {
	fn rdf_fmt(&self, namespace: &Vocabulary, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::Iri(id) => id.rdf_fmt(namespace, f),
			Self::Blank(id) => id.fmt(f),
			Self::Literal(lit) => lit.fmt(namespace, f),
		}
	}
}

impl RdfDisplay for super::Term {
	fn rdf_fmt(&self, namespace: &Vocabulary, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "<{}>", self.iri(namespace).unwrap())
	}
}

impl<S: RdfDisplay, P: RdfDisplay, O: RdfDisplay, G: RdfDisplay> RdfDisplay
	for super::Quad<S, P, O, G>
{
	fn rdf_fmt(&self, namespace: &Vocabulary, f: &mut fmt::Formatter) -> fmt::Result {
		match self.graph() {
			Some(graph) => write!(
				f,
				"{} {} {} {}",
				self.subject().rdf_display(namespace),
				self.predicate().rdf_display(namespace),
				self.object().rdf_display(namespace),
				graph.rdf_display(namespace)
			),
			None => write!(
				f,
				"{} {} {}",
				self.subject().rdf_display(namespace),
				self.predicate().rdf_display(namespace),
				self.object().rdf_display(namespace)
			),
		}
	}
}

impl<T: RdfDisplay, F> RdfDisplay for locspan::Loc<T, F> {
	fn rdf_fmt(&self, namespace: &Vocabulary, f: &mut fmt::Formatter) -> fmt::Result {
		self.value().rdf_fmt(namespace, f)
	}
}

impl<'a, T: RdfDisplay> RdfDisplay for &'a T {
	fn rdf_fmt(&self, namespace: &Vocabulary, f: &mut fmt::Formatter) -> fmt::Result {
		(*self).rdf_fmt(namespace, f)
	}
}
