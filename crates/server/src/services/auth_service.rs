use jsonwebtoken::{EncodingKey, Header, encode};
use proto::models::{IssueTokenRequest, IssueTokenResponse, auth_serviec_server::AuthServiec};
use serde::{Deserialize, Serialize};
use tonic::{Request, Response, Status};

#[derive(Debug, Default)]
pub struct FluxAuthService {
    secret: String
}

impl FluxAuthService {
    pub fn new(secret: String) -> Self {
        Self {
            secret
        }
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
            user_name: req.user_name,
            user_email: req.user_email,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_ref()),
        )
        .map_err(|_| tonic::Status::internal("Failed to generate token"))?;

        Ok(tonic::Response::new(proto::models::IssueTokenResponse {
            access_token: token,
        }))
    }
}
