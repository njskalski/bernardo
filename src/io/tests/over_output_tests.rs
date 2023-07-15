use crate::io::output::Output;
use crate::io::over_output::OverOutput;
use crate::io::tests::local_mock_output::LocalMockOutput;
use crate::primitives::rect::Rect;
use crate::primitives::sized_xy::SizedXY;
use crate::primitives::xy::XY;

#[test]
fn over_output_test_1() {
    let mut parent_output = LocalMockOutput {
        size: XY::new(10, 10),
        visible_rect: Rect::from_zero(XY::new(10, 10)),
    };

    let over_output = OverOutput::new(&mut parent_output, XY::new(10, 20), XY::new(0, 2));

    assert_eq!(over_output.size(), XY::new(10, 20));
    assert_eq!(over_output.visible_rect(), Rect::from_zero(XY::new(10, 8)));
}