use serde::{Deserialize, Serialize};

use crate::widget::widget::WID;

#[cfg(test)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct ExtInfo(pub &'static str, pub WID);

#[cfg(not(test))]
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct ExtInfo();

impl Default for ExtInfo {
    fn default() -> Self {
        #[cfg(test)]
        {
            ExtInfo("", 0)
        }

        #[cfg(not(test))]
        {
            ()
        }
    }
}