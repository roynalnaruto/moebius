use anyhow::anyhow;
use ethers::{middleware::nonce_manager::NonceManagerMiddleware, prelude::*, signers::LocalWallet};
use gumdrop::Options;
use moebius::{Broadcaster, MoebiusWatcher};
use serde::Deserialize;
use solana_sdk::{pubkey::Pubkey, signature::read_keypair_file};
use std::{convert::TryFrom, fs::File, path::PathBuf, str::FromStr, sync::Arc, time::Duration};

#[derive(Debug, Options, Clone)]
struct Opts {
    help: bool,

    #[options(help = "path to config json file", default = "config/localnet.json")]
    config: PathBuf,

    #[options(help = "path to Ethereum private key", default = "config/localnet.key")]
    eth_private_key: PathBuf,

    #[options(help = "path to Moebius authority key")]
    authority_key: PathBuf,

    #[options(help = "polling interval (ms)", default = "1000")]
    interval: u64,
}

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "MoebiusContract")]
    moebius_contract: Address,

    #[serde(rename = "EthRpcUrl")]
    eth_rpc_url: String,

    #[serde(rename = "SolanaRpcUrl")]
    solana_rpc_url: String,

    #[serde(rename = "MoebiusAccount")]
    moebius_account: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opts = Opts::parse_args_default_or_exit();
    let cfg: Config = serde_json::from_reader(File::open(opts.config.clone())?)?;

    if cfg.eth_rpc_url.starts_with("http") {
        let provider = Provider::<Http>::try_from(cfg.eth_rpc_url.clone())?;
        run(cfg, opts, provider).await?;
    } else {
        let ws = Ws::connect(cfg.eth_rpc_url.clone()).await?;
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
    let signer: LocalWallet = std::fs::read_to_string(opts.eth_private_key)?.parse()?;
    let address = signer.address();
    let provider = SignerMiddleware::new(provider, signer);
    let provider = NonceManagerMiddleware::new(provider, address);
    let client = Arc::new(provider);

    let keypair_path = opts
        .authority_key
        .to_str()
        .ok_or_else(|| anyhow!("Keypair path not utf-8"))?;
    let authority_key = read_keypair_file(keypair_path)
        .map_err(|e| anyhow!("Reading solana keypair file: {}", e.to_string()))?;
    let moebius_account = Pubkey::from_str(&cfg.moebius_account)?;
    let broadcaster =
        Broadcaster::new(cfg.solana_rpc_url.clone(), authority_key, moebius_account).await?;

    let mut watcher = MoebiusWatcher::new(client, cfg.moebius_contract, broadcaster)?;
    watcher.run().await?;

    Ok(())
}
