use std::sync::Arc;
use crate::cli::Cli;
use crate::services::auth_service::FluxAuthService;
use crate::services::clone_service::FluxCloneService;
use crate::services::push_service::FluxPushService;
use crate::user_store::UserStore;
use clap::Parser;
use proto::models::auth_serviec_server::AuthServiecServer;
use proto::models::clone_service_server::CloneServiceServer;
use proto::models::push_service_server::PushServiceServer;
use tokio::sync::Mutex;
use tonic::transport::Server;

mod cli;
mod services;
mod user_store;
mod error;

// TODO: Save token for each user on server and check request so the tokens match
// THIS NEEDS SOME BIG CLEANING AFTER IT IS DONE
// thinking about it I don't think any token should be checked for clone, i mean if it is bound to a repository than you can't have it before cloning.
// either let the user clone anything on the server that is in the uploads folder or save the token inside home, something like ~/.flux/credentials
// and those credentials would be considered if no local ones (per repo) are set.
// this gets way more complicated than i thought :))
// cleanup, see tower layer for tonic, see if we can have an auth interceptor
// figure out how to test?!

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let port = cli.port;
    let upload_root = cli.upload_root;
    let addr = format!("[::1]:{}", port).parse()?;
    let secret = cli.secret;
    let user_store = match UserStore::open(cli.user_store_path.clone()) {
        Ok(store) => store,
        Err(_) => UserStore::new(cli.user_store_path)?,
    };

    let shared_store = Arc::new(Mutex::new(user_store));
    let auth_service = FluxAuthService::new(secret, shared_store.clone());
    let push_service = FluxPushService::new(upload_root.clone(), shared_store.clone());
    let clone_service = FluxCloneService::new(upload_root.clone());

    Server::builder()
        .add_service(AuthServiecServer::new(auth_service))
        .add_service(PushServiceServer::new(push_service))
        .add_service(CloneServiceServer::new(clone_service))
        .serve(addr)
        .await?;

    Ok(())
}
