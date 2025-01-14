/*
This serves as a fixed list of attention nodes Gladius knows how to handle. Each of them should act
as a "serializable" pill for main view, with enough data to switch and/or recreate "what I was
looking at before I navigated away from here". At this point, at least two kind of nodes must exits:
- editor view
- code results view (references)
 */

#[derive(Debug)]
pub enum AttentionNode {
    Editor {},
    CodeResultsView {},
}
