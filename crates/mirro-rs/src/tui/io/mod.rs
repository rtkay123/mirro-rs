use std::sync::{Arc, atomic::AtomicBool, mpsc::Sender};

pub mod handler;

pub enum IoEvent {
    Initialise,
    ClosePopUp,
    Export {
        in_progress: Arc<AtomicBool>,
        progress_transmitter: Sender<f32>,
    },
}
