use test_log::test;

use crate::experiments::subwidget_pointer::{Getter, GetterMut, SubwidgetPointer};
use crate::layout::layout::Layout;
use crate::layout::leaf_layout::LeafLayout;
use crate::layout::split_layout::{SplitDirection, SplitLayout, SplitRule};
use crate::layout::tests::mock_complex_widget::MockComplexWidget;
use crate::layout::tests::mock_widget::MockWidget;
use crate::primitives::rect::Rect;
use crate::primitives::xy::XY;
use crate::widget::widget::Widget;

#[test]
fn split_layout_test_1() {
    let mock_parent_widget = MockComplexWidget::new(
        XY::new(10, 10),
        vec![MockWidget::new(XY::new(3, 3)),
             MockWidget::new(XY::new(3, 3)),
             MockWidget::new(XY::new(3, 3)),
        ],
        Box::new(|parent_widget: &MockComplexWidget| -> Box<dyn Layout<MockComplexWidget>> {
            SplitLayout::new(SplitDirection::Horizontal)
                .with(SplitRule::Proportional(1.0), LeafLayout::new(parent_widget.get_subwidget_ptr(0)).boxed())
                .with(SplitRule::Proportional(1.0), LeafLayout::new(parent_widget.get_subwidget_ptr(0)).boxed())
                .with(SplitRule::Proportional(1.0), LeafLayout::new(parent_widget.get_subwidget_ptr(0)).boxed())
                .boxed()
        }));


    // let mut mock_parent_widget = MockWidget::new(XY::new(1, 1));
    // let mut mock_widget = MockWidget::new(XY::new(3, 3));
    //
    // let mut mock_widget_pointer: SubwidgetPointer<MockWidget> = SubwidgetPointer::new(
    //     Box::new(|_| {
    //         &mock_widget as &dyn Widget
    //     }) as Box<dyn Getter<MockWidget>>,
    //     Box::new(|_| {
    //         &mut mock_widget as &mut dyn Widget
    //     }) as Box<dyn GetterMut<MockWidget>>,
    // );
    //
    // let layout = SplitLayout::new(SplitDirection::Horizontal)
    //     .with(SplitRule::Proportional(1.0), LeafLayout::new(mock_widget_pointer.clone()).boxed())
    //     .with(SplitRule::Proportional(1.0), LeafLayout::new(mock_widget_pointer.clone()).boxed())
    //     .with(SplitRule::Proportional(1.0), LeafLayout::new(mock_widget_pointer.clone()).boxed())
    //     .boxed();
    //
    // let layout_res = layout.layout(&mut mock_parent_widget, XY::new(10, 10), Rect::from_zero(XY::new(10, 10)));
}