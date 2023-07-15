use test_log::test;

use crate::experiments::subwidget_pointer::{Getter, GetterMut, SubwidgetPointer};
use crate::layout::layout::Layout;
use crate::layout::leaf_layout::LeafLayout;
use crate::layout::split_layout::{SplitDirection, SplitLayout, SplitRule};
use crate::layout::tests::mock_complex_widget::MockComplexWidget;
use crate::layout::tests::mock_widget::MockWidget;
use crate::primitives::rect::Rect;
use crate::primitives::xy::XY;
use crate::widget::fill_policy::SizePolicy;
use crate::widget::widget::Widget;

#[test]
fn split_layout_test_widget_determined() {
    let mut mock_parent_widget = MockComplexWidget::new(
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

    {
        let layout_res = mock_parent_widget.get_layout_res(XY::new(9, 9), Rect::from_zero(XY::new(9, 9)));

        assert_eq!(layout_res.wwrs.len(), 3);
        assert_eq!(layout_res.wwrs[0].rect(), Rect::new(XY::new(0, 0), XY::new(3, 3)));
        assert_eq!(layout_res.wwrs[1].rect(), Rect::new(XY::new(3, 0), XY::new(3, 3)));
        assert_eq!(layout_res.wwrs[2].rect(), Rect::new(XY::new(6, 0), XY::new(3, 3)));
    }

    {
        let layout_res = mock_parent_widget.get_layout_res(XY::new(10, 10), Rect::from_zero(XY::new(10, 10)));

        assert_eq!(layout_res.wwrs.len(), 3);
        assert_eq!(layout_res.wwrs[0].rect(), Rect::new(XY::new(0, 0), XY::new(3, 3)));
        // The first cell gets additional 1
        assert_eq!(layout_res.wwrs[1].rect(), Rect::new(XY::new(4, 0), XY::new(3, 3)));
        assert_eq!(layout_res.wwrs[2].rect(), Rect::new(XY::new(7, 0), XY::new(3, 3)));
    }
}

#[test]
fn split_layout_test_layout_determined() {
    let mut mock_parent_widget = MockComplexWidget::new(
        XY::new(10, 10),
        vec![
            MockWidget::new(XY::new(1, 1)).with_size_policy(SizePolicy::MATCH_LAYOUT),
            MockWidget::new(XY::new(1, 1)).with_size_policy(SizePolicy::MATCH_LAYOUT),
            MockWidget::new(XY::new(1, 1)).with_size_policy(SizePolicy::MATCH_LAYOUT),
        ],
        Box::new(|parent_widget: &MockComplexWidget| -> Box<dyn Layout<MockComplexWidget>> {
            SplitLayout::new(SplitDirection::Horizontal)
                .with(SplitRule::Proportional(1.0), LeafLayout::new(parent_widget.get_subwidget_ptr(0)).boxed())
                .with(SplitRule::Proportional(1.0), LeafLayout::new(parent_widget.get_subwidget_ptr(0)).boxed())
                .with(SplitRule::Proportional(1.0), LeafLayout::new(parent_widget.get_subwidget_ptr(0)).boxed())
                .boxed()
        }));

    {
        let layout_res = mock_parent_widget.get_layout_res(XY::new(9, 9), Rect::from_zero(XY::new(9, 9)));

        assert_eq!(layout_res.wwrs.len(), 3);
        assert_eq!(layout_res.wwrs[0].rect(), Rect::new(XY::new(0, 0), XY::new(3, 9)));
        assert_eq!(layout_res.wwrs[1].rect(), Rect::new(XY::new(3, 0), XY::new(3, 9)));
        assert_eq!(layout_res.wwrs[2].rect(), Rect::new(XY::new(6, 0), XY::new(3, 9)));
    }

    {
        let layout_res = mock_parent_widget.get_layout_res(XY::new(10, 10), Rect::from_zero(XY::new(10, 10)));

        assert_eq!(layout_res.wwrs.len(), 3);
        assert_eq!(layout_res.wwrs[0].rect(), Rect::new(XY::new(0, 0), XY::new(4, 10)));
        // The first cell gets additional 1
        assert_eq!(layout_res.wwrs[1].rect(), Rect::new(XY::new(4, 0), XY::new(3, 10)));
        assert_eq!(layout_res.wwrs[2].rect(), Rect::new(XY::new(7, 0), XY::new(3, 10)));
    }
}

// #[test]
// fn split_layout_test_layout_determined() {
//     let mut mock_parent_widget = MockComplexWidget::new(
//         XY::new(10, 10),
//         vec![MockWidget::new(XY::new(1, 1)),
//              MockWidget::new(XY::new(1, 1)),
//              MockWidget::new(XY::new(1, 1)),
//         ],
//         Box::new(|parent_widget: &MockComplexWidget| -> Box<dyn Layout<MockComplexWidget>> {
//             SplitLayout::new(SplitDirection::Horizontal)
//                 .with(SplitRule::Proportional(1.0), LeafLayout::new(parent_widget.get_subwidget_ptr(0)).with_size_policy(SizePolicy::MATCH_LAYOUT).boxed())
//                 .with(SplitRule::Proportional(1.0), LeafLayout::new(parent_widget.get_subwidget_ptr(0)).with_size_policy(SizePolicy::MATCH_LAYOUT).boxed())
//                 .with(SplitRule::Proportional(1.0), LeafLayout::new(parent_widget.get_subwidget_ptr(0)).with_size_policy(SizePolicy::MATCH_LAYOUT).boxed())
//                 .boxed()
//         }));
// }