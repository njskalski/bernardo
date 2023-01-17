use crate::widget::widget::Widget;

pub trait Getter<W>: Fn(&W) -> &dyn Widget {
    fn clone_box(&self) -> Box<dyn Getter<W>>;
}

impl<W, T: Fn(&W) -> &(dyn Widget) + Clone + 'static> Getter<W> for T {
    fn clone_box(&self) -> Box<dyn Getter<W>> {
        Box::new(self.clone())
    }
}

impl<W: 'static> Clone for Box<dyn Getter<W>> {
    fn clone(&self) -> Self { (**self).clone_box() }
}

pub trait GetterMut<W>: Fn(&mut W) -> &mut dyn Widget {
    fn clone_box(&self) -> Box<dyn GetterMut<W>>;
}

impl<W, T: Fn(&mut W) -> &mut (dyn Widget) + Clone + 'static> GetterMut<W> for T {
    fn clone_box(&self) -> Box<dyn GetterMut<W>> {
        Box::new(self.clone())
    }
}

impl<W: 'static> Clone for Box<dyn GetterMut<W>> {
    fn clone(&self) -> Self { (**self).clone_box() }
}

pub trait GetterOp<W>: Fn(&W) -> Option<&dyn Widget> {
    fn clone_box(&self) -> Box<dyn GetterOp<W>>;
}

impl<W, T: Fn(&W) -> Option<&(dyn Widget)> + Clone + 'static> GetterOp<W> for T {
    fn clone_box(&self) -> Box<dyn GetterOp<W>> {
        Box::new(self.clone())
    }
}

impl<W: 'static> Clone for Box<dyn GetterOp<W>> {
    fn clone(&self) -> Self { (**self).clone_box() }
}

pub trait GetterOpMut<W>: Fn(&mut W) -> Option<&mut dyn Widget> {
    fn clone_box(&self) -> Box<dyn GetterOpMut<W>>;
}

impl<W, T: Fn(&mut W) -> Option<&mut (dyn Widget)> + Clone + 'static> GetterOpMut<W> for T {
    fn clone_box(&self) -> Box<dyn GetterOpMut<W>> {
        Box::new(self.clone())
    }
}

impl<W: 'static> Clone for Box<dyn GetterOpMut<W>> {
    fn clone(&self) -> Self { (**self).clone_box() }
}


pub struct SubwidgetPointer<W: Widget> {
    getter: Box<dyn Getter<W>>,
    getter_mut: Box<dyn GetterMut<W>>,
}

impl<W: Widget> Clone for SubwidgetPointer<W> {
    fn clone(&self) -> Self {
        SubwidgetPointer {
            getter: self.getter.clone_box(),
            getter_mut: self.getter_mut.clone_box(),
        }
    }
}

impl<W: Widget> SubwidgetPointer<W> {
    pub fn new(getter: Box<dyn Getter<W>>, getter_mut: Box<dyn GetterMut<W>>) -> Self {
        SubwidgetPointer {
            getter,
            getter_mut,
        }
    }

    pub fn get<'a>(&self, parent: &'a W) -> &'a dyn Widget {
        (self.getter.clone())(parent)
    }

    pub fn get_mut<'a>(&self, parent: &'a mut W) -> &'a mut dyn Widget {
        (self.getter_mut.clone())(parent)
    }
}

pub struct SubwidgetPointerOp<W: Widget> {
    getter_op: Box<dyn GetterOp<W>>,
    getter_op_mut: Box<dyn GetterOpMut<W>>,
}

impl<W: Widget> Clone for SubwidgetPointerOp<W> {
    fn clone(&self) -> Self {
        SubwidgetPointerOp {
            getter_op: self.getter_op.clone_box(),
            getter_op_mut: self.getter_op_mut.clone_box(),
        }
    }
}

impl<W: Widget> SubwidgetPointerOp<W> {
    pub fn new(getter_op: Box<dyn GetterOp<W>>, getter_op_mut: Box<dyn GetterOpMut<W>>) -> Self {
        SubwidgetPointerOp {
            getter_op,
            getter_op_mut,
        }
    }

    pub fn get<'a>(&self, parent: &'a W) -> Option<&'a dyn Widget> {
        (self.getter_op)(parent)
    }

    pub fn get_mut<'a>(&self, parent: &'a mut W) -> Option<&'a mut dyn Widget> {
        (self.getter_op_mut)(parent)
    }
}

#[macro_export]
macro_rules! subwidget {
($parent: ident.$ child: ident) => {
    crate::experiments::subwidget_pointer::SubwidgetPointer::new(
        Box::new(|p : &$parent| { &p.$child}),
        Box::new(|p : &mut $parent| { &mut p.$child}),
    )
}
}

#[macro_export]
macro_rules! selfwidget {
($parent: ident) => {
    crate::experiments::subwidget_pointer::SubwidgetPointer::new(
        Box::new(|p : &$parent| { p as &dyn Widget}),
        Box::new(|p : &mut $parent| { p as &mut dyn Widget}),
    )
}
}

#[macro_export]
macro_rules! subwidget_op {
($parent: ident.$ child: ident) => {
    crate::experiments::subwidget_pointer::SubwidgetPointerOp::new(
        Box::new(|p : &$parent| { p.$child.as_ref().map(|w| {w as &dyn Widget}) }),
        Box::new(|p : &mut $parent| { p.$child.as_mut().map(|w| {w as &mut dyn Widget}) }),
    )
}
}

#[cfg(test)]
mod tests {
    use crate::config::theme::Theme;
    use crate::experiments::subwidget_pointer::{SubwidgetPointer, SubwidgetPointerOp};
    use crate::io::input_event::InputEvent;
    use crate::io::output::Output;
    use crate::primitives::size_constraint::SizeConstraint;
    use crate::primitives::xy::XY;
    use crate::widget::any_msg::AnyMsg;
    use crate::widget::widget::{WID, Widget};

    #[test]
    fn test_interface() {
        struct DummySubwidget {}
        impl Widget for DummySubwidget {
            fn id(&self) -> WID {
                unimplemented!()
            }

            fn typename(&self) -> &'static str {
                unimplemented!()
            }

            fn size(&self) -> XY {
                unimplemented!()
            }

            fn layout(&mut self, _sc: SizeConstraint) -> XY {
                unimplemented!()
            }

            fn on_input(&self, _input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
                unimplemented!()
            }

            fn update(&mut self, _msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
                unimplemented!()
            }

            fn render(&self, _theme: &Theme, _focused: bool, _output: &mut dyn Output) {
                unimplemented!()
            }
        }
        struct DummyWidget {
            subwidget: DummySubwidget,
            self_pointer: SubwidgetPointer<DummyWidget>,

            subwidget_op: Option<DummySubwidget>,
            self_pointer_op: SubwidgetPointerOp<DummyWidget>,
        }
        impl Widget for DummyWidget {
            fn id(&self) -> WID {
                unimplemented!()
            }

            fn typename(&self) -> &'static str {
                unimplemented!()
            }

            fn size(&self) -> XY {
                unimplemented!()
            }

            fn layout(&mut self, _sc: SizeConstraint) -> XY {
                unimplemented!()
            }

            fn on_input(&self, _input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
                unimplemented!()
            }

            fn update(&mut self, _msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
                unimplemented!()
            }

            fn render(&self, _theme: &Theme, _focused: bool, _output: &mut dyn Output) {
                unimplemented!()
            }
        }

        let _sp = SubwidgetPointer::new(
            Box::new(|dw: &DummyWidget| {
                &dw.subwidget
            }),
            Box::new(|dw: &mut DummyWidget| {
                &mut dw.subwidget
            }),
        );

        let sp3 = subwidget!(DummyWidget.subwidget);

        let _sp4 = sp3.clone();

        impl DummyWidget {
            pub fn new() -> Self {
                DummyWidget {
                    subwidget: DummySubwidget {},
                    self_pointer: subwidget!(DummyWidget.subwidget),
                    subwidget_op: Some(DummySubwidget {}),
                    self_pointer_op: subwidget_op!(DummyWidget.subwidget_op),
                }
            }
        }
    }
}