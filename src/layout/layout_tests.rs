#[cfg(test)]
pub mod tests {
    use std::cmp::min;

    use crate::config::theme::Theme;
    use crate::experiments::subwidget_pointer::SubwidgetPointer;
    use crate::io::input_event::InputEvent;
    use crate::io::output::Output;
    use crate::layout::layout::{Layout, LayoutResult};
    use crate::layout::split_layout::{SplitDirection, SplitLayout, SplitRule};
    use crate::layout::widget_with_rect::WidgetWithRect;
    use crate::primitives::rect::Rect;
    use crate::primitives::size_constraint::SizeConstraint;
    use crate::primitives::xy::XY;
    use crate::widget::any_msg::AnyMsg;
    use crate::widget::widget::{get_new_widget_id, WID, Widget};

    struct MockLayout {
        wid: WID,
        min_size: XY,
        preferred_size: XY,
    }

    impl MockLayout {
        pub fn new(min_size: XY) -> MockLayout {
            MockLayout {
                wid: get_new_widget_id(),
                min_size,
                preferred_size: min_size,
            }
        }

        pub fn with_preferred_size(self, preferred_size: XY) -> MockLayout {
            MockLayout {
                preferred_size,
                ..self
            }
        }
    }

    struct MockWidget {}

    impl Default for MockWidget {
        fn default() -> Self {
            MockWidget {}
        }
    }

    impl Widget for MockWidget {
        fn id(&self) -> WID { todo!() }
        fn typename(&self) -> &'static str { todo!() }
        fn min_size(&self) -> XY { todo!() }
        fn update_and_layout(&mut self, sc: SizeConstraint) -> XY { todo!() }
        fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> { todo!() }
        fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> { todo!() }
        fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) { todo!() }
    }

    impl Layout<MockWidget> for MockLayout {
        fn min_size(&self, _root: &MockWidget) -> XY {
            self.min_size
        }

        fn layout(&self, root: &mut MockWidget, sc: SizeConstraint) -> LayoutResult<MockWidget> {
            assert!(sc.bigger_equal_than(self.min_size));

            let mut result = self.preferred_size;
            sc.x().map(|max_x| result.x = min(result.x, max_x));
            sc.y().map(|max_y| result.y = min(result.y, max_y));

            LayoutResult::new(vec![
                WidgetWithRect::new(
                    SubwidgetPointer::new(Box::new(|_| { todo!() }), Box::new(|_| { todo!() })),
                    Rect::new(XY::ZERO, result),
                    true,
                )
            ], result)
        }
    }

    fn get_results(items: &Vec<(SplitRule, XY, Option<XY>)>, sc: SizeConstraint) -> (XY, Vec<u16>) {
        let mut layout = SplitLayout::new(SplitDirection::Horizontal);
        for item in items.into_iter() {
            let mut mock_layout = MockLayout::new(item.1);
            match item.2 {
                None => {}
                Some(preferred_size) => {
                    mock_layout = mock_layout.with_preferred_size(preferred_size);
                }
            }

            layout = layout.with(item.0, Box::new(mock_layout));
        }

        let mut mock_widget = MockWidget::default();
        let layout_result = layout.layout(&mut mock_widget, sc);

        let mut result: Vec<u16> = Vec::new();
        for wwr in layout_result.wwrs {
            result.push(wwr.rect().size.x);

            let x_offset = result.iter().fold(0 as u16, |acc, item| acc + item);
            assert_eq!(wwr.rect().pos.x, x_offset, "wwr.pos.x = {}, x_offset = {}", wwr.rect().pos, x_offset);
        }

        (layout_result.total_size, result)
    }

    #[test]
    fn test_split_1() {
        let items: Vec<(SplitRule, XY, Option<XY>)> = vec![
            (SplitRule::Fixed(2), XY::new(1, 1), None),
            (SplitRule::Proportional(1.0), XY::new(1, 1), None),
            (SplitRule::Proportional(1.0), XY::new(1, 1), None),
        ];

        assert_eq!(get_results(&items, SizeConstraint::simple(XY::new(10, 10))),
                   (XY::new(10, 10), vec![2, 4, 4])
        );
    }
}