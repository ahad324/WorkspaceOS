use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkspaceState {
    Created,
    Initializing,
    Ready,
    Active,
    Paused,
    Indexing,
    Error,
    Recovering,
    Stopped,
}

impl WorkspaceState {
    pub fn can_transition_to(self, target: WorkspaceState) -> bool {
        match (self, target) {
            // Self-loop is always allowed
            (a, b) if a == b => true,
            
            // Core transitions
            (WorkspaceState::Created, WorkspaceState::Initializing) => true,
            (WorkspaceState::Initializing, WorkspaceState::Ready) => true,
            (WorkspaceState::Initializing, WorkspaceState::Error) => true,
            
            (WorkspaceState::Ready, WorkspaceState::Active) => true,
            (WorkspaceState::Ready, WorkspaceState::Stopped) => true,
            
            (WorkspaceState::Active, WorkspaceState::Indexing) => true,
            (WorkspaceState::Active, WorkspaceState::Paused) => true,
            (WorkspaceState::Active, WorkspaceState::Error) => true,
            (WorkspaceState::Active, WorkspaceState::Stopped) => true,
            
            (WorkspaceState::Indexing, WorkspaceState::Active) => true,
            (WorkspaceState::Indexing, WorkspaceState::Error) => true,
            
            (WorkspaceState::Paused, WorkspaceState::Active) => true,
            (WorkspaceState::Paused, WorkspaceState::Stopped) => true,
            
            (WorkspaceState::Error, WorkspaceState::Recovering) => true,
            (WorkspaceState::Error, WorkspaceState::Stopped) => true,
            
            (WorkspaceState::Recovering, WorkspaceState::Active) => true,
            (WorkspaceState::Recovering, WorkspaceState::Error) => true,
            
            (WorkspaceState::Stopped, WorkspaceState::Initializing) => true,
            
            _ => false,
        }
    }

    pub fn transition_to(self, target: WorkspaceState) -> Result<Self, String> {
        if self.can_transition_to(target) {
            Ok(target)
        } else {
            Err(format!(
                "Invalid workspace state transition from {:?} to {:?}",
                self, target
            ))
        }
    }
}
