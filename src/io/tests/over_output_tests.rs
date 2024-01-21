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

    /*
    entire visible rect is there, it's just in new coords
    ┌───child──┐
    │          │
    ┢━━━parent━┪
    ┃          ┃
    ┃          ┃
    ┡━━━━━━━━━━┩
    │          │
    │          │
    └──────────┘
     */
    assert_eq!(over_output.size(), XY::new(10, 20));
    assert_eq!(over_output.visible_rect(), Rect::new(XY::new(0, 2), XY::new(10, 10)));
}

#[test]
fn over_output_test_2() {
    let mut parent_output = LocalMockOutput {
        size: XY::new(10, 10),
        visible_rect: Rect::from_zero(XY::new(10, 10)),
    };

    let over_output = OverOutput::new(&mut parent_output, XY::new(10, 20), XY::new(2, 2));

    /*
    not entire visible rect is there. It has new coords and new size.
    ┌───child──┐
    │          │
    │  ┏━━━parent━┓
    │  ┃       │  ┃
    │  ┃       │  ┃
    │  ┗━━━━━━━━━━┛
    │          │
    │          │
    └──────────┘
     */
    assert_eq!(over_output.size(), XY::new(10, 20));
    assert_eq!(over_output.visible_rect(), Rect::new(XY::new(2, 2), XY::new(8, 10)));
}

#[test]
fn over_output_test_3() {
    let mut parent_output = LocalMockOutput {
        size: XY::new(10, 10),
        visible_rect: Rect::from_zero(XY::new(10, 10)),
    };

    let over_output = OverOutput::new(&mut parent_output, XY::new(10, 10), XY::new(2, 3));

    /*
    visible rect is cut on both axis.
    ┌───child──┐
    │          │
    │  ┏━━━parent━┓
    │  ┃       │  ┃
    └──╂───────┘  ┃
       ┗━━━━━━━━━━┛

     */
    assert_eq!(over_output.size(), XY::new(10, 10));
    assert_eq!(over_output.visible_rect(), Rect::new(XY::new(2, 3), XY::new(8, 7)));
}

// As much as this exhaust the positioning of parent to child (scroll offset is always positive), I
// need to cover also cases of visible rect not covering entire first output.

#[test]
fn over_output_test_4() {
    let mut parent_output = LocalMockOutput {
        size: XY::new(10, 10),
        visible_rect: Rect::new(XY::new(5, 0), XY::new(5, 10)),
    };

    let over_output = OverOutput::new(&mut parent_output, XY::new(10, 20), XY::new(0, 2));

    /*
    x means invisible
    ┌───child──┐
    │          │
    ┢━━━parent━┪
    ┃xxxxx     ┃
    ┃xxxxx     ┃
    ┡━━━━━━━━━━┩
    │          │
    │          │
    └──────────┘
     */
    assert_eq!(over_output.size(), XY::new(10, 20));
    assert_eq!(over_output.visible_rect(), Rect::new(XY::new(5, 2), XY::new(5, 10)));
}

#[test]
fn over_output_test_5() {
    let mut parent_output = LocalMockOutput {
        size: XY::new(10, 10),
        visible_rect: Rect::new(XY::new(5, 0), XY::new(5, 10)),
    };

    let over_output = OverOutput::new(&mut parent_output, XY::new(10, 20), XY::new(2, 2));

    /*
    x means invisible
    ┌───child──┐
    │          │
    │  ┏━━━parent━┓
    │  ┃xxxxx  │  ┃
    │  ┃xxxxx  │  ┃
    │  ┗━━━━━━━━━━┛
    │          │
    │          │
    └──────────┘
     */
    assert_eq!(over_output.size(), XY::new(10, 20));
    assert_eq!(over_output.visible_rect(), Rect::new(XY::new(7, 2), XY::new(3, 10)));
}

#[test]
fn over_output_test_6() {
    let mut parent_output = LocalMockOutput {
        size: XY::new(10, 10),
        visible_rect: Rect::new(XY::new(5, 0), XY::new(5, 10)),
    };

    let over_output = OverOutput::new(&mut parent_output, XY::new(10, 10), XY::new(2, 3));

    /*

    ┌───child──┐
    │          │
    │  ┏━━━parent━┓
    │  ┃xxxxx  │  ┃
    └──╂xxxxx──┘  ┃
       ┗━━━━━━━━━━┛

     */
    assert_eq!(over_output.size(), XY::new(10, 10));
    assert_eq!(over_output.visible_rect(), Rect::new(XY::new(7, 3), XY::new(3, 7)));
}
