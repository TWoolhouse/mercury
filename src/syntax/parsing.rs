#[derive(pest_derive::Parser)]
#[grammar = "syntax/mercury.pest"]
pub(super) struct Mercury;

use pest::iterators::{Pair, Pairs};
pub(super) type Node<'i> = Pair<'i, Rule>;
pub(super) type List<'i> = Pairs<'i, Rule>;

pub(super) trait NodeParent<'i> {
    fn into_child(self) -> Node<'i>;
}

impl<'i> NodeParent<'i> for Node<'i> {
    fn into_child(self) -> Node<'i> {
        self.into_inner()
            .next()
            .expect("Parent should have a child")
    }
}
