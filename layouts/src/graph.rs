use crate::Pattern;

pub type Graph<R = rdf_types::Term> = grdf::BTreeGraph<Pattern<R>>;

pub type Dataset<R = rdf_types::Term> = grdf::BTreeDataset<Pattern<R>>;
