use crate::{function::BuiltInFunction, ty::{BuiltInType, LiteralType}, Function, Item, Module, Registry, Type};

impl Module {
	pub fn prelude<R>(registry: &mut Registry<R>) -> Self {
		let n = registry.create_namespace();
		let mut result = Self::new();

		let ty_any = registry.insert_type(n, Type::Any);
		let ty_resource = registry.insert_type(n, Type::resource());
		let ty_unit = registry.insert_type(n, Type::Literal(LiteralType::Unit));
		let ty_bool = registry.insert_type(n, Type::Literal(LiteralType::boolean()));
		let ty_number = registry.insert_type(n, Type::Literal(LiteralType::number()));
		let ty_string = registry.insert_type(n, Type::Literal(LiteralType::text_string()));
		let ty_bytes = registry.insert_type(n, Type::Literal(LiteralType::byte_string()));

		result.insert("any", Item::Type(ty_any));
		result.insert("resource", Item::Type(ty_resource));
		result.insert("unit", Item::Type(ty_unit));
		result.insert("bool", Item::Type(ty_bool));
		result.insert("number", Item::Type(ty_number));
		result.insert("string", Item::Type(ty_string));
		result.insert("bytes", Item::Type(ty_bytes));
		
		let ty = |ty: BuiltInType| match ty {
			BuiltInType::Any => ty_any,
			BuiltInType::Resource => ty_resource,
			BuiltInType::Unit => ty_unit,
			BuiltInType::Boolean => ty_bool,
			BuiltInType::Number => ty_number,
			BuiltInType::TextString => ty_string,
			BuiltInType::ByteString => ty_bytes
		};

		result.insert("string_of", Item::Function(registry.insert_function(n, Function::built_in(BuiltInFunction::StringOf, ty))));
		result
	}
}