use crate::{
	function::BuiltInFunction,
	ty::{BuiltInType, LiteralType, TypeDesc},
	Function, Item, Module, Registry,
};

impl<R> Module<R> {
	pub fn prelude(registry: &mut Registry<R>) -> Self {
		let n = registry.create_namespace();
		let mut result = Self::new();

		let ty_any = registry.insert_type(n, TypeDesc::Any);
		let ty_resource = registry.insert_type(n, TypeDesc::resource());
		let ty_unit = registry.insert_type(n, TypeDesc::Literal(LiteralType::Unit));
		let ty_bool = registry.insert_type(n, TypeDesc::Literal(LiteralType::boolean()));
		let ty_number = registry.insert_type(n, TypeDesc::Literal(LiteralType::number()));
		let ty_int = registry.insert_type(n, TypeDesc::Literal(LiteralType::int()));
		let ty_uint = registry.insert_type(n, TypeDesc::Literal(LiteralType::uint()));
		let ty_string = registry.insert_type(n, TypeDesc::Literal(LiteralType::text_string()));
		let ty_bytes = registry.insert_type(n, TypeDesc::Literal(LiteralType::byte_string()));

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
				Item::Function(registry.insert_function(n, Function::built_in(f, ty))),
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
