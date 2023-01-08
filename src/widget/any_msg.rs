use std::any::Any;
use std::fmt::Debug;

pub trait AnyMsg: Any + 'static + Debug + AsAny {}

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn boxed(self) -> Box<dyn AnyMsg> where Self: Sized, Self: AnyMsg {
        Box::new(self)
    }

    fn someboxed(self) -> Option<Box<dyn AnyMsg>> where Self: Sized, Self: AnyMsg {
        Some(self.boxed())
    }
}

impl<T: AnyMsg> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

impl dyn AnyMsg {
    pub fn as_msg<T: AnyMsg>(&self) -> Option<&T> {
        self.as_any().downcast_ref::<T>()
    }

    pub fn as_msg_mut<T: AnyMsg>(&mut self) -> Option<&mut T> {
        self.as_any_mut().downcast_mut::<T>()
    }
}
