use iref::{IriBuf, IriRef, IriRefBuf};
use locspan::{Loc, Location};
use rdf_types::{loc::Literal, Quad};
use std::collections::HashMap;

use treeldr_vocab::*;

pub trait Build<F> {
	type Target;

	fn build(
		self,
		ctx: &mut Context<F>,
		quads: &mut Vec<LocQuad<F>>,
	) -> Result<Self::Target, Loc<Error<F>, F>>;
}

#[derive(Debug)]
pub enum Error<F> {
	Undefined(crate::Id),
	InvalidExpandedCompactIri(String),
	UndefinedPrefix(String),
	AlreadyDefinedPrefix(String, Location<F>),
}

pub struct Context<F> {
	base_iri: IriBuf,
	namespace: Vocabulary,
	prefixes: HashMap<String, Loc<IriBuf, F>>,
	scope: Option<Name>,
}

impl<F: Clone> Context<F> {
	pub fn new(base_iri: IriBuf) -> Self {
		Self {
			base_iri,
			namespace: Vocabulary::new(),
			prefixes: HashMap::new(),
			scope: None,
		}
	}

	pub fn namespace(&self) -> &Vocabulary {
		&self.namespace
	}

	pub fn into_namespace(self) -> Vocabulary {
		self.namespace
	}

	pub fn base_iri(&self) -> IriBuf {
		match &self.scope {
			Some(scope) => {
				let mut iri = scope.iri(&self.namespace).unwrap().to_owned();
				iri.path_mut().open();
				iri
			}
			None => self.base_iri.clone(),
		}
	}

	pub fn declare_prefix(
		&mut self,
		prefix: String,
		iri: IriBuf,
		loc: Location<F>,
	) -> Result<(), Loc<Error<F>, F>> {
		use std::collections::hash_map::Entry;
		match self.prefixes.entry(prefix) {
			Entry::Occupied(entry) => Err(Loc(
				Error::AlreadyDefinedPrefix(entry.key().to_owned(), entry.get().location().clone()),
				loc,
			)),
			Entry::Vacant(entry) => {
				entry.insert(Loc(iri, loc));
				Ok(())
			}
		}
	}

	pub fn expand_compact_iri(
		&self,
		prefix: &str,
		iri_ref: IriRef,
		loc: &Location<F>,
	) -> Result<IriBuf, Loc<Error<F>, F>> {
		match self.prefixes.get(prefix) {
			Some(iri) => match IriBuf::try_from(iri.as_str().to_string() + iri_ref.as_str()) {
				Ok(iri) => Ok(iri),
				Err((_, string)) => Err(Loc(Error::InvalidExpandedCompactIri(string), loc.clone())),
			},
			None => Err(Loc(Error::UndefinedPrefix(prefix.to_owned()), loc.clone())),
		}
	}
}

impl<F: Clone> Build<F> for Loc<crate::Document<F>, F> {
	type Target = ();

	fn build(
		self,
		ctx: &mut Context<F>,
		quads: &mut Vec<LocQuad<F>>,
	) -> Result<(), Loc<Error<F>, F>> {
		let Loc(doc, _) = self;

		for import in doc.imports {
			import.build(ctx, quads)?
		}

		for ty in doc.types {
			ty.build(ctx, quads)?
		}

		for layout in doc.layouts {
			layout.build(ctx, quads)?
		}

		Ok(())
	}
}

impl<F: Clone> Build<F> for Loc<crate::Id, F> {
	type Target = Loc<Name, F>;

	fn build(
		self,
		ctx: &mut Context<F>,
		_quads: &mut Vec<LocQuad<F>>,
	) -> Result<Self::Target, Loc<Error<F>, F>> {
		let Loc(id, loc) = self;
		let iri = match id {
			crate::Id::Name(name) => {
				let mut iri_ref = IriRefBuf::from_string(name).unwrap();
				iri_ref.resolve(ctx.base_iri().as_iri());
				iri_ref.try_into().unwrap()
			}
			crate::Id::IriRef(iri_ref) => iri_ref.resolved(ctx.base_iri().as_iri()),
			crate::Id::Compact(prefix, iri_ref) => {
				ctx.expand_compact_iri(&prefix, iri_ref.as_iri_ref(), &loc)?
			}
		};

		Ok(Loc(Name::from_iri(iri, &mut ctx.namespace), loc))
	}
}

impl<F: Clone> Build<F> for Loc<crate::Import<F>, F> {
	type Target = ();

	fn build(
		self,
		ctx: &mut Context<F>,
		_quads: &mut Vec<LocQuad<F>>,
	) -> Result<(), Loc<Error<F>, F>> {
		let Loc(import, loc) = self;
		ctx.declare_prefix(
			import.prefix.into_value().into_string(),
			import.iri.into_value(),
			loc,
		)
	}
}

fn build_doc<F: Clone>(
	Loc(doc, loc): Loc<crate::Documentation<F>, F>,
	subject: Loc<Id, F>,
	quads: &mut Vec<LocQuad<F>>,
) {
	let mut short = String::new();
	let mut short_loc = loc.clone();

	let mut long = String::new();
	let mut long_loc = loc.clone();

	let mut separated = false;

	for Loc(line, line_loc) in doc.items {
		let line = line.trim();

		if separated {
			if long.is_empty() {
				long_loc = line_loc;
			} else {
				long_loc.span_mut().append(line_loc.span());
			}

			long.push_str(line);
		} else if line.is_empty() {
			separated = true
		} else {
			if short.is_empty() {
				short_loc = line_loc;
			} else {
				short_loc.span_mut().append(line_loc.span());
			}

			short.push_str(line);
		}
	}

	if !long.is_empty() {
		quads.push(Loc(
			Quad(
				subject.clone(),
				Loc(Name::Rdfs(Rdfs::Comment), loc.clone()),
				Loc(
					Object::Literal(Literal::String(Loc(long.into(), long_loc.clone()))),
					long_loc,
				),
				None,
			),
			loc.clone(),
		))
	}

	if !short.is_empty() {
		quads.push(Loc(
			Quad(
				subject,
				Loc(Name::Rdfs(Rdfs::Comment), loc.clone()),
				Loc(
					Object::Literal(Literal::String(Loc(short.into(), short_loc.clone()))),
					short_loc,
				),
				None,
			),
			loc,
		))
	}
}

impl<F: Clone> Build<F> for Loc<crate::TypeDefinition<F>, F> {
	type Target = ();

	fn build(
		self,
		ctx: &mut Context<F>,
		quads: &mut Vec<LocQuad<F>>,
	) -> Result<(), Loc<Error<F>, F>> {
		let implicit_layout = Loc(self.implicit_layout_definition(), self.location().clone());

		let Loc(def, _) = self;
		let Loc(id, id_loc) = def.id.build(ctx, quads)?;

		quads.push(Loc(
			Quad(
				Loc(Id::Iri(id), id_loc.clone()),
				Loc(Name::Rdf(Rdf::Type), id_loc.clone()),
				Loc(Object::Iri(Name::Rdfs(Rdfs::Class)), id_loc.clone()),
				None,
			),
			id_loc.clone(),
		));

		if let Some(doc) = def.doc {
			build_doc(doc, Loc(Id::Iri(id), id_loc.clone()), quads)
		}

		for property in def.properties.into_value() {
			ctx.scope = Some(id);
			let Loc(prop, prop_loc) = property.build(ctx, quads)?;
			ctx.scope = None;

			quads.push(Loc(
				Quad(
					Loc(Id::Iri(prop), prop_loc.clone()),
					Loc(Name::Rdfs(Rdfs::Domain), prop_loc.clone()),
					Loc(Object::Iri(id), id_loc.clone()),
					None,
				),
				prop_loc,
			));
		}

		implicit_layout.build(ctx, quads)
	}
}

impl<F: Clone> Build<F> for Loc<crate::PropertyDefinition<F>, F> {
	type Target = Loc<Name, F>;

	fn build(
		self,
		ctx: &mut Context<F>,
		quads: &mut Vec<LocQuad<F>>,
	) -> Result<Self::Target, Loc<Error<F>, F>> {
		let Loc(def, loc) = self;
		let Loc(id, id_loc) = def.id.build(ctx, quads)?;

		quads.push(Loc(
			Quad(
				Loc(Id::Iri(id), id_loc.clone()),
				Loc(Name::Rdf(Rdf::Type), id_loc.clone()),
				Loc(Object::Iri(Name::Rdf(Rdf::Property)), id_loc.clone()),
				None,
			),
			id_loc.clone(),
		));

		if let Some(doc) = def.doc {
			build_doc(doc, Loc(Id::Iri(id), id_loc.clone()), quads)
		}

		if let Some(Loc(ty, _)) = def.ty {
			let scope = ctx.scope.take();
			let object = ty.expr.build(ctx, quads)?;
			ctx.scope = scope;

			quads.push(Loc(
				Quad(
					Loc(Id::Iri(id), id_loc.clone()),
					Loc(Name::Rdfs(Rdfs::Range), object.location().clone()),
					object,
					None,
				),
				loc,
			));

			for Loc(ann, ann_loc) in ty.annotations {
				match ann {
					crate::Annotation::Multiple => quads.push(Loc(
						Quad(
							Loc(Id::Iri(id), id_loc.clone()),
							Loc(Name::Schema(Schema::MultipleValues), ann_loc.clone()),
							Loc(Object::Iri(Name::Schema(Schema::True)), ann_loc.clone()),
							None,
						),
						ann_loc,
					)),
					crate::Annotation::Required => quads.push(Loc(
						Quad(
							Loc(Id::Iri(id), id_loc.clone()),
							Loc(Name::Schema(Schema::ValueRequired), ann_loc.clone()),
							Loc(Object::Iri(Name::Schema(Schema::True)), ann_loc.clone()),
							None,
						),
						ann_loc,
					)),
				}
			}
		}

		Ok(Loc(id, id_loc))
	}
}

impl<F: Clone> Build<F> for Loc<crate::TypeExpr<F>, F> {
	type Target = Loc<Object<F>, F>;

	fn build(
		self,
		ctx: &mut Context<F>,
		quads: &mut Vec<LocQuad<F>>,
	) -> Result<Self::Target, Loc<Error<F>, F>> {
		let Loc(ty, loc) = self;

		match ty {
			crate::TypeExpr::Id(id) => {
				let Loc(id, _) = id.build(ctx, quads)?;
				Ok(Loc(Object::Iri(id), loc))
			}
			crate::TypeExpr::Reference(r) => r.build(ctx, quads),
		}
	}
}

impl<F: Clone> Build<F> for Loc<crate::LayoutDefinition<F>, F> {
	type Target = ();

	fn build(
		self,
		ctx: &mut Context<F>,
		quads: &mut Vec<LocQuad<F>>,
	) -> Result<(), Loc<Error<F>, F>> {
		let Loc(def, _) = self;
		let Loc(id, id_loc) = def.id.build(ctx, quads)?;
		let Loc(ty_id, ty_id_loc) = def.ty_id.build(ctx, quads)?;

		quads.push(Loc(
			Quad(
				Loc(Id::Iri(id), id_loc.clone()),
				Loc(Name::Rdf(Rdf::Type), id_loc.clone()),
				Loc(Object::Iri(Name::TreeLdr(TreeLdr::Layout)), id_loc.clone()),
				None,
			),
			id_loc.clone(),
		));

		let for_loc = id_loc.clone().with(ty_id_loc.span());
		quads.push(Loc(
			Quad(
				Loc(Id::Iri(id), id_loc.clone()),
				Loc(Name::TreeLdr(TreeLdr::LayoutFor), id_loc.clone()),
				Loc(Object::Iri(ty_id), ty_id_loc),
				None,
			),
			for_loc,
		));

		if let Some(doc) = def.doc {
			build_doc(doc, Loc(Id::Iri(id), id_loc.clone()), quads)
		}

		let Loc(fields, fields_loc) = def.fields;
		let mut fields_head = Loc(Object::Iri(Name::Rdf(Rdf::Nil)), fields_loc);
		for field in fields.into_iter().rev() {
			ctx.scope = Some(ty_id);

			let item_label = ctx.namespace.new_blank_label();
			let first = field.build(ctx, quads)?;
			let first_loc = first.location().clone();

			quads.push(Loc(
				Quad(
					Loc(Id::Blank(item_label), first_loc.clone()),
					Loc(Name::Rdf(Rdf::First), first_loc.clone()),
					first,
					None,
				),
				first_loc.clone(),
			));

			quads.push(Loc(
				Quad(
					Loc(Id::Blank(item_label), first_loc.clone()),
					Loc(Name::Rdf(Rdf::Rest), first_loc.clone()),
					fields_head,
					None,
				),
				first_loc.clone(),
			));

			fields_head = Loc(Object::Blank(item_label), first_loc);

			ctx.scope = None;
		}

		quads.push(Loc(
			Quad(
				Loc(Id::Iri(id), id_loc.clone()),
				Loc(Name::TreeLdr(TreeLdr::Fields), id_loc.clone()),
				fields_head,
				None,
			),
			id_loc,
		));

		Ok(())
	}
}

impl<F: Clone> Build<F> for Loc<crate::FieldDefinition<F>, F> {
	type Target = Loc<Object<F>, F>;

	fn build(
		self,
		ctx: &mut Context<F>,
		quads: &mut Vec<LocQuad<F>>,
	) -> Result<Self::Target, Loc<Error<F>, F>> {
		let Loc(def, loc) = self;

		let label = ctx.namespace.new_blank_label();
		let Loc(prop_id, prop_id_loc) = def.id.build(ctx, quads)?;

		quads.push(Loc(
			Quad(
				Loc(Id::Blank(label), loc.clone()),
				Loc(Name::Rdf(Rdf::Type), loc.clone()),
				Loc(Object::Iri(Name::TreeLdr(TreeLdr::Field)), loc.clone()),
				None,
			),
			loc.clone(),
		));

		quads.push(Loc(
			Quad(
				Loc(Id::Blank(label), prop_id_loc.clone()),
				Loc(Name::TreeLdr(TreeLdr::FieldFor), prop_id_loc.clone()),
				Loc(Object::Iri(prop_id), prop_id_loc.clone()),
				None,
			),
			prop_id_loc.clone(),
		));

		if let Some(doc) = def.doc {
			build_doc(doc, Loc(Id::Blank(label), prop_id_loc.clone()), quads)
		}

		let Loc(name, name_loc) = match def.alias {
			Some(Loc(alias, alias_loc)) => Loc(alias.into_string(), alias_loc),
			None => Loc(
				prop_id
					.iri(&ctx.namespace)
					.unwrap()
					.path()
					.file_name()
					.expect("invalid property IRI")
					.to_owned(),
				prop_id_loc.clone(),
			),
		};

		quads.push(Loc(
			Quad(
				Loc(Id::Blank(label), prop_id_loc.clone()),
				Loc(Name::Rdfs(Rdfs::Label), prop_id_loc.clone()),
				Loc(
					Object::Literal(Literal::String(Loc(name.into(), name_loc.clone()))),
					name_loc.clone(),
				),
				None,
			),
			name_loc,
		));

		if let Some(Loc(layout, _)) = def.layout {
			let scope = ctx.scope.take();
			let object = layout.expr.build(ctx, quads)?;
			ctx.scope = scope;
			quads.push(Loc(
				Quad(
					Loc(Id::Blank(label), prop_id_loc.clone()),
					Loc(Name::Rdfs(Rdfs::Range), object.location().clone()),
					object,
					None,
				),
				loc.clone(),
			));

			for Loc(ann, ann_loc) in layout.annotations {
				match ann {
					crate::Annotation::Multiple => quads.push(Loc(
						Quad(
							Loc(Id::Blank(label), prop_id_loc.clone()),
							Loc(Name::Schema(Schema::MultipleValues), ann_loc.clone()),
							Loc(Object::Iri(Name::Schema(Schema::True)), ann_loc.clone()),
							None,
						),
						ann_loc,
					)),
					crate::Annotation::Required => quads.push(Loc(
						Quad(
							Loc(Id::Blank(label), prop_id_loc.clone()),
							Loc(Name::Schema(Schema::ValueRequired), ann_loc.clone()),
							Loc(Object::Iri(Name::Schema(Schema::True)), ann_loc.clone()),
							None,
						),
						ann_loc,
					)),
				}
			}
		}

		Ok(Loc(Object::Blank(label), loc))
	}
}

impl<F: Clone> Build<F> for Loc<crate::LayoutExpr<F>, F> {
	type Target = Loc<Object<F>, F>;

	fn build(
		self,
		ctx: &mut Context<F>,
		quads: &mut Vec<LocQuad<F>>,
	) -> Result<Self::Target, Loc<Error<F>, F>> {
		let Loc(ty, loc) = self;

		match ty {
			crate::LayoutExpr::Id(id) => {
				let Loc(id, _) = id.build(ctx, quads)?;
				Ok(Loc(Object::Iri(id), loc))
			}
			crate::LayoutExpr::Reference(r) => {
				let deref_ty = r.build(ctx, quads)?;
				let ty = ctx.namespace.new_blank_label();
				quads.push(Loc(
					Quad(
						Loc(Id::Blank(ty), loc.clone()),
						Loc(Name::Rdf(Rdf::Type), loc.clone()),
						Loc(Object::Iri(Name::TreeLdr(TreeLdr::Layout)), loc.clone()),
						None,
					),
					loc.clone(),
				));
				quads.push(Loc(
					Quad(
						Loc(Id::Blank(ty), loc.clone()),
						Loc(Name::TreeLdr(TreeLdr::DerefTo), loc.clone()),
						deref_ty,
						None,
					),
					loc.clone(),
				));
				Ok(Loc(Object::Blank(ty), loc))
			}
		}
	}
}
