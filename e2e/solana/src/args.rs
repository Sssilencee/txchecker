use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None, ignore_errors = true)]
pub struct Args {
    /// Path to a `rabbitmqp-server` binary
    #[arg(short, long)]
    pub rabbitmq: String,

    /// Path to a `txchecker` binary
    #[arg(short, long)]
    pub txchecker: String,

    /// Path to a `solana-test-validator` binary
    #[arg(short, long)]
    pub solana_test_validator: String,
}