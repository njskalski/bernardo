/*
This is an umberella struct to enable advanced testing scenarios, without introducing dependency
injection.
 */

#[cfg(not(test))]
pub mod x {
    #[derive(Clone)]
    pub struct SideChannel {}

    impl Default for SideChannel {
        fn default() -> Self {
            SideChannel {}
        }
    }

    impl SideChannel {
        pub fn is_recording(&self) -> bool {
            false
        }
    }
}

#[cfg(test)]
pub mod x {
    use std::sync::{Arc, RwLock};

    use crossbeam_channel::{Receiver, Sender};

    use crate::mocks::mock_navcomp_provider::{MockCompletionMatcher, MockNavCompEvent, MockNavCompProviderPilot};

    pub struct SideChannelInternal {
        is_recording: bool,
        mock_navcomp_channel: (Sender<MockNavCompEvent>, Receiver<MockNavCompEvent>),
        mock_navcomp_pilot: MockNavCompProviderPilot,
        completions: Arc<RwLock<Vec<MockCompletionMatcher>>>,
    }

    pub type SideChannel = Arc<SideChannelInternal>;

    impl Default for SideChannelInternal {
        fn default() -> Self {
            let completions: Arc<RwLock<Vec<MockCompletionMatcher>>> = Default::default();
            let mock_navcomp_channel: (Sender<MockNavCompEvent>, Receiver<MockNavCompEvent>) = crossbeam_channel::unbounded::<MockNavCompEvent>();
            let mock_navcomp_pilot = MockNavCompProviderPilot::new(
                mock_navcomp_channel.1.clone(),
                completions.clone(),
            );

            SideChannelInternal {
                is_recording: false,
                mock_navcomp_channel,
                mock_navcomp_pilot,
                completions,
            }
        }
    }

    impl SideChannelInternal {
        pub fn with_recording(self) -> Self {
            Self {
                is_recording: true,
                ..self
            }
        }

        pub fn is_recording(&self) -> bool {
            self.is_recording
        }

        pub fn get_navcomp_prov_args(&self) -> (Sender<MockNavCompEvent>, Arc<RwLock<Vec<MockCompletionMatcher>>>) {
            (self.mock_navcomp_channel.0.clone(), self.completions.clone())
        }

        pub fn get_navcomp_pilot(&self) -> &MockNavCompProviderPilot {
            &self.mock_navcomp_pilot
        }
    }
}