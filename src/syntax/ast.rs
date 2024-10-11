#[derive(Debug, ll1::Token)]
pub enum TokenKind {
	#[token(skip, whitespace)]
	Whitespace,

	#[token(skip, regexp("#[^\n]*"))]
	Comment,

	#[token(regexp("([a-zA-Z_][a-zA-Z0-9_]*)?:[a-zA-Z0-9_:/?#%+\\-]+"))]
	CompactIri,

	#[token(regexp("<[^>]*>"))]
	FullIri,

	#[token(ident)]
	Ident,

	#[token("prefix")]
	Prefix,

	#[token("fn")]
	Fn,

	#[token("enum")]
	Enum,

	#[token("match")]
	Match,

	#[token("let")]
	Let,

	#[token("in")]
	In,

	#[token("all")]
	All,

	#[token("list")]
	ListKeyword,

	#[token("as")]
	As,

	#[token("where")]
	Where,

	#[token("null")]
	Unit,

	#[token("true")]
	True,

	#[token("false")]
	False,

	#[token(regexp("[0-9]+"))]
	Number,

	#[token(regexp("\"[^\"]*\""))]
	TextString,

	#[token(".")]
	Period,

	#[token(",")]
	Comma,

	#[token(":")]
	Colon,

	#[token(";")]
	Semicolon,

	#[token("=>")]
	Arrow,

	#[token("=")]
	Equal,

	#[token("[")]
	BracketL,

	#[token("]")]
	BracketR,

	#[token("{")]
	BraceL,

	#[token("}")]
	BraceR,

	#[token("(")]
	ParenthesisL,

	#[token(")")]
	ParenthesisR,

	#[token("...")]
	Ellipsis,

	#[token("^^")]
	DoubleCarret,

	#[token("@")]
	At
}

#[derive(Debug, ll1::Parse)]
pub struct Module {
	pub items: Vec<Item>
}

#[derive(Debug, ll1::Parse)]
pub enum Item {
	Prefix(PrefixDefinition),
	Function(Function)
}

#[derive(Debug, ll1::Parse)]
pub struct PrefixDefinition {
	pub prefix: Prefix,
	pub ident: Ident,
	pub eq: Equal,
	pub value: Iri
}

#[derive(Debug, ll1::Parse)]
pub struct Function {
	pub fn_: Fn,
	pub paren_l: ParenthesisL,
	pub ident: Ident,
	pub args: Vec<ArgDef>,
	pub paren_r: ParenthesisR,
	pub eq: Equal,
	pub type_: Option<TypeAnnotation>,
	pub body: Expr
}

#[derive(Debug, ll1::Parse)]
pub enum ArgDef {
	Typed(ParenthesisL, Ident, TypeAnnotation, ParenthesisR),
	Untyped(Ident)
}

#[derive(Debug, ll1::Parse)]
pub struct TypeAnnotation {
	pub colon: Colon,
	pub expr: TypeExpr
}

#[derive(Debug, ll1::Parse)]
pub enum ConstValue {
	Literal(LiteralExpr),
	List(ConstList),
	Map(ConstMap),
}

#[derive(Debug, ll1::Parse)]
pub struct ConstList {
	pub bracket_l: BracketL,
	pub items: List<ConstValue>,
	pub bracket_r: BracketR
}

#[derive(Debug, ll1::Parse)]
pub struct ConstMap {
	pub brace_l: BraceL,
	pub entries: List<ConstEntry>,
	pub brace_r: BraceR
}

#[derive(Debug, ll1::Parse)]
pub struct ConstEntry {
	pub key: ConstValue,
	pub colon: Colon,
	pub value: ConstValue,
}

#[derive(Debug, ll1::Parse)]
pub enum Expr {
	/// ```
	/// let x where {}; foo
	/// ```
	Let(Let, Bindings, Semicolon, InnerExpr),
	Inner(InnerExpr),
}

#[derive(Debug, ll1::Parse)]
pub enum InnerExpr {
	Var(Ident),
	Literal(LiteralExpr),
	List(ListExpr),
	Map(MapExpr),
	Match(MatchExpr),
	Call(CallExpr),
}

#[derive(Debug, ll1::Parse)]
pub enum LiteralExpr {
	Unit(Unit),
	True(True),
	False(False),
	Number(Number),
	TextString(TextString),
}

#[derive(Debug, ll1::Parse)]
pub struct ListExpr {
	pub bracket_l: BracketL,
	pub desc: ListDescription,
	pub bracket_r: BracketR
}

#[derive(Debug, ll1::Parse)]
pub enum ListDescription {
	/// ```tldr
	/// [ all x where { ... } =>  ]
	/// ```
	All(All, Bindings, Option<Mapping>),

	/// ```tldr
	/// [ list x where { ... } in list =>  ]
	/// ```
	List(ListKeyword, Bindings, In, Box<Expr>, Option<Mapping>),
	Given(List<Expr>),
}

#[derive(Debug, ll1::Parse)]
pub struct MapExpr {
	pub brace_l: BraceL,
	pub entries: List<Entry>,
	pub brace_r: BraceR
}

#[derive(Debug, ll1::Parse)]
pub struct MatchExpr {
	pub match_: Match,
	pub cases: List<MatchCase>,
}

#[derive(Debug, ll1::Parse)]
pub struct MatchCase {
	pub name: Ident,
	pub value: Expr,
}

#[derive(Debug, ll1::Parse)]
pub struct CallExpr {
	pub paren_l: ParenthesisL,
	pub function: Ident,
	pub args: Vec<Expr>,
	pub paren_r: ParenthesisR,
}

#[derive(Debug, ll1::Parse)]
pub struct Bindings {
	pub variables: List<Ident>,
	pub bound: Option<WhereBound>,
}

#[derive(Debug, ll1::Parse)]
pub struct WhereBound {
	pub where_: Where,
	pub brace_l: BraceL,
	pub dataset: RdfDataset,
	pub brace_r: BraceR,
}

#[derive(Debug, ll1::Parse)]
pub struct RdfDataset {
	pub statements: List<RdfStatement, Period>
}

#[derive(Debug, ll1::Parse)]
pub struct RdfStatement {
	pub subject: RdfTerm,
	pub predicates_objects: List<RdfPredicateObjects, Semicolon>
}

#[derive(Debug, ll1::Parse)]
pub struct RdfPredicateObjects {
	pub predicate: RdfTerm,
	pub objects: List<RdfTerm, Comma>
}

#[derive(Debug, ll1::Parse)]
pub enum RdfTerm {
	Var(Ident),
	Iri(Iri),
	Literal(RdfLiteral)
}

#[derive(Debug, ll1::Parse)]
pub enum Iri {
	Compact(CompactIri),
	Full(FullIri)
}

#[derive(Debug, ll1::Parse)]
pub struct RdfLiteral {
	pub value: TextString,
	pub annotation: Option<RdfLiteralAnnotation>
}

#[derive(Debug, ll1::Parse)]
pub enum RdfLiteralAnnotation {
	Type(DoubleCarret, Iri),
	Language(At, TextString)
}

#[derive(Debug, ll1::Parse)]
pub struct Mapping {
	pub arrow: Arrow,
	pub expr: Box<Expr>,
}

#[derive(Debug, ll1::Parse)]
pub struct Entry {
	pub key: ConstValue,
	pub colon: Colon,
	pub value: Expr,
}

#[derive(Debug, ll1::Parse)]
pub enum TypeExpr {
	Var(Ident),
	Literal(LiteralExpr),
	List(ListTypeExpr),
	Map(MapTypeExpr),
	Enum(EnumTypeExpr),
}

/// List type expression.
/// 
/// ```
/// [A, B, C]
/// [A, B, C, ... T],
/// [... T]
/// ```
#[derive(Debug, ll1::Parse)]
pub struct ListTypeExpr {
	pub prefix: List<TypeExpr>,
	pub rest: Option<ListTypeExprRest>
}

#[derive(Debug, ll1::Parse)]
pub struct ListTypeExprRest {
	pub ellipsis: Ellipsis,
	pub type_: Box<TypeExpr>
}

#[derive(Debug, ll1::Parse)]
pub struct MapTypeExpr {
	pub brace_l: BraceL,
	pub entries: List<EntryType>,
	pub brace_r: BraceR,
}

#[derive(Debug, ll1::Parse)]
pub struct EntryType {
	pub key: TypeExpr,
	pub colon: Colon,
	pub value: TypeExpr,
}

#[derive(Debug, ll1::Parse)]
pub struct EnumTypeExpr {
	pub enum_: Enum,
	pub brace_l: BraceL,
	pub variants: List<Variant>,
	pub brace_r: BraceR,
}

#[derive(Debug, ll1::Parse)]
pub struct Variant {
	pub ident: Ident,
	pub comma: Comma,
	pub type_: TypeExpr,
}

#[derive(Debug, ll1::Parse)]
pub enum List<T, S = Comma> {
	Empty,
	NonEmpty(Box<T>, ListRest<T, S>),
}

impl<T, S> List<T, S> {
	pub fn is_empty(&self) -> bool {
		matches!(self, Self::Empty)
	}

	pub fn len(&self) -> usize {
		match self {
			Self::Empty => 0,
			Self::NonEmpty(_, rest) => 1 + rest.len()
		}
	}

	pub fn first(&self) -> Option<&T> {
		match self {
			Self::Empty => None,
			Self::NonEmpty(item, _) => Some(item)
		}
	}

	pub fn iter(&self) -> ListIter<T, S> {
		ListIter {
			current: match self {
				Self::Empty => None,
				Self::NonEmpty(item, rest) => Some((item, rest))
			}
		}
	}
}

#[derive(Debug, ll1::Parse)]
pub enum ListRest<T, S = Comma> {
	Empty,
	NonEmpty(S, Box<List<T, S>>),
}

impl<T, S> ListRest<T, S> {
	pub fn is_empty(&self) -> bool {
		match self {
			Self::Empty => true,
			Self::NonEmpty(_, list) => list.is_empty()
		}
	}

	pub fn len(&self) -> usize {
		match self {
			Self::Empty => 0,
			Self::NonEmpty(_, list) => list.len()
		}
	}
}

pub struct ListIter<'a, T, S = Comma> {
	current: Option<(&'a T, &'a ListRest<T, S>)>
}

impl<'a, T, S> Iterator for ListIter<'a, T, S> {
	type Item = &'a T;

	fn next(&mut self) -> Option<Self::Item> {
		let (item, rest) = self.current.take()?;
		
		if let ListRest::NonEmpty(_, list) = rest {
			if let List::NonEmpty(next_item, next_rest) = &**list {
				self.current = Some((next_item, next_rest));
			}
		}

		Some(item)
	}
}

impl<'a, T, S> IntoIterator for &'a List<T, S> {
	type Item = &'a T;
	type IntoIter = ListIter<'a, T, S>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}