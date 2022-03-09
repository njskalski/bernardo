use crate::AnyMsg;

#[derive(Debug)]
pub enum GenericDialogMsg {
    Left,
    Right,
    Cancel,
    Hit,
    JustPassMessage(Box<dyn AnyMsg>),
}

impl AnyMsg for GenericDialogMsg {}
