use rdf_types::{dataset::BTreeDataset, RDF_FIRST, RDF_REST};
use serde::{Deserialize, Serialize};

use crate::abs::{
	self,
	syntax::{
		Build, CompactIri, Context, Dataset, Error, Pattern, Scope, ValueFormatOrLayout, ValueIntro,
	},
};

use super::{
	LayoutHeader, LayoutRef, OrderedListLayoutType, SizedListLayoutType, UnorderedListLayoutType,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ListLayout {
	Ordered(OrderedListLayout),
	Unordered(UnorderedListLayout),
	Sized(SizedListLayout),
}

impl ListLayout {
	pub fn id(&self) -> Option<&CompactIri> {
		match self {
			Self::Ordered(l) => l.header.id.as_ref(),
			Self::Unordered(l) => l.header.id.as_ref(),
			Self::Sized(l) => l.header.id.as_ref(),
		}
	}
}

impl<C: Context> Build<C> for ListLayout
where
	C::Resource: Clone,
{
	type Target = abs::layout::ListLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		match self {
			Self::Ordered(l) => l
				.build(context, scope)
				.map(abs::layout::ListLayout::Ordered),
			Self::Unordered(l) => l
				.build(context, scope)
				.map(abs::layout::ListLayout::Unordered),
			Self::Sized(l) => l.build(context, scope).map(abs::layout::ListLayout::Sized),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OrderedListLayout {
	#[serde(rename = "type")]
	pub type_: OrderedListLayoutType,

	#[serde(flatten)]
	pub header: LayoutHeader,

	pub node: ListNodeOrLayout,

	#[serde(
		default = "Pattern::default_head",
		skip_serializing_if = "Pattern::is_default_head"
	)]
	pub head: Pattern,

	#[serde(
		default = "Pattern::default_tail",
		skip_serializing_if = "Pattern::is_default_tail"
	)]
	pub tail: Pattern,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ListNodeOrLayout {
	ListNode(ListNode),
	Layout(LayoutRef),
}

fn default_list_dataset<C: Context>(
	context: &mut C,
	head: u32,
	rest: u32,
	first: crate::Pattern<C::Resource>,
) -> BTreeDataset<crate::Pattern<C::Resource>>
where
	C::Resource: Clone,
{
	let mut dataset = BTreeDataset::new();

	dataset.insert(rdf_types::Quad(
		crate::Pattern::Var(head),
		crate::Pattern::Resource(context.iri_resource(RDF_FIRST)),
		first,
		None,
	));

	dataset.insert(rdf_types::Quad(
		crate::Pattern::Var(head),
		crate::Pattern::Resource(context.iri_resource(RDF_REST)),
		crate::Pattern::Var(rest),
		None,
	));

	dataset
}

impl<C: Context> Build<C> for ListNodeOrLayout
where
	C::Resource: Clone,
{
	type Target = abs::layout::list::ordered::NodeLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let head = scope.variable_count();
		let rest = head + 1;
		let first = rest + 1;
		match self {
			Self::ListNode(n) => n.build(context, scope),
			Self::Layout(layout_ref) => Ok(abs::layout::list::ordered::NodeLayout {
				intro: 1u32,
				value: crate::ValueFormat {
					layout: layout_ref.build(context, scope)?,
					input: vec![crate::Pattern::Var(first)],
					graph: None,
				},
				dataset: default_list_dataset(context, head, rest, crate::Pattern::Var(first)),
			}),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListNode {
	#[serde(
		default = "ListNode::default_head",
		skip_serializing_if = "ListNode::is_default_head"
	)]
	pub head: String,

	#[serde(
		default = "ListNode::default_rest",
		skip_serializing_if = "ListNode::is_default_rest"
	)]
	pub rest: String,

	#[serde(default, skip_serializing_if = "ValueIntro::is_default")]
	pub intro: ValueIntro,

	pub value: ValueFormatOrLayout,

	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub dataset: Option<Dataset>,
}

impl ListNode {
	pub fn default_head() -> String {
		"head".to_string()
	}

	pub fn is_default_head(value: &str) -> bool {
		value == "head"
	}

	pub fn default_rest() -> String {
		"rest".to_string()
	}

	pub fn is_default_rest(value: &str) -> bool {
		value == "rest"
	}
}

impl<C: Context> Build<C> for OrderedListLayout
where
	C::Resource: Clone,
{
	type Target = abs::layout::OrderedListLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let (header, scope) = self.header.build(context, scope)?;
		Ok(abs::layout::OrderedListLayout {
			input: header.input,
			intro: header.intro,
			node: self.node.build(context, &scope)?,
			head: self.head.build(context, &scope)?,
			tail: self.tail.build(context, &scope)?,
			dataset: header.dataset,
			extra_properties: header.properties,
		})
	}
}

impl<C: Context> Build<C> for ListNode
where
	C::Resource: Clone,
{
	type Target = abs::layout::list::ordered::NodeLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let head = scope.variable_count();
		let rest = head + 1;
		let scope = scope.with_intro(
			[&self.head, &self.rest]
				.into_iter()
				.chain(self.intro.as_slice()),
		)?;
		Ok(abs::layout::list::ordered::NodeLayout {
			intro: self.intro.len() as u32,
			value: self.value.build(context, &scope)?,
			dataset: match &self.dataset {
				Some(dataset) => dataset.build(context, &scope)?,
				None => match &self.value {
					ValueFormatOrLayout::Format(f) => {
						if f.input.len() == 1 {
							let first = f.input.first().unwrap().build(context, &scope)?;
							default_list_dataset(context, head, rest, first)
						} else {
							BTreeDataset::new()
						}
					}
					ValueFormatOrLayout::Layout(_) => {
						let first = crate::Pattern::Var(scope.variable("value")?);
						default_list_dataset(context, head, rest, first)
					}
				},
			},
		})
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnorderedListLayout {
	#[serde(rename = "type")]
	pub type_: UnorderedListLayoutType,

	#[serde(flatten)]
	pub header: LayoutHeader,

	pub item: ListItem,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListItem {
	#[serde(default, skip_serializing_if = "ValueIntro::is_default")]
	pub intro: ValueIntro,

	pub value: ValueFormatOrLayout,

	#[serde(default, skip_serializing_if = "Dataset::is_empty")]
	pub dataset: Dataset,

	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub property: Option<Pattern>,
}

impl<C: Context> Build<C> for UnorderedListLayout
where
	C::Resource: Clone,
{
	type Target = abs::layout::UnorderedListLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let subject = if self.header.input.is_empty() {
			None
		} else {
			Some(0)
		};

		let (header, scope) = self.header.build(context, scope)?;

		Ok(abs::layout::UnorderedListLayout {
			input: header.input,
			intro: header.intro,
			item: self.item.build(context, &scope, subject)?,
			dataset: header.dataset,
			extra_properties: header.properties,
		})
	}
}

impl ListItem {
	fn build<C: Context>(
		&self,
		context: &mut C,
		scope: &Scope,
		subject: Option<u32>,
	) -> Result<abs::layout::list::ItemLayout<C::Resource>, Error>
	where
		C::Resource: Clone,
	{
		let object = if self.intro.is_empty() {
			None
		} else {
			Some(scope.variable_count())
		};

		let scope = scope.with_intro(self.intro.as_slice())?;

		let mut dataset = self.dataset.build(context, &scope)?;
		if let Some(prop) = &self.property {
			match subject {
				Some(subject) => match object {
					Some(object) => {
						let prop = prop.build(context, &scope)?;
						dataset.insert(rdf_types::Quad(
							crate::Pattern::Var(subject),
							prop,
							crate::Pattern::Var(object),
							None,
						));
					}
					None => return Err(Error::NoPropertyObject),
				},
				None => return Err(Error::NoPropertySubject),
			}
		}

		Ok(abs::layout::list::ItemLayout {
			intro: self.intro.len() as u32,
			value: self.value.build(context, &scope)?,
			dataset,
		})
	}
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SizedListLayout {
	#[serde(rename = "type")]
	pub type_: SizedListLayoutType,

	#[serde(flatten)]
	pub header: LayoutHeader,

	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub items: Vec<ListItem>,
}

impl<C: Context> Build<C> for SizedListLayout
where
	C::Resource: Clone,
{
	type Target = abs::layout::SizedListLayout<C::Resource>;

	fn build(&self, context: &mut C, scope: &Scope) -> Result<Self::Target, Error> {
		let subject = if self.header.input.is_empty() {
			None
		} else {
			Some(0)
		};

		let (header, scope) = self.header.build(context, scope)?;

		let mut items = Vec::with_capacity(self.items.len());
		for item in &self.items {
			items.push(item.build(context, &scope, subject)?)
		}

		Ok(abs::layout::SizedListLayout {
			input: header.input,
			intro: header.intro,
			items,
			dataset: header.dataset,
			extra_properties: header.properties,
		})
	}
}
