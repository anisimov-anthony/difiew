use clap::Parser;

#[derive(Parser, Debug, Clone, Default)]
#[command(version, about)]
pub struct BinArgs {
    #[arg(long, default_value = "test-topic")]
    pub topic: String,

    #[arg(long, default_value = "/ip4/0.0.0.0/tcp/0")]
    pub tcp_listen: String,

    #[arg(long, default_value_t = 10)]
    pub heartbeat_interval: u64,
}
