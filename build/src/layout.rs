/// Layout.
pub enum Layout<R> {
	Never,
	Literal(LiteralLayout<R>),
	Product(ProductLayout<R>),
	List(ListLayout),
	Sum(SumLayout<R>),
	Always,
	Union(Vec<Ref<R, Layout<R>>>),
	Intersection(Vec<Ref<R, Layout<R>>>)
}