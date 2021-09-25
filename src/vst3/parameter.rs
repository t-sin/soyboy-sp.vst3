use std::convert::TryFrom;

pub enum PluginParameter {
    Param1 = 0,
}

impl TryFrom<u32> for PluginParameter {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value == PluginParameter::Param1 as u32 {
            Ok(PluginParameter::Param1)
        } else {
            Err(())
        }
    }
}
