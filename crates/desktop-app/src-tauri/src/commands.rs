use flux_core::internals::repository::Repository;
use crate::models::RepositoryInfo;

#[tauri::command]
pub fn open_repository(path: String) -> Result<RepositoryInfo, String> {
    let repo = Repository::open(Some(path)).map_err(|err| err.to_string())?;
    RepositoryInfo::from_repo(&repo)
}
