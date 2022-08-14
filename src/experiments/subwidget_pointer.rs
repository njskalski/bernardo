use crate::Widget;

struct SubwidgetPointer<W: Widget> {
    getter: fn(&W) -> &dyn Widget,
    getter_mut: fn(&mut W) -> &mut dyn Widget,
}

impl<W: Widget> Clone for SubwidgetPointer<W> {
    fn clone(&self) -> Self {
        SubwidgetPointer {
            getter: self.getter,
            getter_mut: self.getter_mut,
        }
    }
}

impl<W: Widget> SubwidgetPointer<W> {
    pub fn new(getter: fn(&W) -> &dyn Widget, getter_mut: fn(&mut W) -> &mut dyn Widget) -> Self {
        SubwidgetPointer {
            getter,
            getter_mut,
        }
    }

    // pub fn sugar_new<G, GM>(getter: G, getter_mut: GM) -> Self
    //     where
    //         G: (Fn(&W) -> &dyn Widget) + 'static,
    //         GM: (Fn(&mut W) -> &mut dyn Widget) + 'static,
    // {
    //     SubwidgetPointer::new(Box::new(getter), Box::new(getter_mut))
    // }

    fn get<'a>(&'a self, parent: &'a W) -> &dyn Widget {
        (self.getter)(parent)
    }

    fn get_mut<'a>(&self, parent: &'a mut W) -> &'a mut dyn Widget {
        (self.getter_mut.clone())(parent)
    }
}

#[derive(Clone)]
struct SubwidgetPointerOp<W: Widget> {
    op: Option<SubwidgetPointer<W>>,
}

impl<W: Widget> SubwidgetPointerOp<W> {
    fn get<'a>(&'a self, parent: &'a W) -> Option<&dyn Widget> {
        self.op.as_ref().map(|sp| sp.get(parent))
    }

    fn get_mut<'a>(&'a self, parent: &'a mut W) -> Option<&mut dyn Widget> {
        self.op.as_ref().map(move |sp| sp.get_mut(parent))
    }
}

impl<W: Widget> From<SubwidgetPointer<W>> for SubwidgetPointerOp<W> {
    fn from(sp: SubwidgetPointer<W>) -> Self {
        SubwidgetPointerOp {
            op: Some(sp)
        }
    }
}

macro_rules! subwidget {
($parent: ident.$ child: ident) => {
    SubwidgetPointer::new(
        |p : &($parent)| { &p.$child},
        |p : &mut ($parent)| { &mut p.$child}
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
                Some(self.self_pointer.get(self))
            }

            fn get_subwidget_mut(&mut self, wid: WID) -> Option<&mut dyn Widget> where Self: Sized {
                let x = self.self_pointer.clone();
                Some(x.get_mut(self))
            }
        }

        let sp = SubwidgetPointer::new(
            |dw: &DummyWidget| {
                &dw.subwidget
            },
            |dw: &mut DummyWidget| {
                &mut dw.subwidget
            },
        );

        let sp3 = subwidget!(DummyWidget.subwidget);

        impl DummyWidget {
            pub fn new() -> Self {
                DummyWidget {
                    subwidget: DummySubwidget {},
                    self_pointer: subwidget!(DummyWidget.subwidget).into(),
                }
            }
        }
    }
}