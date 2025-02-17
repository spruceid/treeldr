use rdf_types::interpretation::ConstGenerativeInterpretation;

pub struct RootContext<'a, I: ?Sized, D: ?Sized> {
	pub interpretation: &'a I,
	pub dataset: &'a D,
}

impl<'a, I: ?Sized, D: ?Sized> RootContext<'a, I, D> {
	pub fn new(interpretation: &'a I, dataset: &'a D) -> Self {
		Self {
			interpretation,
			dataset,
		}
	}
}

impl<I, D> RootContext<'_, I, D>
where
	I: ?Sized + ConstGenerativeInterpretation,
	D: ?Sized,
{
	pub fn new_resource(&self) -> I::Resource {
		self.interpretation.new_resource()
	}
}
