use crate::experiments::deref_str::DerefStr;
use crate::primitives::border::SINGLE_BORDER_STYLE;
use crate::widget::any_msg::AsAny;
use crate::widgets::button::ButtonWidget;
use crate::widgets::generic_dialog::generic_dialog::GenericDialog;
use crate::widgets::save_file_dialog::save_file_dialog_msg::SaveFileDialogMsg::{CancelOverride, ConfirmOverride};

const CANCEL_STRING: &'static str = "Cancel";
const OVERRIDE_STRING: &'static str = "Override";

pub fn override_dialog<T: DerefStr>(filename: T) -> GenericDialog {
    let text = format!("File \n\"{}\"\n already exists.\n Do you wish to override?", filename.as_ref_str());

    GenericDialog::new(Box::new(text)).with_border(
        &SINGLE_BORDER_STYLE,
    ).with_option(
        ButtonWidget::new(Box::new(CANCEL_STRING)).with_on_hit(
            |_| {
                CancelOverride.someboxed()
            }
        )
    ).with_option(
        ButtonWidget::new(Box::new(OVERRIDE_STRING)).with_on_hit(
            |_| {
                ConfirmOverride.someboxed()
            }
        )
    )
}
