use iref::{Iri, IriBuf};

pub struct Uri<'a>(Iri<'a>);

#[derive(Clone, Debug)]
pub struct UriBuf(IriBuf);