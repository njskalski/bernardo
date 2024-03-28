use std::time::Duration;

use crossbeam_channel::{select, Receiver};
use log::{debug, error, warn};

use crate::experiments::screen_shot::screenshot;
use crate::mocks::full_setup::FullSetup;
use crate::mocks::meta_frame::MetaOutputFrame;
use crate::widget::widget::Widget;

pub trait WithWaitFor {
    const DEFAULT_TIMEOUT: Duration = Duration::from_secs(3);
    const DEFAULT_TIMEOUT_IN_FRAMES: usize = 180; //60fps :D

    fn is_frame_based_wait(&self) -> bool;
    fn screenshot(&self) -> bool {
        self.last_frame().map(|frame| screenshot(&frame.buffer)).unwrap_or(false)
    }
    fn last_frame(&self) -> Option<&MetaOutputFrame>;

    fn set_last_frame(&mut self, meta_output_frame: MetaOutputFrame);

    fn output_receiver(&self) -> &Receiver<MetaOutputFrame>;

    /*
    waits with default timeout for condition F to be satisfied, returns whether that happened or not
     */
    fn wait_for<F: Fn(&Self) -> bool>(&mut self, condition: F) -> bool {
        // maybe it's already true?
        if self.last_frame().is_some() && condition(&self) {
            return true;
        }

        let mut waited_frames: usize = 0;

        /*
        When self.frame_based_wait == false, we wait at most DEFAULT_TIMEOUT for matching frame.
        Otherwise, we wait up to DEFAULT_TIMEOUT_IN_FRAMES frames, before returning false.
        The latter setting is designed for debugging, in continous integration it should be off.
         */

        if !self.is_frame_based_wait() {
            loop {
                select! {
                    recv(self.output_receiver()) -> frame_res => {
                        match frame_res {
                            Ok(frame) => {
                                self.set_last_frame(frame);
                                if condition(&self) {
                                    return true;
                                }
                                debug!("no hit on condition");
                            }
                            Err(e) => {
                                error!("error receiving frame: {:?}", e);
                                return false;
                            }
                        }
                    },
                    default(Self::DEFAULT_TIMEOUT) => {
                        error!("timeout, making screenshot.");
                        self.screenshot();
                        return false;
                    }
                }
            }
        } else {
            warn!("TEST WAIT-TIMEOUT IS DISABLED");
            loop {
                select! {
                    recv(self.output_receiver()) -> frame_res => {
                        match frame_res {
                            Ok(frame) => {
                                self.set_last_frame(frame);
                                if condition(&self) {
                                    return true;
                                }
                                debug!("no hit on condition");
                            }
                            Err(e) => {
                                error!("error receiving frame: {:?}", e);
                                return false;
                            }
                        }
                    }
                }
                waited_frames += 1;
                if waited_frames >= Self::DEFAULT_TIMEOUT_IN_FRAMES {
                    error!("waited {} frames to no avail", waited_frames);
                    return false;
                }
            }
        }
    }
}
