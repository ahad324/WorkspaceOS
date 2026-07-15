use crate::state::WorkspaceState;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::sync::broadcast;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FsEvent {
    Created(PathBuf),
    Modified(PathBuf),
    Deleted(PathBuf),
    Renamed(PathBuf, PathBuf),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WorkspaceEvent {
    WorkspaceCreated {
        id: String,
        name: String,
        root: PathBuf,
    },
    WorkspaceLoaded {
        id: String,
    },
    WorkspaceActivated {
        id: String,
    },
    WorkspaceStateChanged {
        id: String,
        old_state: WorkspaceState,
        new_state: WorkspaceState,
    },
    WorkspaceDeleted {
        id: String,
    },
    FsUpdate {
        id: String,
        event: FsEvent,
    },
}

pub struct WorkspaceEventBus {
    sender: broadcast::Sender<WorkspaceEvent>,
}

impl Default for WorkspaceEventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkspaceEventBus {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(4096);
        Self { sender }
    }

    pub fn publish(&self, event: WorkspaceEvent) {
        let _ = self.sender.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<WorkspaceEvent> {
        self.sender.subscribe()
    }
}
