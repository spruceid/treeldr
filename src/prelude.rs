use crate::{
	function::BuiltInFunction, module::Item, ty::BuiltInTypeMap, Function, Module, Package,
};

impl<P: Copy, R> Package<P, R> {
	pub fn prelude(id: P) -> Self {
		let mut result = Self::new(id);
		let root = Module::prelude(&mut result);
		result.set_root(root);
		result
	}
}

impl<P: Copy, R> Module<P, R> {
	pub fn prelude(package: &mut Package<P, R>) -> Self {
		let mut result = Self::new();

		let types = BuiltInTypeMap::new(|ty| package.insert_type(ty.into_description()));

		for (ty, &ty_ref) in &types {
			result.insert(ty.name(), Item::Type(ty_ref));
		}

		let mut declare = |name, f| {
			result.insert(
				name,
				Item::Function(package.insert_function(Function::built_in(f, &types))),
			)
		};

		declare("get", BuiltInFunction::Get);
		declare("iri_of", BuiltInFunction::IriOf);
		declare("bool_of", BuiltInFunction::BoolOf);
		declare("typed_bool_of", BuiltInFunction::TypedBoolOf);
		declare("number_of", BuiltInFunction::NumberOf);
		declare("typed_number_of", BuiltInFunction::TypedNumberOf);
		declare("int_of", BuiltInFunction::IntOf);
		declare("typed_int_of", BuiltInFunction::TypedIntOf);
		declare("uint_of", BuiltInFunction::UintOf);
		declare("typed_uint_of", BuiltInFunction::TypedUintOf);
		declare("string_of", BuiltInFunction::StringOf);
		declare("typed_string_of", BuiltInFunction::TypedStringOf);

		result
	}
}
