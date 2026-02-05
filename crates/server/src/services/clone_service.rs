use futures::stream::Stream;
use proto::models::clone_service_server::CloneService;
use proto::models::{Chunk, CloneRequest};
use std::path::PathBuf;
use std::pin::Pin;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

#[derive(Debug, Default)]
pub struct FluxCloneService {
    upload_root: String,
    chunk_size: usize,
}

impl FluxCloneService {
    pub fn new(upload_root: String) -> Self {
        Self {
            upload_root,
            chunk_size: 256 * 1024,
        }
    }
}

#[tonic::async_trait]
impl CloneService for FluxCloneService {
    type CloneRepositoryStream =
        Pin<Box<dyn Stream<Item = Result<Chunk, Status>> + Send + 'static>>;

    async fn clone_repository(
        &self,
        request: Request<CloneRequest>,
    ) -> Result<Response<Self::CloneRepositoryStream>, Status> {
        let full_path = request.into_inner().name;

        let parts: Vec<String> = full_path
            .split('/')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        if parts.len() < 2 {
            return Err(Status::invalid_argument(
                "Path must be in format 'user/repo'",
            ));
        }

        let user_name = parts[0].clone();
        let repo_name = parts[1].clone();

        let safe_user = std::path::Path::new(&user_name)
            .file_name()
            .and_then(|f| f.to_str())
            .ok_or_else(|| Status::invalid_argument("Invalid user name"))?
            .to_string();

        let safe_repo = std::path::Path::new(&repo_name)
            .file_name()
            .and_then(|f| f.to_str())
            .ok_or_else(|| Status::invalid_argument("Invalid repo name"))?
            .to_string();

        let (tx, rx) = tokio::sync::mpsc::channel(32);
        let upload_root = self.upload_root.clone();
        let chunk_size = self.chunk_size;

        tokio::spawn(async move {
            let uploads_path = PathBuf::from(upload_root)
                .join(safe_user)
                .join(&safe_repo)
                .join(".flux.tar.gz");

            let bytes = match tokio::fs::read(&uploads_path).await {
                Ok(b) => b,
                Err(_) => {
                    let _ = tx
                        .send(Err(Status::not_found(format!(
                            "File not found at {:?}",
                            uploads_path
                        ))))
                        .await;
                    return;
                }
            };

            for chunk_bytes in bytes.chunks(chunk_size) {
                let chunk = Chunk {
                    repo_name: safe_repo.clone(),
                    content: chunk_bytes.to_vec(),
                };

                if tx.send(Ok(chunk)).await.is_err() {
                    break;
                }
            }
        });

        Ok(Response::new(Box::pin(ReceiverStream::new(rx))))
    }
}
