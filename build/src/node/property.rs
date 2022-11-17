pub use treeldr::Property;

pub enum BindingRef<'a, M> {
	Resource(crate::resource::BindingRef<'a, M>),
	Class(crate::ty::BindingRef<'a, M>),
	DatatypeRestriction(crate::ty::datatype::restriction::BindingRef<'a, M>),
	Property(crate::prop::BindingRef<'a, M>),
	// Layout(crate::layout::BindingRef<'a, M>),
	// LayoutField(crate::layout::field::BindingRef<'a, M>),
	// LayoutVariant(crate::layout::variant::BindingRef<'a, M>),
	LayoutRestriction(crate::layout::restriction::BindingRef<'a, M>),
	List(crate::list::BindingRef<'a, M>)
}

/// Iterator over the bindings of a given node.
pub struct Bindings<'a, M> {
	resource: crate::resource::Bindings<'a, M>,
	class: crate::ty::Bindings<'a, M>,
	datatype_restriction: crate::ty::datatype::restriction::Bindings<'a, M>,
	property: crate::prop::Bindings<'a, M>,
	// layout: crate::layout::Bindings<'a, M>,
	// layout_field: crate::layout::field::Bindings<'a, M>,
	// layout_variant: crate::layout::variant::Bindings<'a, M>,
	layout_restriction: crate::layout::restriction::Bindings<'a, M>,
	list: crate::list::Bindings<'a, M>,
}

impl<'a, M> Iterator for Bindings<'a, M> {
	type Item = BindingRef<'a, M>;

	fn next(&mut self) -> Option<Self::Item> {
		todo!()
		// self.resource
		// 	.next()
		// 	.map(BindingRef::Resource)
		// 	.or_else(|| {
		// 		self.class
		// 			.next()
		// 			.map(BindingRef::Class)
		// 			.or_else(|| {
		// 				self.datatype_restriction
		// 					.next()
		// 					.map(BindingRef::DatatypeRestriction)
		// 					.or_else(|| {
		// 						self.property
		// 							.next()
		// 							.map(BindingRef::Property)
		// 							.or_else(|| {
		// 								self.layout
		// 									.next()
		// 									.map(BindingRef::Layout)
		// 									.or_else(|| {
		// 										self.layout_field
		// 											.next()
		// 											.map(BindingRef::LayoutField)
		// 											.or_else(|| {
		// 												self.layout_variant
		// 													.next()
		// 													.map(BindingRef::LayoutVariant)
		// 													.or_else(|| {
		// 														self.layout_restriction
		// 															.next()
		// 															.map(BindingRef::LayoutRestriction)
		// 															.or_else(|| {
		// 																self.list
		// 																	.next()
		// 																	.map(BindingRef::List)
		// 															})
		// 													})
		// 											})
		// 									})
		// 							})
		// 					})
		// 			})
		// 	})
	}
}