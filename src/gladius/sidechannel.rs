#[cfg(not(test))]
pub mod x {
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
    pub struct SideChannel {
        is_recording: bool,
    }

    impl Default for SideChannel {
        fn default() -> Self {
            SideChannel {
                is_recording: false,
            }
        }
    }

    impl SideChannel {
        pub fn with_recording(self) -> Self {
            Self {
                is_recording: true,
                ..self
            }
        }

        pub fn is_recording(&self) -> bool {
            self.is_recording
        }
    }
}