use ethers::{middleware::nonce_manager::NonceManagerMiddleware, prelude::*, signers::LocalWallet};
use gumdrop::Options;
use moebius::MoebiusWatcher;
use serde::Deserialize;
use std::{convert::TryFrom, fs::File, path::PathBuf, sync::Arc, time::Duration};

#[derive(Debug, Options, Clone)]
struct Opts {
    help: bool,

    #[options(help = "path to config json file", default = "config/localnet.json")]
    config: PathBuf,

    #[options(help = "path to private key", default = "config/localnet.key")]
    private_key: PathBuf,

    #[options(help = "polling interval (ms)", default = "1000")]
    interval: u64,
}

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "Moebius")]
    moebius: Address,
    #[serde(rename = "RpcUrl")]
    rpc_url: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opts = Opts::parse_args_default_or_exit();
    let cfg: Config = serde_json::from_reader(File::open(opts.config.clone())?)?;

    if cfg.rpc_url.starts_with("http") {
        let provider = Provider::<Http>::try_from(cfg.rpc_url.clone())?;
        run(cfg, opts, provider).await?;
    } else {
        let ws = Ws::connect(cfg.rpc_url.clone()).await?;
        let provider = Provider::new(ws);
        run(cfg, opts, provider).await?;
    }

    Ok(())
}

async fn run<P: JsonRpcClient + 'static>(
    cfg: Config,
    opts: Opts,
    provider: Provider<P>,
) -> anyhow::Result<()> {
    let provider = provider.interval(Duration::from_millis(opts.interval));
    let signer: LocalWallet = std::fs::read_to_string(opts.private_key)?.parse()?;
    let address = signer.address();
    let provider = SignerMiddleware::new(provider, signer);
    let provider = NonceManagerMiddleware::new(provider, address);
    let client = Arc::new(provider);

    let mut watcher = MoebiusWatcher::new(client, cfg.moebius)?;
    watcher.run().await?;

    Ok(())
}
