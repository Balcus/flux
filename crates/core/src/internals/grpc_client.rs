use anyhow::Context;
use proto::models::auth_serviec_client::AuthServiecClient;
use proto::models::{Chunk, IssueTokenResponse};
use proto::models::{CloneRequest, UploadStatus};
use proto::models::{
    clone_service_client::CloneServiceClient, push_service_client::PushServiceClient,
};
use tokio_stream::wrappers::ReceiverStream;
use tonic::transport::Channel;
use url::Url;

// TODO: on clone create the direcotry for the repository, change iside the directory and then do the rest
// make the clone push to origin so it wont create a new folder on server if i just change the name of the local folder

const CHUNK_SIZE: usize = 256 * 1024;

#[derive(Debug)]
pub struct GrpcClient {
    url: String,
    pub auth_client: AuthServiecClient<Channel>,
    pub push_client: PushServiceClient<Channel>,
    pub clone_client: CloneServiceClient<Channel>,
}

impl GrpcClient {
    pub async fn connect_remote(url: String) -> anyhow::Result<Self> {
        let auth_client = AuthServiecClient::connect(url.clone())
            .await
            .with_context(|| format!("Failed to connect to remote repository at '{url}'."))?;
        let push_client = PushServiceClient::connect(url.clone())
            .await
            .with_context(|| format!("Failed to connect to remote repository at '{url}'."))?;
        let clone_client = CloneServiceClient::connect(url.clone())
            .await
            .with_context(|| format!("Failed to connect to remote repository at '{url}'."))?;
        Ok(Self {
            auth_client,
            push_client,
            clone_client,
            url,
        })
    }

    pub fn repo_name(&self) -> anyhow::Result<String> {
        let url = Url::parse(&self.url)
            .with_context(|| format!("Failed to parse url: '{}'.", self.url))?;
        let repo_name = url
            .path_segments()
            .and_then(|mut p| p.next_back())
            .with_context(|| format!("Failed to parse url: '{}'.", self.url))?;
        Ok(repo_name.to_string())
    }

    pub async fn auth(
        &mut self,
        user_name: String,
        user_email: String,
    ) -> anyhow::Result<IssueTokenResponse> {
        let request = tonic::Request::new(proto::models::IssueTokenRequest {
            user_name,
            user_email,
        });
        let response = self
            .auth_client
            .issue_token(request)
            .await
            .context("Failed authentication for remote server.")?;
        Ok(response.into_inner())
    }

    pub async fn push(
        &mut self,
        repo_name: String,
        content: Vec<u8>,
        user_email: String,
        user_name: String,
        access_token: String,
    ) -> anyhow::Result<UploadStatus> {
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
        let mut request = tonic::Request::new(stream);
        request
            .metadata_mut()
            .insert("user-email", user_email.parse().unwrap());
        request
            .metadata_mut()
            .insert("user-name", user_name.parse().unwrap());
        request.metadata_mut().insert(
            "authorization",
            format!("Bearer {}", access_token).parse().unwrap(),
        );

        let response = self
            .push_client
            .push(request)
            .await
            .context("Failed to push to remote repository.")?;
        Ok(response.into_inner())
    }

    pub fn extract_path(&self) -> anyhow::Result<String> {
        let url = Url::parse(&self.url)
            .with_context(|| format!("Failed to parse url: '{}'.", self.url))?;
        Ok(url.path().trim_start_matches('/').to_string())
    }

    pub async fn clone_repository(&mut self) -> anyhow::Result<Vec<u8>> {
        let path = self.extract_path()?;
        let request = tonic::Request::new(CloneRequest { name: path });

        let mut stream = self
            .clone_client
            .clone_repository(request)
            .await
            .context("Failed to clone repository.")?
            .into_inner();

        let mut content = Vec::new();
        while let Some(chunk) = stream
            .message()
            .await
            .context("Failed to clone repository.")?
        {
            content.extend_from_slice(&chunk.content);
        }

        Ok(content)
    }
}
