use crate::{
	function::BuiltInFunction,
	module::Item,
	ty::{BuiltInType, LiteralType, TypeDesc},
	Function, Module, Package,
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

		let ty_any = package.insert_type(TypeDesc::Any);
		let ty_resource = package.insert_type(TypeDesc::resource());
		let ty_unit = package.insert_type(TypeDesc::Literal(LiteralType::Unit));
		let ty_bool = package.insert_type(TypeDesc::Literal(LiteralType::boolean()));
		let ty_number = package.insert_type(TypeDesc::Literal(LiteralType::number()));
		let ty_int = package.insert_type(TypeDesc::Literal(LiteralType::int()));
		let ty_uint = package.insert_type(TypeDesc::Literal(LiteralType::uint()));
		let ty_string = package.insert_type(TypeDesc::Literal(LiteralType::text_string()));
		let ty_bytes = package.insert_type(TypeDesc::Literal(LiteralType::byte_string()));

		result.insert("any", Item::Type(ty_any));
		result.insert("resource", Item::Type(ty_resource));
		result.insert("unit", Item::Type(ty_unit));
		result.insert("bool", Item::Type(ty_bool));
		result.insert("number", Item::Type(ty_number));
		result.insert("int", Item::Type(ty_int));
		result.insert("uint", Item::Type(ty_uint));
		result.insert("string", Item::Type(ty_string));
		result.insert("bytes", Item::Type(ty_bytes));

		let ty = |ty: BuiltInType| match ty {
			BuiltInType::Any => ty_any,
			BuiltInType::Resource => ty_resource,
			BuiltInType::Unit => ty_unit,
			BuiltInType::Boolean => ty_bool,
			BuiltInType::Number => ty_number,
			BuiltInType::Int => ty_int,
			BuiltInType::Uint => ty_uint,
			BuiltInType::TextString => ty_string,
			BuiltInType::ByteString => ty_bytes,
		};

		let mut declare = |name, f| {
			result.insert(
				name,
				Item::Function(package.insert_function(Function::built_in(f, ty))),
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
