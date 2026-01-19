pub mod commit;
pub mod diff;
pub mod log;
pub mod rollback;
pub mod start;
pub mod status;

use std::path::PathBuf;

pub fn get_db_path(custom_path: Option<PathBuf>) -> PathBuf {
    custom_path.unwrap_or_else(|| {
        std::env::current_dir()
            .unwrap()
            .join(".gitent")
            .join("gitent.db")
    })
}
