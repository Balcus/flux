use crate::error;
use proto::models::Chunk;
use proto::models::{CloneRequest, UploadStatus};
use proto::models::{
    clone_service_client::CloneServiceClient, push_service_client::PushServiceClient,
};
use tokio_stream::wrappers::ReceiverStream;
use tonic::transport::Channel;
use url::Url;
// TODO: on clone create the direcotry for the repository, change iside the directory and then do the rest
// make the clone push to origin so it wont create a new folder on server if i just change the name of the local folder
pub type Result<T> = std::result::Result<T, error::GrpcClientError>;

const CHUNK_SIZE: usize = 256 * 1024;

#[derive(Debug)]
pub struct GrpcClient {
    url: String,
    pub push_client: PushServiceClient<Channel>,
    pub clone_client: CloneServiceClient<Channel>,
}

impl GrpcClient {
    pub async fn connect_remote(url: String) -> Result<Self> {
        let push_client = PushServiceClient::connect(url.clone()).await.map_err(|e| {
            error::GrpcClientError::ConnectRemote {
                url: url.clone(),
                source: e,
            }
        })?;
        let clone_client = CloneServiceClient::connect(url.clone())
            .await
            .map_err(|e| error::GrpcClientError::ConnectRemote {
                url: url.clone(),
                source: e,
            })?;
        Ok(Self {
            push_client,
            clone_client,
            url,
        })
    }

    fn repo_name(&self) -> Result<String> {
        let url = Url::parse(&self.url).map_err(|e| error::GrpcClientError::Url {
            url: self.url.clone(),
            source: Some(e),
        })?;
        let repo_name = url.path_segments().and_then(|p| p.last()).ok_or_else(|| {
            error::GrpcClientError::Url {
                url: self.url.clone(),
                source: None,
            }
        })?;
        Ok(repo_name.to_string())
    }

    pub async fn push(&mut self, repo_name: String, content: Vec<u8>) -> Result<UploadStatus> {
        let (tx, rx) = tokio::sync::mpsc::channel(32);

        tokio::spawn(async move {
            for chunk in content.chunks(CHUNK_SIZE) {
                let msg = Chunk {
                    repo_name: repo_name.clone(),
                    content: chunk.to_vec(),
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
        let response = self
            .push_client
            .push(request)
            .await
            .map_err(|e| error::GrpcClientError::Push(e))?;
        Ok(response.into_inner())
    }

    pub async fn clone_repository(&mut self) -> Result<Vec<u8>> {
        let repo_name = self.repo_name()?;

        let request = tonic::Request::new(CloneRequest { name: repo_name });

        let mut stream = self
            .clone_client
            .clone_repository(request)
            .await
            .map_err(|e| error::GrpcClientError::Clone(e))?
            .into_inner();

        let mut content = Vec::new();

        while let Some(chunk) = stream
            .message()
            .await
            .map_err(|e| error::GrpcClientError::Clone(e))?
        {
            content.extend_from_slice(&chunk.content);
        }

        Ok(content)
    }
}
