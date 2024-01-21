#[cfg(test)]
pub mod tests {
    #![allow(dead_code)]

    use crate::config::theme::Theme;
    use crate::experiments::screenspace::Screenspace;
    use crate::experiments::subwidget_pointer::SubwidgetPointer;
    use crate::io::input_event::InputEvent;
    use crate::io::output::Output;
    use crate::layout::layout::Layout;
    use crate::layout::leaf_layout::LeafLayout;
    use crate::layout::split_layout::{SplitDirection, SplitLayout, SplitRule};
    use crate::primitives::rect::Rect;
    use crate::primitives::xy::XY;
    use crate::widget::any_msg::AnyMsg;
    use crate::widget::complex_widget::{ComplexWidget, DisplayState};
    use crate::widget::fill_policy::SizePolicy;
    use crate::widget::widget::{get_new_widget_id, Widget, WID};

    struct MockWidget {
        pub wid: WID,
        pub full_size: XY,
        pub size_policy: SizePolicy,
    }

    impl MockWidget {
        fn new() -> Self {
            MockWidget {
                wid: get_new_widget_id(),
                full_size: XY::new(10, 20),
                size_policy: SizePolicy::MATCH_LAYOUT,
            }
        }

        fn with_size_policy(self, size_policy: SizePolicy) -> Self {
            MockWidget { size_policy, ..self }
        }
    }

    impl Widget for MockWidget {
        fn id(&self) -> WID {
            self.wid
        }
        fn typename(&self) -> &'static str {
            "mock_widget"
        }
        fn size_policy(&self) -> SizePolicy {
            self.size_policy
        }
        fn full_size(&self) -> XY {
            self.full_size
        }
        fn layout(&mut self, screenspace: Screenspace) {}
        fn on_input(&self, _input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
            todo!()
        }
        fn update(&mut self, _msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
            todo!()
        }
        fn render(&self, _theme: &Theme, _focused: bool, _output: &mut dyn Output) {
            todo!()
        }

        fn static_typename() -> &'static str
        where
            Self: Sized,
        {
            "mock_widget"
        }
    }

    struct MockComplexWidget {
        pub wid: WID,
        pub size: XY,
        pub items: Vec<(SplitRule, MockWidget)>,
    }

    impl MockComplexWidget {
        fn new(items: Vec<(SplitRule, MockWidget)>) -> Self {
            MockComplexWidget {
                wid: get_new_widget_id(),
                size: XY::new(10, 10),
                items,
            }
        }

        fn get_widget_pointer(idx: usize) -> SubwidgetPointer<MockComplexWidget> {
            let idx2 = idx;
            SubwidgetPointer::new(
                Box::new(move |root: &MockComplexWidget| &root.items.get(idx).unwrap().1 as &dyn Widget),
                Box::new(move |root: &mut MockComplexWidget| &mut root.items.get_mut(idx2).unwrap().1 as &mut dyn Widget),
            )
        }

        fn get_results(&mut self, screenspace: Screenspace) -> (XY, Vec<u16>) {
            let layout = self.get_layout();
            let layout_result = layout.layout(self, screenspace);

            let mut result: Vec<u16> = Vec::new();
            for wwr in layout_result.wwrs {
                let y_offset = result.iter().fold(0 as u16, |acc, item| acc + item);
                assert_eq!(
                    wwr.rect().pos.y,
                    y_offset,
                    "wwr.pos.y = {}, y_offset = {}",
                    wwr.rect().pos,
                    y_offset
                );
                result.push(wwr.rect().size.y);
            }

            (layout_result.total_size, result)
        }
    }

    impl Widget for MockComplexWidget {
        fn id(&self) -> WID {
            self.wid
        }

        fn static_typename() -> &'static str
        where
            Self: Sized,
        {
            "mock_complex_widget"
        }

        fn typename(&self) -> &'static str {
            "mock_complex_widget"
        }

        fn full_size(&self) -> XY {
            self.size
        }

        fn layout(&mut self, screenspace: Screenspace) {
            self.complex_layout(screenspace)
        }
        fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
            todo!()
        }
        fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
            todo!()
        }
        fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
            todo!()
        }
    }

    impl ComplexWidget for MockComplexWidget {
        fn get_layout(&self) -> Box<dyn Layout<Self>> {
            let mut layout = SplitLayout::new(SplitDirection::Vertical);
            for (idx, (split_rule, _)) in self.items.iter().enumerate() {
                layout = layout.with(*split_rule, Box::new(LeafLayout::new(Self::get_widget_pointer(idx))));
            }

            Box::new(layout)
        }

        fn get_default_focused(&self) -> SubwidgetPointer<Self> {
            todo!()
        }
        fn set_display_state(&mut self, display_state: DisplayState<Self>) {
            todo!()
        }
        fn get_display_state_op(&self) -> Option<&DisplayState<Self>> {
            todo!()
        }
        fn get_display_state_mut_op(&mut self) -> Option<&mut DisplayState<Self>> {
            todo!()
        }
    }

    #[test]
    fn test_split_1() {
        let items: Vec<(SplitRule, MockWidget)> = vec![
            (SplitRule::Fixed(2), MockWidget::new()),
            (SplitRule::Proportional(1.0), MockWidget::new()),
            (SplitRule::Proportional(1.0), MockWidget::new()),
        ];

        let mut mcw = MockComplexWidget::new(items);

        assert_eq!(
            mcw.get_results(Screenspace::new(XY::new(10, 10), Rect::from_zero(XY::new(10, 10)))),
            (XY::new(10, 10), vec![2, 4, 4])
        );

        assert_eq!(
            mcw.get_results(Screenspace::new(XY::new(6, 6), Rect::from_zero(XY::new(6, 6)))),
            (XY::new(6, 6), vec![2, 2, 2])
        );
    }

    #[test]
    fn test_split_2() {
        let items: Vec<(SplitRule, MockWidget)> = vec![
            (SplitRule::Fixed(2), MockWidget::new()),
            (SplitRule::Proportional(1.0), MockWidget::new()),
            (SplitRule::Proportional(2.0), MockWidget::new()),
        ];

        let mut mcw = MockComplexWidget::new(items);

        assert_eq!(
            mcw.get_results(Screenspace::new(XY::new(11, 11), Rect::from_zero(XY::new(11, 11)))),
            (XY::new(11, 11), vec![2, 3, 6])
        );
    }

    #[test]
    fn test_split_3() {
        let mut items: Vec<(SplitRule, MockWidget)> =
            vec![(SplitRule::Fixed(5), MockWidget::new()), (SplitRule::Fixed(5), MockWidget::new())];

        let mut mcw = MockComplexWidget::new(items);

        assert_eq!(
            mcw.get_results(Screenspace::new(XY::new(10, 13), Rect::new(XY::ZERO, XY::new(10, 13)))),
            (XY::new(10, 13), vec![5, 5])
        );
    }

    #[test]
    fn test_split_4() {
        let mut items: Vec<(SplitRule, MockWidget)> =
            vec![(SplitRule::Fixed(2), /*XY::new(1, 1), Some(XY::new(10, 2))*/ MockWidget::new())];

        for idx in 0..10 {
            items.push((SplitRule::Proportional(2.0 as f32), MockWidget::new()));
        }

        let mut mcw = MockComplexWidget::new(items);

        assert_eq!(
            mcw.get_results(Screenspace::new(XY::new(10, 12), Rect::new(XY::ZERO, XY::new(10, 12)))),
            (XY::new(10, 12), vec![2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1])
        );
    }
}
