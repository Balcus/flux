use proto::models::push_service_server::PushService;
use proto::models::{Chunk, UploadStatus, UploadStatusCode};
use tokio::sync::Mutex;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tonic::{Request, Response, Status, Streaming};

use crate::user_store::UserStore;

#[derive(Debug)]
pub struct FluxPushService {
    upload_root: String,
    user_store: Arc<Mutex<UserStore>>
}

impl FluxPushService {
    pub fn new(upload_root: String, user_store: Arc<Mutex<UserStore>>) -> Self {
        Self { upload_root, user_store }
    }
}

#[tonic::async_trait]
impl PushService for FluxPushService {
    async fn push(
        &self,
        request: Request<Streaming<Chunk>>,
    ) -> Result<Response<UploadStatus>, Status> {
        let metadata = request.metadata();

        let user_email = metadata
            .get("user-email")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| Status::unauthenticated("Missing user-email header"))?
            .to_string();

        let user_name = metadata
            .get("user-name")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| Status::unauthenticated("Missing user-name header"))?
            .to_string();

        let access_token = metadata
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or_else(|| Status::unauthenticated("Missing or invalid authorization token"))?
            .to_string();

        if !self.user_store.lock().await.is_token_valid(user_name.clone(), user_email, access_token).await {
            return Err(Status::permission_denied("Failed to validate user credentials"));
        }

        let mut stream = request.into_inner();
        let mut repo_name: Option<String> = None;
        let mut buf: Vec<u8> = Vec::new();

        while let Some(chunk) = stream.message().await? {
            if repo_name.is_none() && !chunk.repo_name.is_empty() {
                repo_name = Some(chunk.repo_name.clone());
            }
            buf.extend_from_slice(&chunk.content);
        }

        let raw_repo_name = repo_name
            .filter(|n| !n.is_empty() && !n.chars().all(|c| c == '.'))
            .ok_or_else(|| Status::invalid_argument("Missing or invalid repository name"))?;

        let safe_user_dir = std::path::Path::new(&user_name)
            .file_name()
            .and_then(|f| f.to_str())
            .ok_or_else(|| Status::invalid_argument("Invalid user name"))?;

        let safe_repo_name = std::path::Path::new(&raw_repo_name)
            .file_name()
            .and_then(|f| f.to_str())
            .ok_or_else(|| Status::invalid_argument("Invalid repository name"))?;

        let repo_dir = PathBuf::from(&self.upload_root)
            .join(safe_user_dir)
            .join(safe_repo_name);

        tokio::fs::create_dir_all(&repo_dir).await.map_err(|e| {
            Status::internal(format!("Failed to create directory: {}", e))
        })?;

        let archive_path = repo_dir.join(".flux.tar.gz");
        let mut file = tokio::fs::File::create(&archive_path)
            .await
            .map_err(|e| Status::internal(format!("Failed to create file: {}", e)))?;

        file.write_all(&buf).await.map_err(|e| Status::internal(e.to_string()))?;
        file.flush().await.map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(UploadStatus {
            response_message: format!("Stored in {}/{}", safe_user_dir, safe_repo_name),
            code: UploadStatusCode::Ok as i32,
        }))
    }
}
