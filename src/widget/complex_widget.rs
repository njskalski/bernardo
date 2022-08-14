use log::{debug, error, warn};

use crate::{Output, Theme, Widget};
use crate::io::sub_output::SubOutput;
use crate::layout::dummy_layout::DummyLayout;
use crate::layout::layout::Layout;
use crate::primitives::xy::XY;
use crate::widget::widget::WID;

pub trait ComplexWidget: Widget {
    fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
        let focused_id_op = self.get_focused().map(|f| f.id());
        // if self.display_state.wirs.is_empty() {
        //     error!("call to render before layout");
        //     return;
        // }
        //
        // for wir in &self.display_state.wirs {
        //     match self.get_subwidget(wir.wid) {
        //         Some(widget) => {
        //             let sub_output = &mut SubOutput::new(output, wir.rect);
        //             widget.render(theme,
        //                           Some(widget.id()) == focused_id_op,
        //                           sub_output,
        //             );
        //         }
        //         None => {
        //             warn!("subwidget {} not found!", wir.wid);
        //         }
        //     }
        // }
    }

    fn get_layout(&mut self) -> Box<dyn Layout + '_> {
        error!("call to default internal_layout on {} {}", self.typename(), self.id());
        Box::new(DummyLayout::new(self.id(), self.min_size()))
    }

    fn subwidgets_mut(&mut self) -> Box<dyn std::iter::Iterator<Item=&mut dyn Widget> + '_> where Self: Sized {
        error!("call to default subwidget_mut on {} {}", self.typename(), self.id());
        Box::new(std::iter::empty())
    }

    fn subwidgets(&self) -> Box<dyn std::iter::Iterator<Item=&dyn Widget> + '_> where Self: Sized {
        error!("call to default subwidget on {} {}", self.typename(), self.id());
        Box::new(std::iter::empty())
    }

    fn set_focused(&mut self, wid: WID) -> bool {
        error!("deprecated set_focus");
        false
    }

    fn get_subwidget(&self, wid: WID) -> Option<&dyn Widget> where Self: Sized {
        for widget in self.subwidgets() {
            if widget.id() == wid {
                return Some(widget);
            }
        }

        None
    }

    fn get_subwidget_mut(&mut self, wid: WID) -> Option<&mut dyn Widget> where Self: Sized {
        for widget in self.subwidgets_mut() {
            if widget.id() == wid {
                return Some(widget);
            }
        }

        None
    }
}