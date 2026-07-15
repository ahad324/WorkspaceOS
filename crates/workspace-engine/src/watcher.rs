use crate::event_bus::{FsEvent, WorkspaceEvent, WorkspaceEventBus};
use crate::ignore::IgnoreMatcher;
use crate::WorkspaceError;
use notify::{RecommendedWatcher, Watcher};
use notify_debouncer_full::{new_debouncer, DebouncedEvent, Debouncer};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};

pub struct WorkspaceWatcher {
    debouncer: Debouncer<RecommendedWatcher, notify_debouncer_full::FileIdMap>,
}

impl WorkspaceWatcher {
    pub fn new(
        id: &str,
        root: PathBuf,
        event_bus: Arc<WorkspaceEventBus>,
    ) -> Result<Self, WorkspaceError> {
        let root_clone = root.clone();
        let id_clone = id.to_string();
        let ignore_matcher = Arc::new(IgnoreMatcher::new(&root));

        let mut debouncer = new_debouncer(
            Duration::from_millis(500),
            None,
            move |result: Result<Vec<DebouncedEvent>, Vec<notify::Error>>| match result {
                Ok(events) => {
                    for event in events {
                        let kind = event.kind;
                        let paths = &event.paths;

                        // Filter out ignored paths
                        let mut filtered_paths = Vec::new();
                        for p in paths {
                            if !ignore_matcher.is_ignored(p) {
                                filtered_paths.push(p.clone());
                            }
                        }

                        if filtered_paths.is_empty() {
                            continue;
                        }

                        let fs_event = match kind {
                            notify::EventKind::Create(_) => {
                                if let Some(path) = filtered_paths.first() {
                                    path.strip_prefix(&root_clone)
                                        .map(|p| FsEvent::Created(p.to_path_buf()))
                                        .ok()
                                } else {
                                    None
                                }
                            }
                            notify::EventKind::Modify(notify::event::ModifyKind::Name(
                                notify::event::RenameMode::Both,
                            )) => {
                                if filtered_paths.len() >= 2 {
                                    let from = filtered_paths[0]
                                        .strip_prefix(&root_clone)
                                        .unwrap_or(&filtered_paths[0])
                                        .to_path_buf();
                                    let to = filtered_paths[1]
                                        .strip_prefix(&root_clone)
                                        .unwrap_or(&filtered_paths[1])
                                        .to_path_buf();
                                    Some(FsEvent::Renamed(from, to))
                                } else if let Some(path) = filtered_paths.first() {
                                    path.strip_prefix(&root_clone)
                                        .map(|p| FsEvent::Modified(p.to_path_buf()))
                                        .ok()
                                } else {
                                    None
                                }
                            }
                            notify::EventKind::Modify(_) => {
                                if let Some(path) = filtered_paths.first() {
                                    path.strip_prefix(&root_clone)
                                        .map(|p| FsEvent::Modified(p.to_path_buf()))
                                        .ok()
                                } else {
                                    None
                                }
                            }
                            notify::EventKind::Remove(_) => {
                                if let Some(path) = filtered_paths.first() {
                                    path.strip_prefix(&root_clone)
                                        .map(|p| FsEvent::Deleted(p.to_path_buf()))
                                        .ok()
                                } else {
                                    None
                                }
                            }
                            notify::EventKind::Any => {
                                if let Some(path) = filtered_paths.first() {
                                    path.strip_prefix(&root_clone)
                                        .map(|p| FsEvent::Modified(p.to_path_buf()))
                                        .ok()
                                } else {
                                    None
                                }
                            }
                            _ => None,
                        };

                        if let Some(fs_ev) = fs_event {
                            event_bus.publish(WorkspaceEvent::FsUpdate {
                                id: id_clone.clone(),
                                event: fs_ev,
                            });
                        }
                    }
                }
                Err(errors) => {
                    for err in errors {
                        error!("Watcher error: {}", err);
                    }
                }
            },
        )
        .map_err(|e| {
            WorkspaceError::IO(std::io::Error::other(format!(
                "Failed to create watcher: {}",
                e
            )))
        })?;

        // Start watching root directory recursively
        debouncer
            .watcher()
            .watch(&root, notify::RecursiveMode::Recursive)
            .map_err(|e| {
                WorkspaceError::IO(std::io::Error::other(format!(
                    "Failed to start watch root: {}",
                    e
                )))
            })?;

        // Add root path to debouncer cache
        debouncer
            .cache()
            .add_root(&root, notify::RecursiveMode::Recursive);

        info!(
            "Native event-driven filesystem watcher active on {:?}",
            root
        );
        Ok(Self { debouncer })
    }

    pub fn stop(mut self) {
        // Stop watching
        let _ = self.debouncer.watcher().unwatch(&PathBuf::new());
    }
}
