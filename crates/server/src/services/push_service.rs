use proto::models::push_service_server::PushService;
use proto::models::{Chunk, UploadStatus, UploadStatusCode};
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;
use tonic::{Request, Response, Status, Streaming};

#[derive(Debug, Default)]
pub struct FluxPushService {
    upload_root: String,
}

impl FluxPushService {
    pub fn new(upload_root: String) -> Self {
        Self { upload_root }
    }
}

#[tonic::async_trait]
impl PushService for FluxPushService {
    async fn push(
        &self,
        request: Request<Streaming<Chunk>>,
    ) -> Result<Response<UploadStatus>, Status> {
        let mut stream = request.into_inner();
        let mut repo_name: Option<String> = None;
        let mut total_bytes: u64 = 0;
        let mut chunks_received: u64 = 0;
        let mut buf: Vec<u8> = Vec::new();

        while let Some(chunk) = stream.message().await? {
            if repo_name.is_none() && !chunk.repo_name.is_empty() {
                repo_name = Some(chunk.repo_name.clone());
            }

            chunks_received += 1;
            total_bytes += chunk.content.len() as u64;
            buf.extend_from_slice(&chunk.content);
        }

        let repo_name = repo_name
            .filter(|n| !n.is_empty() && !n.chars().all(|c| c == '.'))
            .ok_or_else(|| Status::invalid_argument("Missing or invalid repository name"))?;

        let safe_name = std::path::Path::new(&repo_name)
            .file_name()
            .and_then(|f| f.to_str())
            .ok_or_else(|| Status::invalid_argument("Invalid repository name"))?;

        let repo_dir = PathBuf::from(self.upload_root.clone()).join(safe_name);
        tokio::fs::create_dir_all(&repo_dir).await.map_err(|e| {
            Status::internal(format!("Failed to create directory for repository: {}", e))
        })?;

        let archive_path = repo_dir.join(format!("{}.tar.gz", ".flux"));
        let mut file = tokio::fs::File::create(&archive_path)
            .await
            .map_err(|e| Status::internal(format!("Failed to create archive file: {}", e)))?;

        file.write_all(&buf)
            .await
            .map_err(|e| Status::internal(format!("Write failed: {}", e)))?;
        file.flush()
            .await
            .map_err(|e| Status::internal(format!("Flush failed: {}", e)))?;

        println!(
            "Stored '{}': {} chunks, {} bytes -> {:?}",
            safe_name, chunks_received, total_bytes, archive_path
        );

        Ok(Response::new(UploadStatus {
            response_message: format!(
                "Stored '{}' — {} chunks, {} bytes",
                safe_name, chunks_received, total_bytes
            ),
            code: UploadStatusCode::Ok as i32,
        }))
    }
}
