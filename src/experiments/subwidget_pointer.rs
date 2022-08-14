use crate::Widget;

trait Getter<W>: Fn(&W) -> &dyn Widget {
    fn clone_box(&self) -> Box<dyn Getter<W>>;
}

impl<W, T: Fn(&W) -> &(dyn Widget) + Clone + 'static> Getter<W> for T {
    fn clone_box(&self) -> Box<dyn Getter<W>> {
        Box::new(self.clone())
    }
}

impl<W: 'static> Clone for Box<dyn Getter<W>> {
    fn clone(&self) -> Self { self.clone_box() }
}

trait GetterMut<W>: Fn(&mut W) -> &mut dyn Widget {
    fn clone_box(&self) -> Box<dyn GetterMut<W>>;
}

impl<W, T: Fn(&mut W) -> &mut (dyn Widget) + Clone + 'static> GetterMut<W> for T {
    fn clone_box(&self) -> Box<dyn GetterMut<W>> {
        Box::new(self.clone())
    }
}

impl<W: 'static> Clone for Box<dyn GetterMut<W>> {
    fn clone(&self) -> Self { self.clone_box() }
}

trait GetterOp<W>: Fn(&W) -> Option<&dyn Widget> {
    fn clone_box(&self) -> Box<dyn GetterOp<W>>;
}

impl<W, T: Fn(&W) -> Option<&(dyn Widget)> + Clone + 'static> GetterOp<W> for T {
    fn clone_box(&self) -> Box<dyn GetterOp<W>> {
        Box::new(self.clone())
    }
}

impl<W: 'static> Clone for Box<dyn GetterOp<W>> {
    fn clone(&self) -> Self { self.clone_box() }
}

trait GetterOpMut<W>: Fn(&mut W) -> Option<&mut dyn Widget> {
    fn clone_box(&self) -> Box<dyn GetterOpMut<W>>;
}

impl<W, T: Fn(&mut W) -> Option<&mut (dyn Widget)> + Clone + 'static> GetterOpMut<W> for T {
    fn clone_box(&self) -> Box<dyn GetterOpMut<W>> {
        Box::new(self.clone())
    }
}

impl<W: 'static> Clone for Box<dyn GetterOpMut<W>> {
    fn clone(&self) -> Self { self.clone_box() }
}


struct SubwidgetPointer<W: Widget> {
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

    fn get<'a>(&self, parent: &'a W) -> &'a dyn Widget {
        (self.getter.clone())(parent)
    }

    fn get_mut<'a>(&self, parent: &'a mut W) -> &'a mut dyn Widget {
        (self.getter_mut.clone())(parent)
    }
}

//
struct SubwidgetPointerOp<W: Widget> {
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

    fn get<'a>(&self, parent: &'a W) -> Option<&'a dyn Widget> {
        (self.getter_op)(parent)
    }

    fn get_mut<'a>(&self, parent: &'a mut W) -> Option<&'a mut dyn Widget> {
        (self.getter_op_mut)(parent)
    }
}

macro_rules! subwidget {
($parent: ident.$ child: ident) => {
    SubwidgetPointer::new(
        Box::new(|p : &($parent)| { &p.$child}),
        Box::new(|p : &mut ($parent)| { &mut p.$child}),
    )
}
}

macro_rules! subwidget_op {
($parent: ident.$ child: ident) => {
    SubwidgetPointerOp::new(
        Box::new(|p : &($parent)| { p.$child.as_ref().map(|w| {w as &dyn Widget}) }),
        Box::new(|p : &mut ($parent)| { p.$child.as_mut().map(|w| {w as &mut dyn Widget}) }),
    )
}
}

#[cfg(test)]
mod tests {
    use crate::{AnyMsg, InputEvent, Output, SizeConstraint, Theme, Widget};
    use crate::experiments::subwidget_pointer::{SubwidgetPointer, SubwidgetPointerOp};
    use crate::primitives::xy::XY;
    use crate::widget::action_trigger::ActionTrigger;
    use crate::widget::widget::WID;

    #[test]
    fn test_interface() {
        struct DummySubwidget {}
        impl Widget for DummySubwidget {
            fn id(&self) -> WID {
                todo!()
            }

            fn typename(&self) -> &'static str {
                todo!()
            }

            fn min_size(&self) -> XY {
                todo!()
            }

            fn layout(&mut self, sc: SizeConstraint) -> XY {
                todo!()
            }

            fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
                todo!()
            }

            fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
                todo!()
            }

            fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
                todo!()
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
                todo!()
            }

            fn typename(&self) -> &'static str {
                todo!()
            }

            fn min_size(&self) -> XY {
                todo!()
            }

            fn layout(&mut self, sc: SizeConstraint) -> XY {
                todo!()
            }

            fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>> {
                todo!()
            }

            fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>> {
                todo!()
            }

            fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output) {
                todo!()
            }

            fn get_subwidget(&self, wid: WID) -> Option<&dyn Widget> where Self: Sized {
                self.self_pointer_op.get(self)
            }

            fn get_subwidget_mut(&mut self, wid: WID) -> Option<&mut dyn Widget> where Self: Sized {
                (self.self_pointer_op.clone()).get_mut(self)
            }
        }

        let sp = SubwidgetPointer::new(
            Box::new(|dw: &DummyWidget| {
                &dw.subwidget
            }),
            Box::new(|dw: &mut DummyWidget| {
                &mut dw.subwidget
            }),
        );

        let sp3 = subwidget!(DummyWidget.subwidget);

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