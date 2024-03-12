use crate::Pattern;

pub type Graph<R = rdf_types::Term> = rdf_types::dataset::BTreeGraph<Pattern<R>>;

pub type Dataset<R = rdf_types::Term> = rdf_types::dataset::BTreeDataset<Pattern<R>>;
