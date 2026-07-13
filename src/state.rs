use std::path::PathBuf;

use crate::{APP_NAME, ENV_STATE_DIR};

pub fn get_state_dir() -> PathBuf {
    if let Some(env_state_dir) = ENV_STATE_DIR.value.clone() {
        return PathBuf::from(env_state_dir);
    }

    #[cfg(target_os = "windows")]
    {
        env::var_os("ProgramData")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(r"C:\ProgramData"))
            .join(APP_NAME)
    }

    #[cfg(target_os = "macos")]
    {
        PathBuf::from("/Library/Application Support").join(APP_NAME)
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        PathBuf::from("/var/lib").join(APP_NAME)
    }
}
