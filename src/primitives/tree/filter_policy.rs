#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum FilterPolicy {
    MatchNode,
    MatchNodeOrAncestors,
}
