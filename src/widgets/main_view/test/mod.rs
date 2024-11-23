use crate::primitives::xy::XY;
use crate::widgets::main_view::util::get_rect_for_context_menu;

#[test]
fn util_get_rect_for_context_menu_test_1() {
    let max_x = 100;
    let max_y = 80;

    for x in 0..max_x {
        for y in 0..max_y {
            let point = XY::new(x, y);

            let rect_op = get_rect_for_context_menu(XY::new(max_x, max_y), point);
            assert_eq!(rect_op.is_some(), true, "failed for x = {}, y = {}", x, y);

            let rect = rect_op.unwrap();

            // assert that rect does not cover the point
            assert_eq!(rect.contains(point), false);

            // assert that rect has at least ~40% of each of the screen axes
            // why 40? Because we leave 10% of margins and then we can get the point in the middle.
            assert!(rect.size.x >= (max_x as f64 * 0.38) as u16);
            assert!(rect.size.y >= (max_y as f64 * 0.38) as u16);
        }
    }
}
