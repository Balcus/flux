use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Cli {
    #[arg(short = 'p', long = "port", default_value_t = 50051)]
    pub port: usize,

    #[arg(long = "upload-root", default_value_t = String::from("uploads"))]
    pub upload_root: String,

    #[arg(long = "secret", short = 's')]
    pub secret: String
}
