use std::env;
use std::path::PathBuf;

use lazy_static::lazy_static;

/// Wrapper for 'dirs' that treats MacOS more like Linux, by following the XDG specification.
/// The `XDG_CACHE_HOME` environment variable is checked first. `TITULAR_CONFIG_DIR`
/// is then checked before the `XDG_CONFIG_HOME` environment variable.
/// The fallback directory is `~/.config/titular`, respectively.
pub struct ProjectDirs {    
    config_dir: PathBuf,
}

impl ProjectDirs {
    fn new() -> Option<ProjectDirs> {

        // Checks whether or not $TITULAR_CONFIG_DIR exists. If it doesn't, set our config dir
        // to our system's default configuration home.
        let config_dir =
            if let Some(config_dir_op) = env::var_os("TITULAR_CONFIG_DIR").map(PathBuf::from) {
                config_dir_op
            } else {
                #[cfg(target_os = "macos")]
                let config_dir_op = env::var_os("XDG_CONFIG_HOME")
                    .map(PathBuf::from)
                    .filter(|p| p.is_absolute())
                    .or_else(|| dirs_next::home_dir().map(|d| d.join(".config")));

                #[cfg(not(target_os = "macos"))]
                let config_dir_op = dirs_next::config_dir();

                config_dir_op.map(|d| d.join("titular"))?
            };

        Some(ProjectDirs {
            config_dir,
        })
    }

    pub fn config_dir(&self) -> &PathBuf {
        &self.config_dir
    }
}

lazy_static! {
    pub static ref PROJECT_DIRS: ProjectDirs = ProjectDirs::new().expect("Could not get home directory");
}
