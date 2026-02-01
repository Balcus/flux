use futures::stream::Stream;
use proto::models::clone_service_server::{CloneService, CloneServiceServer};
use proto::models::push_service_server::{PushService, PushServiceServer};
use proto::models::{Chunk, CloneRequest, UploadStatus, UploadStatusCode};
use tokio_stream::wrappers::ReceiverStream;
use std::path::PathBuf;
use std::pin::Pin;
use tokio::io::AsyncWriteExt;
use tonic::{Request, Response, Status, Streaming, transport::Server};

// TODO: cleanup, auth, cli with args to run server
// figure out how to test?!
const UPLOAD_FOLDER: &str = "uploads";
const CHUNK_SIZE: usize = 256 * 1024;

#[derive(Debug, Default)]
pub struct FluxPushService {}

#[derive(Debug, Default)]
pub struct FluxCloneService {}

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

        let repo_dir = PathBuf::from(UPLOAD_FOLDER).join(safe_name);
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

        tokio::spawn(async move {
            let uploads_path = PathBuf::from(UPLOAD_FOLDER).join(&name).join(".flux.tar.gz");
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

            for chunk_bytes in bytes.chunks(CHUNK_SIZE) {
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let push_service = FluxPushService::default();
    let clone_service = FluxCloneService::default();

    Server::builder()
        .add_service(PushServiceServer::new(push_service))
        .add_service(CloneServiceServer::new(clone_service))
        .serve(addr)
        .await?;

    Ok(())
}
