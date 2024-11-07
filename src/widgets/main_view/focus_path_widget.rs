use log::{error, warn};

use crate::config::theme::Theme;
use crate::experiments::screenspace::Screenspace;
use crate::io::input_event::InputEvent;
use crate::io::output::Output;
use crate::io::style::TextStyle;
use crate::primitives::printable::Printable;
use crate::primitives::xy::XY;
use crate::unpack_or_e;
use crate::widget::any_msg::AnyMsg;
use crate::widget::fill_policy::SizePolicy;
use crate::widget::widget::{get_new_widget_id, Widget, WID};

// This is a program-wide status bar
pub struct FocusPathWidget {
    id: WID,
    focus_path_items: Vec<String>,

    layout_res: Option<XY>,
}

impl FocusPathWidget {
    pub const TYPENAME: &'static str = "focus_path_widget";
    pub const SEPARATOR: &'static str = " > ";

    pub fn new() -> Self {
        Self {
            id: get_new_widget_id(),
            focus_path_items: vec![],
            layout_res: None,
        }
    }

    fn count_width(&self) -> u16 {
        let res_usize = self.focus_path_items.iter().map(|item| item.graphemes().count()).sum::<usize>()
            + if self.focus_path_items.is_empty() {
                1 // non empty
            } else {
                (self.focus_path_items.len() - 1) * Self::SEPARATOR.len()
            };

        if res_usize > u16::MAX as usize {
            error!("width of focus-bar exceeds u16::MAX, this is almost for sure an error.");
            u16::MAX
        } else {
            res_usize as u16
        }
    }

    pub fn set_focus_path(&mut self, focus_path: Vec<String>) {
        self.focus_path_items = focus_path;
    }
}

impl Widget for FocusPathWidget {
    fn id(&self) -> WID {
        self.id
    }

    fn static_typename() -> &'static str
    where
        Self: Sized,
    {
        Self::TYPENAME
    }

    fn typename(&self) -> &'static str {
        Self::TYPENAME
    }

    fn full_size(&self) -> XY {
        XY::new(self.count_width(), 1)
    }

    fn layout(&mut self, screenspace: Screenspace) {
        self.layout_res = Some(XY::new(1, screenspace.output_size().x));
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        None
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        None
    }

    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        let size = unpack_or_e!(self.layout_res, (), "render before layout");

        if self.focus_path_items.is_empty() {
            warn!("focus path empty");
            return;
        }

        let mut value: String = String::new();

        for (idx, item) in self.focus_path_items.iter().enumerate() {
            if idx > 0 {
                value += " > "
            }
            value += item;
        }

        let desired_width = value.graphemes().count();
        let available_width = size.y;

        let skip = if (available_width as usize) < desired_width {
            desired_width - (available_width as usize)
        } else {
            0
        };

        for (idx, grapheme) in value.graphemes().enumerate().skip(skip) {
            if idx > u16::MAX as usize {
                error!("overflow");
                break;
            }
            output.print_at(XY::new(idx as u16, 0), theme.default_text(false), grapheme);
        }
    }

    fn size_policy(&self) -> SizePolicy {
        SizePolicy::MATCH_LAYOUTS_WIDTH
    }

    fn is_focusable(&self) -> bool {
        false
    }
}
