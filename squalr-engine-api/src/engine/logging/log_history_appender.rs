use crate::structures::logging::log_event::LogEvent;
use crossbeam_channel::Sender;
use log::Record;
use log4rs::append::Append;
use std::{
    collections::VecDeque,
    fmt,
    sync::{Arc, RwLock},
};

pub struct LogHistoryAppender {
    max_retain_size: usize,
    log_history: Arc<RwLock<VecDeque<LogEvent>>>,
    log_subscribers: Arc<RwLock<Vec<Sender<String>>>>,
}

impl LogHistoryAppender {
    pub fn new(
        log_history: Arc<RwLock<VecDeque<LogEvent>>>,
        log_subscribers: Arc<RwLock<Vec<Sender<String>>>>,
    ) -> Self {
        Self {
            max_retain_size: 4096,
            log_history,
            log_subscribers,
        }
    }
}

impl fmt::Debug for LogHistoryAppender {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self.log_history.read() {
            Ok(history) => formatter
                .debug_struct("LogHistoryAppender")
                .field("log_history_len", &history.len())
                .finish(),
            Err(_) => formatter
                .debug_struct("LogHistoryAppender")
                .field("log_history", &"<poisoned>")
                .finish(),
        }
    }
}

impl Append for LogHistoryAppender {
    fn append(
        &self,
        record: &Record,
    ) -> anyhow::Result<()> {
        let message_str = format!("{}", record.args());
        match self.log_history.write() {
            Ok(mut log_history) => {
                let level = record.level();
                let event = LogEvent {
                    message: message_str.clone(),
                    level,
                };

                while log_history.len() >= self.max_retain_size {
                    log_history.pop_front();
                }

                log_history.push_back(event);
            }
            Err(_error) => {
                // Just silently fail -- logging more errors inside a failing logging framework would risk infinite loops.
            }
        }

        if let Ok(mut subscribers) = self.log_subscribers.write() {
            subscribers.retain(|sender| sender.send(message_str.clone()).is_ok());
        }

        Ok(())
    }

    fn flush(&self) {}
}
