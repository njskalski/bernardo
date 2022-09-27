use crate::io::buffer_output::BufferOutput;
use crate::primitives::rect::Rect;
use crate::primitives::sized_xy::SizedXY;
use crate::primitives::xy::XY;
use crate::widget::widget::WID;
use crate::widgets::editor_widget::editor_widget::EditorWidget;

pub struct EditorInterpreter<'a> {
    buffer: &'a BufferOutput,
    wid: WID,
    rect: Rect,
}

pub fn EditorInterpreter<'a> {
    /*
    This method is primitive, but I don't think it's worth to invest here much.
    It finds a largest full rectangle of a widget at point 'at'.
     */
    pub fn find_at(buffer : &'a BufferOutput, at : XY) -> Option<EditorInterpreter<'a>> {
        if ! at < buffer.size() {
            return None;
        }

        let mut upper_left : Option<(XY, WID)> = None;
        for y in at.y..buffer.size().y {
            for x in at.x..buffer.size().x {
                let xy = XY::new(x, y);
                if buffer[xy].ext.0 == EditorWidget::TYPENAME {
                    upper_left = Some((xy, ext.1));
                    break;
                }
            }
        }

        let (mut upper_left, widget_id) = match upper_left {
            None => {
                return None;
            }
            Some(x) => x,
        };

        while upper_left.y > 0 {
            let candidate_xy = XY::new(upper_left.x, upper_left.y - 1);
            if buffer[candidate_xy].ext.1 == widget_id {
                upper_left.y -= 1;
            } else {
                break;
            }
        }
        while upper_left.x > 0 {
            let candidate_xy = XY::new(upper_left.x - 1, upper_left.y);
            if buffer[candidate_xy].ext.1 == widget_id {
                upper_left.x -= 1;
            } else {
                break;
            }
        }

        while

    }
}