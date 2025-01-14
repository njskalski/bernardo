use crate::promise::streaming_promise::{StreamingPromise, StreamingPromiseState};
use crate::promise::streaming_promise_impl::WrappedMspcReceiver;

#[test]
fn streaming_promise_map_basic_test_1() {
    let (sender, receive) = crossbeam_channel::unbounded::<i32>();
    let mut promise: Box<dyn StreamingPromise<i32>> = Box::new(WrappedMspcReceiver::new(receive).map(|item| item * 7));

    assert_eq!(promise.read().len(), 0);
    assert_eq!(promise.state(), StreamingPromiseState::Streaming);

    for _ in 0..3 {
        let u = promise.update();
        assert_eq!(u.has_changed, false);
        assert_eq!(u.state, StreamingPromiseState::Streaming);
    }

    sender.send(1).unwrap();
    sender.send(2).unwrap();

    {
        let u = promise.update();
        assert_eq!(u.has_changed, true);
        assert_eq!(promise.read().len(), 2);
        assert_eq!(promise.read(), &vec![7, 14]);
        assert_eq!(u.state, StreamingPromiseState::Streaming);
    }

    for _ in 0..3 {
        let u = promise.update();
        assert_eq!(u.has_changed, false);
        assert_eq!(promise.read().len(), 2);
        assert_eq!(u.state, StreamingPromiseState::Streaming);
    }

    sender.send(3).unwrap();

    {
        let u = promise.update();
        assert_eq!(u.has_changed, true);
        assert_eq!(promise.read().len(), 3);
        assert_eq!(promise.read(), &vec![7, 14, 21]);
        assert_eq!(u.state, StreamingPromiseState::Streaming);
    }

    for _ in 0..3 {
        let u = promise.update();
        assert_eq!(u.has_changed, false);
        assert_eq!(promise.read().len(), 3);
        assert_eq!(u.state, StreamingPromiseState::Streaming);
    }

    drop(sender);

    {
        let u = promise.update();
        assert_eq!(u.has_changed, true);
        assert_eq!(promise.read().len(), 3);
        assert_eq!(u.state, StreamingPromiseState::Finished);
    }
}
