use clap::{builder::ArgPredicate, Parser};

pub const SOLANA_CONFIG_NAME: &str = "solana.toml";
pub const SOLANA_CONFIG_NAME_DEV: &str = "solana.dev.toml";

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Enables dev mode
    #[arg(short, long, default_value="false")]
    pub dev: bool,

    /// Path to `.env` file
    #[arg(short, long, default_value = ".env")]
    pub env: String,

    /// Path to `solana.toml` file
    #[arg(
        short, long,
        default_value_if("dev", ArgPredicate::Equals("true".into()), SOLANA_CONFIG_NAME_DEV),
        default_value = SOLANA_CONFIG_NAME,
    )]
    pub solana_config: String,
}

pub fn load() -> Args {
    Args::parse()
}

pub fn load_default() -> Args {
    Args::parse_from([""; 0])
}