use ignore::gitignore::{Gitignore, GitignoreBuilder};
use std::path::{Path, PathBuf};
use tracing::warn;

pub struct IgnoreMatcher {
    root: PathBuf,
    gitignore: Option<Gitignore>,
}

impl IgnoreMatcher {
    pub fn new(root: &Path) -> Self {
        let gitignore_path = root.join(".gitignore");
        let gitignore = if gitignore_path.exists() {
            let mut builder = GitignoreBuilder::new(root);
            if let Some(err) = builder.add(&gitignore_path) {
                warn!("Error parsing .gitignore in {:?}: {}", root, err);
            }
            match builder.build() {
                Ok(gi) => Some(gi),
                Err(err) => {
                    warn!("Failed to build gitignore for {:?}: {}", root, err);
                    None
                }
            }
        } else {
            None
        };

        Self {
            root: root.to_path_buf(),
            gitignore,
        }
    }

    pub fn is_ignored(&self, path: &Path) -> bool {
        // Force-ignore hidden configuration directories and large build artifact stores
        if let Some(str_path) = path.to_str() {
            let normalized = str_path.replace('\\', "/");
            if normalized.contains("/.git/")
                || normalized.ends_with("/.git")
                || normalized.contains("/.workspaceos/")
                || normalized.ends_with("/.workspaceos")
                || normalized.contains("/node_modules/")
                || normalized.contains("/target/")
            {
                return true;
            }
        }

        if let Some(ref gitignore) = self.gitignore {
            if let Ok(relative) = path.strip_prefix(&self.root) {
                let relative_unix = relative.to_string_lossy().replace('\\', "/");
                let is_dir = path.is_dir();
                if gitignore.matched(&relative_unix, is_dir).is_ignore() {
                    return true;
                }

                // Match parents recursively as directories
                let mut current = relative.parent();
                while let Some(parent) = current {
                    let parent_str = parent.to_string_lossy();
                    if parent_str.is_empty() || parent_str == "." {
                        break;
                    }
                    let parent_unix = parent_str.replace('\\', "/");
                    if gitignore.matched(&parent_unix, true).is_ignore() {
                        return true;
                    }
                    current = parent.parent();
                }
            }
        }

        false
    }
}
