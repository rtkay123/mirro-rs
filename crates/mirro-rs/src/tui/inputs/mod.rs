use self::key::Key;

pub mod event;
pub mod key;

pub enum InputEvent {
    Input(Key),
    Tick,
}
