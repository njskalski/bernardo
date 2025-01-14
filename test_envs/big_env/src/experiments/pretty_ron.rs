use ron::ser::PrettyConfig;
use serde::Serialize;

pub trait ToPrettyRonString: Serialize {
    fn to_pretty_ron_string(&self) -> Result<String, ron::Error> {
        ron::ser::to_string_pretty(self, PrettyConfig::default())
    }
}
