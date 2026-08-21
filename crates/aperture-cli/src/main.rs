use aperture_client::Client;
use aperture_core::{BreakerConfig, BulkheadConfig, Decision, LimitConfig};
use aperture_server::ControlPlane;
use clap::{Parser, Subcommand};
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "aperture", about = "aperture traffic control tooling")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Hammer a tight local plane. Burst is small on purpose so some requests deny.
    Demo {
        #[arg(long, default_value = "20")]
        requests: u32,
    },
    Version,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    match cli.command {
        Commands::Demo { requests } => {
            let plane = ControlPlane::with_limits(
                LimitConfig {
                    rate: 0.0001,
                    burst: 3,
                    window_ms: 60_000,
                },
                BreakerConfig::default(),
                BulkheadConfig {
                    max_concurrent: 8,
                    max_queue: 0,
                    queue_timeout_ms: 1,
                },
            );
            let client = Client::new(Arc::new(plane));
            let mut allowed = 0u32;
            let mut denied = 0u32;
            for i in 0..requests {
                let outcome = client.check("demo")?;
                match outcome.decision {
                    Decision::Allow => {
                        allowed += 1;
                        client.release();
                        client.success();
                    }
                    Decision::Deny | Decision::Shed => {
                        denied += 1;
                    }
                }
                if i % 5 == 0 {
                    println!("req {} decision={:?}", i, outcome.decision);
                }
            }
            println!("done allowed={} denied={}", allowed, denied);
            anyhow::ensure!(
                denied > 0,
                "tight burst should deny something, got allowed={} denied=0",
                allowed
            );
        }
        Commands::Version => println!("aperture 0.1.0"),
    }
    Ok(())
}
