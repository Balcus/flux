use proto::models::push_service_server::{PushService, PushServiceServer};
use proto::models::{Chunk, UploadStatus, UploadStatusCode};
use tonic::{Request, Response, Status, Streaming, transport::Server};

#[derive(Debug, Default)]
pub struct FluxPushService {}

#[tonic::async_trait]
impl PushService for FluxPushService {
    async fn push(
        &self,
        request: Request<Streaming<Chunk>>,
    ) -> Result<Response<UploadStatus>, Status> {
        let mut stream = request.into_inner();
        let mut total_bytes = 0;
        let mut chunks_received = 0;
        let mut full_message = Vec::new();

        while let Some(chunk) = stream.message().await? {
            chunks_received += 1;
            total_bytes += chunk.content.len();
            full_message.extend_from_slice(&chunk.content);

            println!(
                "Received chunk #{}: {} bytes",
                chunks_received,
                chunk.content.len()
            );
        }

        if let Ok(message) = String::from_utf8(full_message) {
            println!("Received message: {message}");
        };
        println!("Total: {} chunks, {} bytes", chunks_received, total_bytes);

        let reply = UploadStatus {
            response_message: format!(
                "Received {} chunks ({} bytes)",
                chunks_received, total_bytes
            ),
            code: UploadStatusCode::Ok as i32,
        };

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let service = FluxPushService::default();

    Server::builder()
        .add_service(PushServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
