use std::collections::HashMap;
use std::sync::Arc;

use log::debug;

use crate::experiments::filename_to_language::filename_to_language;
use crate::fs::path::SPath;
use crate::LangId;
use crate::w7e::handler::NavCompRef;

/*
This class is supposed to group all available navcomp providers so editor can choose from them
whenever they want.
 */
pub struct NavCompGroup {
    navcomps: HashMap<LangId, NavCompRef>,
}

pub type NavCompGroupRef = Arc<NavCompGroup>;

impl NavCompGroup {
    pub fn new() -> Self {
        NavCompGroup {
            navcomps: Default::default()
        }
    }

    pub fn get_navcomp_for(&self, spath: &SPath) -> Option<NavCompRef> {
        // TODO(never) theoretically for the same language we can have multiple navcomps. I do not
        // intend to handle this case any time soon, but I prefer to use spath whenever possible.

        filename_to_language(spath).map(|lang_id| self.get_navcomp_for_lang(lang_id)).flatten()
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
}