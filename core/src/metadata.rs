pub enum Metadata<M> {
	BuiltIn,
	Extern(M),
}

pub trait Merge {
	fn merge_with(&mut self, other: Self);

	fn merged_with(mut self, other: Self) -> Self
	where
		Self: Sized,
	{
		self.merge_with(other);
		self
	}
}

pub trait MaybeLocated {
	type File;

	fn location(&self) -> Option<locspan::Location<Self::File>>;
}
