use iref::{Iri, IriBuf};

pub struct Url<'a>(Iri<'a>);

#[derive(Clone, Debug)]
pub struct UrlBuf(IriBuf);