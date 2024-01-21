use std::collections::HashMap;
use std::sync::Arc;

use log::debug;

use crate::experiments::filename_to_language::filename_to_language;
use crate::fs::path::SPath;
use crate::tsw::lang_id::LangId;
use crate::w7e::handler::NavCompRef;

#[derive(Debug, Copy, Clone)]
pub enum NavCompTick {
    LspTick(LangId, usize),
}

pub type NavCompTickSender = crossbeam_channel::Sender<NavCompTick>;
pub type NavCompTickRecv = crossbeam_channel::Receiver<NavCompTick>;

/*
This class is supposed to group all available navcomp providers so editor can choose from them
whenever they want.

There are two ways of requesting a navcomp: first is with path, always preferred.
Second is with "language".

It is considered faulty usage to request with language, when path is available.

TODO: navcomps probably should be scoped to a path/dirtree, or at least expose a "for file" method
 */
pub struct NavCompGroup {
    navcomps: HashMap<LangId, NavCompRef>,

    tick_sender: NavCompTickSender,
    tick_receiver: NavCompTickRecv,
}

pub type NavCompGroupRef = Arc<NavCompGroup>;

impl NavCompGroup {
    pub fn new() -> Self {
        let (tick_sender, tick_receiver) = crossbeam_channel::unbounded::<NavCompTick>();

        NavCompGroup {
            navcomps: Default::default(),
            tick_sender,
            tick_receiver,
        }
    }

    pub fn get_navcomp_for(&self, spath: &SPath) -> Option<NavCompRef> {
        // TODO(never) theoretically for the same language we can have multiple navcomps. I do not
        // intend to handle this case any time soon, but I prefer to use spath whenever possible.

        filename_to_language(spath)
            .map(|lang_id| self.get_navcomp_for_lang(lang_id))
            .flatten()
    }

    pub fn get_navcomp_for_lang(&self, lang: LangId) -> Option<NavCompRef> {
        self.navcomps.get(&lang).map(|n| n.clone())
    }

    // TODO(never) again, theoretically there could be multiple navcomps for one lang
    pub fn add_option(&mut self, lang_id: LangId, navcomp: NavCompRef) {
        debug!("adding navcomp [{:?}] for lang_id {}", navcomp, lang_id);

        self.navcomps.insert(lang_id, navcomp).map(|old| {
            debug!("removing old navcomp: [{:?}]", old);
        });
    }

    pub fn len(&self) -> usize {
        self.navcomps.len()
    }

    pub fn recvr(&self) -> &NavCompTickRecv {
        &self.tick_receiver
    }

    pub fn todo_sender(&self) -> &NavCompTickSender {
        &self.tick_sender
    }
}
