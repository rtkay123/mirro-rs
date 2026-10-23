use std::time::Duration;

use crossterm::event::{Event, EventStream, KeyEventKind};
use futures_util::StreamExt;
use tokio::time::{Interval, MissedTickBehavior, interval};
use tracing::{error, warn};

use super::InputEvent;
use super::key::Key;

/// Combines asynchronous terminal input with periodic UI refresh ticks.
pub struct Events {
    stream: EventStream,
    tick: Interval,
}

impl Events {
    /// Construct an event source with the given UI refresh interval.
    pub fn new(tick_rate: Duration) -> Events {
        let mut tick = interval(tick_rate);
        tick.set_missed_tick_behavior(MissedTickBehavior::Skip);
        Events {
            stream: EventStream::new(),
            tick,
        }
    }

    /// Wait for the next keypress or periodic UI refresh.
    pub async fn next(&mut self) -> InputEvent {
        loop {
            tokio::select! {
                event = self.stream.next() => match event {
                    Some(Ok(Event::Key(key))) if key.kind != KeyEventKind::Release => {
                        return InputEvent::Input(Key::from(key));
                    }
                    Some(Ok(_)) => continue,
                    Some(Err(error)) => {
                        error!("terminal input error: {error}");
                        return InputEvent::Tick;
                    }
                    None => {
                        warn!("terminal input stream closed");
                        return InputEvent::Tick;
                    }
                },
                _ = self.tick.tick() => return InputEvent::Tick,
            }
        }
    }
}
