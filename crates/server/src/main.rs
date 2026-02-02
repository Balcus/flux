use crate::cli::Cli;
use crate::services::clone_service::FluxCloneService;
use crate::services::push_service::FluxPushService;
use clap::Parser;
use proto::models::clone_service_server::CloneServiceServer;
use proto::models::push_service_server::PushServiceServer;
use tonic::transport::Server;

mod cli;
mod services;

// TODO: cleanup, auth, cli with args to run server
// figure out how to test?!

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let port = cli.port;
    let upload_root = cli.upload_root;

    let addr = format!("[::1]:{}", port).parse()?;
    let push_service = FluxPushService::new(upload_root.clone());
    let clone_service = FluxCloneService::new(upload_root.clone());

    Server::builder()
        .add_service(PushServiceServer::new(push_service))
        .add_service(CloneServiceServer::new(clone_service))
        .serve(addr)
        .await?;

    Ok(())
}
