use crate::primitives::styled_string::StyledString;
use crate::widget::widget::{BaseWidget, get_new_widget_id};
use crate::primitives::xy::XY;
use crate::io::input_event::InputEvent;
use crate::widget::any_msg::AnyMsg;
use crate::io::output::Output;
use log::{debug, warn};
use unicode_width::UnicodeWidthStr;

struct LabelWidget {
    id : usize,
    styled_text : StyledString,
}

impl LabelWidget {
    fn new(styled_text : StyledString) -> Self {
        let id = get_new_widget_id();

        LabelWidget {
            id,
            styled_text,
        }
    }
}

impl BaseWidget for LabelWidget {
    fn id(&self) -> usize {
        self.id
    }

    fn min_size(&self) -> XY {
        self.styled_text.size()
    }

    fn size(&self, max_size: XY) -> XY {
        self.min_size()
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        debug!("sending input to label {:?}", input_event);
        None
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        debug!("attempting to update the label with msg {:?}", msg);
        None
    }

    fn get_focused(&self) -> &dyn BaseWidget {
        self
    }

    fn get_focused_mut(&mut self) -> &mut dyn BaseWidget {
        self
    }

    fn render(&self, focused: bool, output: &mut Output) {
        let mut pos_it = XY::new(0,0);

        // for ssi in self.styled_text.substrings() {
        //     for piece in ssi.text.split("\n") {
        //         output.print_at(pos_it, ssi.style, piece);
        //
        //         // // breaking line
        //         // if curr_line_width > width {
        //         //     width = curr_line_width;
        //         // }
        //         // height += 1;
        //         // curr_line_width = 0;
        //     }
        // }
    }
}