use chrono::{DateTime, Utc};
use machete::models::events::{Event, EventGroup};
use machete_core::ids::InternalId;
use serde::Serialize;

#[derive(Default, Serialize, Debug)]
pub struct Log {
    pub id: InternalId,
    pub name: String,
    pub timestamp: DateTime<Utc>,
    pub description: Option<String>,
    pub events: Vec<Event>,
}

impl Log {
    // Generate user-facing log from database log and events
    // Relevant events are cloned from the list of all events
    pub fn from_log_events(log : EventGroup, events : &Vec<Event>) -> Log {
        let events = events.iter().filter(|e| e.log == Some(log.id)).cloned().collect();

        Log {
            id: log.id,
            name: log.name,
            timestamp: log.timestamp,
            description: log.description,
            events,
        }
    }
}