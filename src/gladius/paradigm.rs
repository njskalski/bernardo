use log::debug;

use crate::io::input_event::InputEvent;
use crate::widget::any_msg::AnyMsg;
use crate::widget::widget::Widget;

// returns (consumed, message_to_parent)
pub fn recursive_treat_views(
    view: &mut dyn Widget,
    ie: InputEvent,
) -> (bool, Option<Box<dyn AnyMsg>>) {
    let my_desc = format!("{:?}", &view).clone();
    let my_id = view.id();

    let focused_child_op = view.get_focused_mut();
    let child_desc = format!("{:?}", &focused_child_op);

    debug!(target: "recursive_treat_views", "{:?}: event {:?}, active_child: {:?}", my_desc, ie, child_desc);

    // first, dig as deep as possible.
    let (child_have_consumed, message_from_child_op) = match focused_child_op {
        Some(focused_child) => {
            debug_assert!(focused_child.id() != my_id,
                          "widget {:?} pointed to itself as it's own child, causing stack overflow", view
            );


            recursive_treat_views(focused_child, ie)
        }
        None => (false, None)
    };
    debug!(target: "recursive_treat_views", "{:?}: event {:?}, active_child: {:?}, child_consumed: {}, message_from_child: {:?}",
            my_desc, ie, child_desc, child_have_consumed, &message_from_child_op);

    if child_have_consumed {
        return match message_from_child_op {
            None => (true, None),
            Some(message_from_child) => {
                let msg_from_child_text = format!("{:?}", &message_from_child);
                let my_message_to_parent = view.update(message_from_child);
                debug!(target: "recursive_treat_views", "{:?}: message_from_child: {:?} sent to me, responding {:?} to parent",
                        my_desc, msg_from_child_text, &my_message_to_parent);
                (true, my_message_to_parent)
            }
        };
    };

    // Either child did not consume (unwinding), or we're on the bottom of path.
    // We're here to consume the Input.
    match view.on_input(ie) {
        None => {
            debug!(target: "recursive_treat_views", "{:?}: did not consume {:?} either.", my_desc, ie);
            // we did not see this input as useful, unfolding the recursion:
            // no consume, no message.
            (false, None)
        }
        Some(internal_message) => {
            debug!(target: "recursive_treat_views", "{:?}: consumed {:?} and am pushing {:?} to myself", my_desc, ie, internal_message);
            let message_to_parent = view.update(internal_message);
            debug!(target: "recursive_treat_views", "{:?}: send {:?} to parent", my_desc, message_to_parent);
            // (message_to_parent.is_some(), message_to_parent)
            (true, message_to_parent)
        }
    }
}
