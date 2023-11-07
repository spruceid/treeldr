use crate::Pattern;

pub type Graph<R> = grdf::BTreeGraph<Pattern<R>>;

pub type Dataset<R> = grdf::BTreeDataset<Pattern<R>>;
