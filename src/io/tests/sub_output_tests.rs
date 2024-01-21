use crate::io::output::Output;
use crate::io::sub_output::SubOutput;
use crate::io::tests::local_mock_output::LocalMockOutput;
use crate::primitives::rect::Rect;
use crate::primitives::sized_xy::SizedXY;
use crate::primitives::xy::XY;

#[test]
fn sub_output_test_1() {
    let mut parent_output = LocalMockOutput {
        size: XY::new(10, 10),
        visible_rect: Rect::from_zero(XY::new(10, 10)),
    };

    let sub_output = SubOutput::new(&mut parent_output, Rect::new(XY::new(3, 3), XY::new(4, 5)));

    assert_eq!(sub_output.size(), XY::new(4, 5));
    assert_eq!(sub_output.visible_rect(), Rect::from_zero(XY::new(4, 5)));
}

#[test]
fn sub_output_test_2() {
    let mut parent_output = LocalMockOutput {
        size: XY::new(10, 10),
        visible_rect: Rect::from_zero(XY::new(5, 10)),
    };

    let sub_output = SubOutput::new(&mut parent_output, Rect::new(XY::new(2, 2), XY::new(6, 6)));

    assert_eq!(sub_output.size(), XY::new(6, 6));
    assert_eq!(sub_output.visible_rect(), Rect::from_zero(XY::new(3, 6)));
}

#[test]
fn sub_output_test_3() {
    let mut parent_output = LocalMockOutput {
        size: XY::new(10, 10),
        visible_rect: Rect::new(XY::new(5, 0), XY::new(5, 10)), // visible only last 5 columns
    };

    let sub_output = SubOutput::new(&mut parent_output, Rect::new(XY::new(2, 2), XY::new(6, 6)));

    assert_eq!(sub_output.size(), XY::new(6, 6));
    // meaning "don't draw columns 2, 3, 4 of parent, that are mine 0, 1, 2
    assert_eq!(sub_output.visible_rect(), Rect::new(XY::new(3, 0), XY::new(3, 6)));
}
