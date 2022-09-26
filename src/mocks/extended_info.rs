/*
So I know this is a violation of "test only output" principle, but I am not interested in "figuring
out where which widget is drawn from images". It's too error prone. Plus this entire module is
behind "cfg(test)", so the information cannot be misused.
 */

use lazy_static::lazy_static;

use crate::io::output::Output;
use crate::primitives::xy::XY;
use crate::widget::widget::{WID, Widget};

lazy_static! {
    static ref ExtendedInfoChannel: (Sender<ExtendedInfo>, Receiver<ExtendedInfo>) = crossbeam_channel::unbounded();
}

pub enum ExtendedInfo {
    WidgetRender { typename: &'static str, wid: WID, upper_left: XY, lower_right: XY, focused: bool },
    ScreenClear,
}

pub fn send_ext_info_render(widget: &dyn Widget, output: &dyn Output, focused: bool) {
    let typename = widget.typename();
    let wid = widget.id();
    let upper_left = output.get_final_position(XY::ZERO).unwrap();
    let lower_right = output.get_final_position(output.size_constraint().visible_hint().size - XY::new(1, 1)).unwrap() + XY::new(1, 1);
    ExtendedInfoChannel.1.send(ExtendedInfo::WidgetRender {
        typename,
        wid,
        upper_left,
        lower_right,
        focused,
    })
}
