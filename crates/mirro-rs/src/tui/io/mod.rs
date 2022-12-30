use std::sync::{atomic::AtomicBool, mpsc::Sender, Arc};

pub mod handler;

pub enum IoEvent {
    Initialise,
    ClosePopUp,
    Export {
        in_progress: Arc<AtomicBool>,
        progress_transmitter: Sender<f32>,
    },
}
