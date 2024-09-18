use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli_args = server::cli::CliArgs::parse();
    server::run(cli_args).await
}
