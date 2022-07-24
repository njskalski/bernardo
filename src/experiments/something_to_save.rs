
//TODO this should be thrown away at some point

/*
Reasons for this thing to exist (use cases in order of importance):
- abstract over fs. I will need this for tests, and for remote filesystems.
- inotify support. Refresh support for when fs is changed in the background.
- fast queries. We need to execute "fuzzy search" over filenames. This requires precomputing a trie/patricia tree, and updating it on inotify.
- async IO without async runtime. I will test for infinite files support and I want to access huge files over internet.
 */

use std::iter;
use std::rc::Rc;

pub trait SomethingToSave {
    fn get_slices(&self) -> Box<dyn Iterator<Item=&[u8]> + '_>;
}

impl SomethingToSave for Vec<u8> {
    fn get_slices(&self) -> Box<dyn Iterator<Item=&[u8]> + '_> {
        Box::new(
            iter::once(
                self.as_slice()
            )
        )
    }
}

impl SomethingToSave for &str {
    fn get_slices(&self) -> Box<dyn Iterator<Item=&[u8]> + '_> {
        Box::new(
            iter::once(
                self.as_bytes()
            )
        )
    }
}

impl SomethingToSave for Rc<String> {
    fn get_slices(&self) -> Box<dyn Iterator<Item=&[u8]> + '_> {
        Box::new(
            iter::once(
                self.as_bytes()
            )
        )
    }
}
