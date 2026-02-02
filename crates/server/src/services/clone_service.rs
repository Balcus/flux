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
        let name = request.into_inner().name;
        let (tx, rx) = tokio::sync::mpsc::channel(32);
        let upload_root = self.upload_root.clone();
        let chunk_size = self.chunk_size;

        tokio::spawn(async move {
            let uploads_path = PathBuf::from(upload_root)
                .join(&name)
                .join(".flux.tar.gz");
            let bytes = match tokio::fs::read(&uploads_path).await {
                Ok(b) => b,
                Err(e) => {
                    let _ = tx
                        .send(Err(Status::internal(format!(
                            "Failed to read repository '{}': {}",
                            name, e
                        ))))
                        .await;
                    return;
                }
            };

            for chunk_bytes in bytes.chunks(chunk_size) {
                let chunk = Chunk {
                    repo_name: name.clone(),
                    content: chunk_bytes.to_vec(),
                };

                if tx.send(Ok(chunk)).await.is_err() {
                    eprintln!("Client dropped connection");
                    break;
                }
            }
        });

        let stream = ReceiverStream::new(rx);
        let boxed_stream: Self::CloneRepositoryStream = Box::pin(stream);

        Ok(Response::new(boxed_stream))
    }
}
