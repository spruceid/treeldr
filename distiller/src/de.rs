use rdf_types::Interpretation;
use treeldr::{layout::LayoutType, Layout, Layouts, Ref, Value};

pub enum Error {
	IncompatibleLayout,
	AbstractLayout,
}

pub type Dehydrated<R> = (grdf::BTreeDataset<R>, Vec<R>);

/// Deserialize the given `value` according to the provided `layout`, returning
/// the deserialized RDF dataset.
pub fn dehydrate<V, I: Interpretation>(
	_vocabulary: &V,
	_interpretation: &I,
	context: &Layouts<I::Resource>,
	_value: &Value,
	layout_ref: &Ref<LayoutType, I::Resource>,
) -> Result<Dehydrated<I::Resource>, Error>
where
	I::Resource: Ord,
{
	match context.get(layout_ref).unwrap() {
		Layout::Never => Err(Error::IncompatibleLayout),
		Layout::Literal(_) => {
			todo!()
		}
		Layout::Sum(_) => {
			todo!()
		}
		Layout::Product(_) => {
			todo!()
		}
		Layout::List(_) => {
			todo!()
		}
		Layout::Always => Err(Error::AbstractLayout),
	}
}
