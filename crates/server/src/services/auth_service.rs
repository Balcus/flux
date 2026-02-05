use jsonwebtoken::{EncodingKey, Header, encode};
use proto::models::{IssueTokenRequest, IssueTokenResponse, auth_serviec_server::AuthServiec};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};

use crate::user_store::UserStore;

#[derive(Debug)]
pub struct FluxAuthService {
    secret: String,
    user_store: Arc<Mutex<UserStore>>,
}

impl FluxAuthService {
    pub fn new(secret: String, user_store: Arc<Mutex<UserStore>>) -> Self {
        Self { secret, user_store }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    user_name: String,
    user_email: String,
}

#[tonic::async_trait]
impl AuthServiec for FluxAuthService {
    async fn issue_token(
        &self,
        request: Request<IssueTokenRequest>,
    ) -> Result<Response<IssueTokenResponse>, Status> {
        let req = request.into_inner();

        let claims = Claims {
            user_name: req.user_name.clone(),
            user_email: req.user_email.clone(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_ref()),
        )
        .map_err(|_| tonic::Status::internal("Failed to generate token"))?;

        self.user_store
            .lock()
            .await
            .add_user(req.user_name, req.user_email, token.clone())
            .await
            .map_err(|e| Status::internal(format!("Failed to add user to remote user store. {e}")))?;

        Ok(tonic::Response::new(proto::models::IssueTokenResponse {
            access_token: token,
        }))
    }
}
