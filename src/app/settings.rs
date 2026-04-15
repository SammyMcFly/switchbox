//
//
//


#[derive(Debug, Clone, Default)]
pub struct Visibility {
    pub show_video: bool,
    pub show_audio: bool,
    pub show_midi: bool,
    // pub show_parameters: bool,
    pub show_devices: bool,
    pub show_clients: bool,
}