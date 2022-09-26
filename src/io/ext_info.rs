use serde::{Deserialize, Serialize, Serializer};

use crate::widget::widget::WID;

#[cfg(test)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ExtInfo(pub &'static str, pub WID);

#[cfg(not(test))]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ExtInfo();

impl Default for ExtInfo {
    fn default() -> Self {
        #[cfg(test)]
        {
            ExtInfo("", 0)
        }

        #[cfg(not(test))]
        {
            ExtInfo()
        }
    }
}
