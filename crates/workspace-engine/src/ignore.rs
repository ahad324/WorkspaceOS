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
                let is_dir = path.is_dir();
                return gitignore.matched(relative, is_dir).is_ignore();
            }
        }

        false
    }
}
