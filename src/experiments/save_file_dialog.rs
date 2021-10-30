/*
this widget is supposed to offer:
- tree view on the right, along with scrolling,
- file list view on the most of display (with scrolling as well)
- filename edit box
- buttons save and cancel

I hope I will discover most of functional constraints while implementing it.
 */

use std::fmt::{Debug, Formatter};

use crate::experiments::focus_group::{FocusGroup, FocusGroupImpl};
use crate::experiments::from_geometry::from_geometry;
use crate::io::input_event::InputEvent;
use crate::io::output::Output;
use crate::layout::layout::Layout;
use crate::layout::leaf_layout::LeafLayout;
use crate::layout::split_layout::{SplitDirection, SplitLayout, SplitRule};
use crate::primitives::xy::XY;
use crate::widget::any_msg::AnyMsg;
use crate::widget::button::ButtonWidget;
use crate::widget::edit_box::EditBoxWidget;
use crate::widget::list_widget::ListWidget;
use crate::widget::mock_file_list::mock::{get_mock_file_list, MockFile};
use crate::widget::stupid_tree::{get_stupid_tree, StupidTree};
use crate::widget::tree_view::TreeViewWidget;
use crate::widget::widget::{get_new_widget_id, WID, Widget};

pub struct SaveFileDialogWidget {
    id: WID,

    layout: Box<dyn Layout<SaveFileDialogWidget>>,

    tree: StupidTree,
    tree_widget: TreeViewWidget<usize>,
    list_widget: ListWidget<MockFile>,
    edit_box: EditBoxWidget,

    ok_button: ButtonWidget,
    cancel_button: ButtonWidget,
}

#[derive(Clone, Copy, Debug)]
pub enum SaveFileDialogMsg {}

impl AnyMsg for SaveFileDialogMsg {}

impl SaveFileDialogWidget {
    pub fn new() -> Self {
        let layout = Box::new(
            SplitLayout::new(SplitDirection::Vertical)
                .with(SplitRule::Proportional(1.0),
                      Box::new(LeafLayout::<SaveFileDialogWidget>::new(
                          Box::new(|s| &s.tree_widget),
                          Box::new(|s| &mut s.tree_widget),
                      )),
                )
                .with(SplitRule::Proportional(4.0),
                      Box::new(SplitLayout::new(SplitDirection::Vertical)
                          .with(SplitRule::Proportional(1.0),
                                Box::new(LeafLayout::<SaveFileDialogWidget>::new(
                                    Box::new(|s| &s.list_widget),
                                    Box::new(|s| &mut s.list_widget),
                                )))
                          .with(SplitRule::Fixed(1),
                                Box::new(LeafLayout::<SaveFileDialogWidget>::new(
                                    Box::new(|s| &s.list_widget),
                                    Box::new(|s| &mut s.list_widget),
                                )))
                      ),
                )
        );

        let file_list = get_mock_file_list();
        let tree = get_stupid_tree();
        let tree_widget = TreeViewWidget::<usize>::new(Box::new(tree));
        let list_widget = ListWidget::new().with_items(file_list);
        let edit_box = EditBoxWidget::new();

        let ok_button = ButtonWidget::new("OK".to_owned());
        let cancel_button = ButtonWidget::new("Cancel".to_owned());

        let mut res = SaveFileDialogWidget {
            id: get_new_widget_id(),

            layout,
            tree: get_stupid_tree(),
            tree_widget,
            list_widget,
            edit_box,

            ok_button,
            cancel_button,

        };


        res
    }
}

impl Widget for SaveFileDialogWidget {
    fn id(&self) -> WID {
        self.id
    }

    fn min_size(&self) -> XY {
        self.layout.min_size(self)
    }

    fn size(&self, max_size: XY) -> XY {
        max_size
    }

    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
        todo!()
    }

    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
        todo!()
    }

    fn get_focused(&self) -> &dyn Widget {
        todo!()
    }

    fn get_focused_mut(&mut self) -> &mut dyn Widget {
        todo!()
    }

    fn render(&self, focused: bool, output: &mut Output) {
        let focused_op = if focused {
            Some(self.get_focused().id())
        } else {
            None
        };

        self.layout.render(self, focused_op, output)
    }
}