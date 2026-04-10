use std::sync::Mutex;
use flux_core::internals::repository::Repository;

pub struct AppState {
    pub repository: Mutex<Option<Repository>>,
}