use crate::Pattern;

pub type Graph<R> = grdf::BTreeGraph<Pattern<R>>;
