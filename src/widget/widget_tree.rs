/*
this is output of views, renderable.
 */

// could be called WidgetTreeNodes
struct WidgetTree {
    node : Box<Widget>,
    children : Vec<Box<WidgetTree>>
}

impl WidgetTree {
    fn new(widget : Widget) -> Self {
        WidgetTree {
            node : Box::new(widget),
            children : vec![]
        }
    }

    fn get_node(&self) -> &Box<WidgetTree> {
        &self.node
    }

    fn get_children(&self) -> &Vec<Box<WidgetTree>> {
        &self.children
    }

    fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }
}