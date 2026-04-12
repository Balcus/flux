use flux_core::internals::repository::Repository;
use std::sync::Mutex;

pub struct AppState {
    pub repository: Mutex<Option<Repository>>,
}
