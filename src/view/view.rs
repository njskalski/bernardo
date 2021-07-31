trait View {
    fn render(&self, focus: bool, frame : XY) -> WidgetTree;
}