// pub struct WrappedIterRef<T> {
//     internal_iter: Box<dyn Iterator<Item=T>>,
//     last: Option<T>,
// }
//
// impl<'a, T> WrappedIterRef<T> {
//     pub fn new<I: Iterator<Item=T>>(internal_iter: I) -> Self {
//         WrappedIterRef {
//             internal_iter: Box::new(internal_iter),
//             last: None,
//         }
//     }
// }
//
// impl<T> Iterator for WrappedIterRef<T> {
//     type Item = T;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         self.last = self.internal_iter.next();
//         self.last.as_ref()
//     }
// }
