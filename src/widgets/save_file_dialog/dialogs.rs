use crate::primitives::border::SINGLE_BORDER_STYLE;
use crate::primitives::printable::Printable;
use crate::widget::any_msg::AsAny;
use crate::widgets::button::ButtonWidget;
use crate::widgets::generic_dialog::generic_dialog::GenericDialog;
use crate::widgets::save_file_dialog::save_file_dialog_msg::SaveFileDialogMsg::{CancelOverride, ConfirmOverride};

const CANCEL_STRING: &str = "Cancel";
const OVERRIDE_STRING: &str = "Override";

pub fn override_dialog<T: Printable>(filename: T) -> GenericDialog {
    let mut text = "File \n\"".to_string();
    for grapheme in filename.graphemes() {
        text += grapheme;
    }

    text += "\"\n already exists.\n Do you wish to override?";

    GenericDialog::new(Box::new(text))
        .with_border(&SINGLE_BORDER_STYLE, Some(" Override file? ".to_string()))
        .with_option(ButtonWidget::new(Box::new(CANCEL_STRING)).with_on_hit(Box::new(|_| CancelOverride.someboxed())))
        .with_option(ButtonWidget::new(Box::new(OVERRIDE_STRING)).with_on_hit(Box::new(|_| ConfirmOverride.someboxed())))
}
