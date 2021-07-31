use crate::primitives::xy::XY;

pub trait SizedXY {
    fn size(&self) -> XY;
}
