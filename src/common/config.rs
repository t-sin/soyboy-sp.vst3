pub struct PluginConfig {
    pub waveform_view_enabled: bool,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            waveform_view_enabled: false,
        }
    }
}
