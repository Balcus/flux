use crate::error;
use proto::models::{Chunk, push_service_client::PushServiceClient};
use tokio_stream::wrappers::ReceiverStream;
use tonic::transport::Channel;
use proto::models::{UploadStatus};

pub type Result<T> = std::result::Result<T, error::GrpcClientError>;

#[derive(Debug)]
pub struct GrpcClient {
    pub client: PushServiceClient<Channel>,
}

impl GrpcClient {
    pub async fn connect_remote(url: String) -> Result<Self> {
        let client = PushServiceClient::connect(url.clone())
            .await
            .map_err(|e| error::GrpcClientError::ConnectRemote { url, source: e })?;
        Ok(Self { client })
    }

    pub async fn push(&mut self) -> Result<UploadStatus> {
        let (tx, rx) = tokio::sync::mpsc::channel(4);

        tokio::spawn(async move {
            let str = "Message sent from the client to the server. Check!";
            let chuck_size = 10;

            for chunk in str.as_bytes().chunks(chuck_size) {
                let msg = Chunk {
                    content: chunk.to_vec()
                };

                if tx.send(msg).await.is_err() {
                    eprint!("Receiver dropped");
                    break;
                }
            }
            println!("Finished sending chunks to server!");
        });

        let stream = ReceiverStream::new(rx);

        let request = tonic::Request::new(stream);
        let response = self.client.push(request).await.map_err(|e| error::GrpcClientError::Push(e))?;
        Ok(response.into_inner())
    }
}
