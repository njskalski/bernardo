use crate::Widget;

struct SubwidgetPointer<W: Widget> {
    getter: Box<dyn Fn(&W) -> &dyn Widget>,
    getter_mut: Box<dyn Fn(&mut W) -> &mut dyn Widget>,
}

impl<W: Widget> SubwidgetPointer<W> {
    pub fn new(getter: Box<dyn Fn(&W) -> &dyn Widget>, getter_mut: Box<dyn Fn(&mut W) -> &mut dyn Widget>) -> Self {
        SubwidgetPointer {
            getter,
            getter_mut,
        }
    }

    pub fn sugar_new<G, GM>(getter: G, getter_mut: GM) -> Self
        where
            G: (Fn(&W) -> &dyn Widget) + 'static,
            GM: (Fn(&mut W) -> &mut dyn Widget) + 'static,
    {
        SubwidgetPointer::new(Box::new(getter), Box::new(getter_mut))
    }
}

macro_rules! subwidget {
($parent: ident.$ child: ident) => {
    SubwidgetPointer::sugar_new(
        |p : &($parent)| { &p.$child},
        |p : &mut ($parent)| { &mut p.$child}
    )
}
}

#[cfg(test)]
mod tests {
    use crate::{AnyMsg, InputEvent, Output, SizeConstraint, Theme, Widget};
    use crate::experiments::subwidget_pointer::SubwidgetPointer;
    use crate::primitives::xy::XY;
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
            self_pointer: Option<SubwidgetPointer<DummyWidget>>,
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
        }

        let sp = SubwidgetPointer::new(
            Box::new(|dw: &DummyWidget| {
                &dw.subwidget
            }),
            Box::new(|dw: &mut DummyWidget| {
                &mut dw.subwidget
            }),
        );

        let sp2 = SubwidgetPointer::sugar_new(
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
                    self_pointer: Some(subwidget!(DummyWidget.subwidget)),
                }
            }
        }
    }
}