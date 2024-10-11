use std::ops::Deref;

use rdf_types::dataset::PatternMatchingDataset;

use crate::{
	eval::{EvalError, RdfContext}, Domain, Function, Value
};

/// Layout.
///
/// A layout is a subclass of function where each input is an RDF resource.
pub struct Layout<R>(Function<R>);

impl<R: Clone + Ord> Layout<R> {
	pub fn hydrate(
		&self,
		rdf: &impl RdfContext<R>,
		dataset: &impl PatternMatchingDataset<Resource = R>,
		args: &[R],
	) -> Result<Value<R>, EvalError<R>> {
		let args: Vec<_> = args.iter().map(|r| Value::Resource(r.clone())).collect();
		self.0.call(rdf, dataset, &args)
	}
}

impl<R> Deref for Layout<R> {
	type Target = Function<R>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<R> Domain for Layout<R> {
	type Resource = R;
}
