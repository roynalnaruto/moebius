use clap::{
    crate_description, crate_name, crate_version, value_t, App, AppSettings, Arg, ArgMatches,
    SubCommand,
};
use rustc_hex::FromHex;
use solana_clap_utils::{
    fee_payer::fee_payer_arg,
    input_parsers::{pubkey_of_signer, signer_of},
    input_validators::{is_url, is_valid_signer},
    keypair::{signer_from_path, DefaultSigner},
    nonce::*,
    offline::*,
};
use solana_cli_output::{return_signers, OutputFormat};
use solana_client::{
    blockhash_query::BlockhashQuery, rpc_client::RpcClient, rpc_config::RpcSendTransactionConfig,
};
use solana_remote_wallet::remote_wallet::RemoteWalletManager;
use solana_sdk::{
    commitment_config::CommitmentConfig, instruction::Instruction, message::Message,
    native_token::*, program_pack::Pack, pubkey::Pubkey, signature::Signer,
    transaction::Transaction,
};
use std::{process::exit, sync::Arc};
use uniswap_program::{instruction::initialize, state::UniswapOracle};

type Error = Box<dyn std::error::Error>;
type CommandResult = Result<Option<(u64, Vec<Vec<Instruction>>)>, Error>;

#[allow(dead_code)]
struct Config {
    rpc_client: RpcClient,
    verbose: bool,
    owner: Pubkey,
    fee_payer: Pubkey,
    commitment_config: CommitmentConfig,
    default_signer: DefaultSigner,
    nonce_account: Option<Pubkey>,
    nonce_authority: Option<Pubkey>,
    blockhash_query: BlockhashQuery,
    sign_only: bool,
}

fn check_fee_payer_balance(config: &Config, required_balance: u64) -> Result<(), Error> {
    let balance = config.rpc_client.get_balance(&config.fee_payer)?;
    if balance < required_balance {
        Err(format!(
            "Fee payer, {}, has insufficient balance: {} required, {} available",
            config.fee_payer,
            lamports_to_sol(required_balance),
            lamports_to_sol(balance)
        )
        .into())
    } else {
        Ok(())
    }
}

type SignersOf = Vec<(Box<dyn Signer>, Pubkey)>;
pub fn signers_of(
    matches: &ArgMatches<'_>,
    name: &str,
    wallet_manager: &mut Option<Arc<RemoteWalletManager>>,
) -> Result<Option<SignersOf>, Box<dyn std::error::Error>> {
    if let Some(values) = matches.values_of(name) {
        let mut results = Vec::new();
        for (i, value) in values.enumerate() {
            let name = format!("{}-{}", name, i + 1);
            let signer = signer_from_path(matches, value, &name, wallet_manager)?;
            let signer_pubkey = signer.pubkey();
            results.push((signer, signer_pubkey));
        }
        Ok(Some(results))
    } else {
        Ok(None)
    }
}

fn command_initialize(
    config: &Config,
    token0: [u8; 20],
    decimal0: u8,
    token1: [u8; 20],
    decimal1: u8,
) -> CommandResult {
    let minimum_balance_for_rent_exemption = if !config.sign_only {
        config
            .rpc_client
            .get_minimum_balance_for_rent_exemption(UniswapOracle::LEN)?
    } else {
        0
    };

    let (account, _) =
        Pubkey::find_program_address(&[&token0[..], &token1[..]], &uniswap_program::id());

    let instructions = vec![initialize(
        &uniswap_program::id(),
        &account,
        &moebius::id(),
        &config.owner,
        token0,
        decimal0,
        token1,
        decimal1,
    )?];

    Ok(Some((
        minimum_balance_for_rent_exemption,
        vec![instructions],
    )))
}

fn main() {
    let app_matches = App::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .arg({
            let arg = Arg::with_name("config_file")
                .short("C")
                .long("config")
                .value_name("PATH")
                .takes_value(true)
                .global(true)
                .help("Configuration file to use");
            if let Some(ref config_file) = *solana_cli_config::CONFIG_FILE {
                arg.default_value(&config_file)
            } else {
                arg
            }
        })
        .arg(
            Arg::with_name("verbose")
                .long("verbose")
                .short("v")
                .takes_value(false)
                .global(true)
                .help("Show additional information"),
        )
        .arg(
            Arg::with_name("json_rpc_url")
                .long("url")
                .value_name("URL")
                .takes_value(true)
                .global(true)
                .validator(is_url)
                .help("JSON RPC URL for the cluster.  Default from the configuration file."),
        )
        .arg(
            Arg::with_name("owner")
                .long("owner")
                .value_name("KEYPAIR")
                .validator(is_valid_signer)
                .takes_value(true)
                .global(true)
                .help(
                    "Specify the token owner account. \
                     This may be a keypair file, the ASK keyword. \
                     Defaults to the client keypair.",
                ),
        )
        .arg(fee_payer_arg().global(true))
        .subcommand(
            SubCommand::with_name("initialize")
                .about("Initialize Uniswap Oracle")
                .arg(
                    Arg::with_name("token0")
                        .long("token0")
                        .value_name("TOKEN_0")
                        .takes_value(true)
                        .required(true)
                        .help("Hex address of first token in Uniswap pair"),
                )
                .arg(
                    Arg::with_name("decimal0")
                        .long("decimal0")
                        .value_name("DECIMAL_0")
                        .takes_value(true)
                        .default_value("18")
                        .help("Decimal places in the first token in Uniswap pair"),
                )
                .arg(
                    Arg::with_name("token1")
                        .long("token1")
                        .value_name("TOKEN_1")
                        .takes_value(true)
                        .required(true)
                        .help("Hex address of second token in Uniswap pair"),
                )
                .arg(
                    Arg::with_name("decimal1")
                        .long("decimal1")
                        .value_name("DECIMAL_1")
                        .takes_value(true)
                        .default_value("18")
                        .help("Decimal places in the second token in Uniswap pair"),
                )
                .nonce_args(true)
                .offline_args(),
        )
        .get_matches();

    let mut wallet_manager = None;
    let mut bulk_signers: Vec<Option<Box<dyn Signer>>> = Vec::new();

    let (sub_command, sub_matches) = app_matches.subcommand();
    let matches = sub_matches.unwrap();

    let config = {
        let cli_config = if let Some(config_file) = matches.value_of("config_file") {
            solana_cli_config::Config::load(config_file).unwrap_or_default()
        } else {
            solana_cli_config::Config::default()
        };
        let json_rpc_url = matches
            .value_of("json_rpc_url")
            .unwrap_or(&cli_config.json_rpc_url)
            .to_string();

        let default_signer_arg_name = "owner".to_string();
        let default_signer_path = matches
            .value_of(&default_signer_arg_name)
            .map(|s| s.to_string())
            .unwrap_or_else(|| cli_config.keypair_path.clone());
        let default_signer = DefaultSigner {
            path: default_signer_path,
            arg_name: default_signer_arg_name,
        };

        bulk_signers.push(None);
        let owner = default_signer
            .signer_from_path(&matches, &mut wallet_manager)
            .unwrap_or_else(|e| {
                eprintln!("error: {}", e);
                exit(1);
            })
            .pubkey();

        let (signer, fee_payer) = signer_from_path(
            &matches,
            matches
                .value_of("fee_payer")
                .unwrap_or(&cli_config.keypair_path),
            "fee_payer",
            &mut wallet_manager,
        )
        .map(|s| {
            let p = s.pubkey();
            (Some(s), p)
        })
        .unwrap_or_else(|e| {
            eprintln!("error: {}", e);
            exit(1);
        });
        bulk_signers.push(signer);

        let verbose = matches.is_present("verbose");

        let nonce_account = pubkey_of_signer(&matches, NONCE_ARG.name, &mut wallet_manager)
            .unwrap_or_else(|e| {
                eprintln!("error: {}", e);
                exit(1);
            });
        let (signer, nonce_authority) =
            signer_of(&matches, NONCE_AUTHORITY_ARG.name, &mut wallet_manager).unwrap_or_else(
                |e| {
                    eprintln!("error: {}", e);
                    exit(1);
                },
            );
        if signer.is_some() {
            bulk_signers.push(signer);
        }

        let blockhash_query = BlockhashQuery::new_from_matches(matches);
        let sign_only = matches.is_present(SIGN_ONLY_ARG.name);

        Config {
            rpc_client: RpcClient::new(json_rpc_url),
            verbose,
            owner,
            fee_payer,
            commitment_config: CommitmentConfig::single_gossip(),
            default_signer,
            nonce_account,
            nonce_authority,
            blockhash_query,
            sign_only,
        }
    };

    solana_logger::setup_with_default("solana=info");

    let _ = match (sub_command, sub_matches) {
        ("initialize", Some(arg_matches)) => {
            // decode token0
            let mut token0 = [0u8; 20];
            let arg_token0 = arg_matches.value_of("token0").unwrap();
            token0.copy_from_slice(&arg_token0.from_hex::<Vec<u8>>().unwrap_or_else(|e| {
                eprintln!("decoding token0: {}", e);
                exit(1);
            }));
            let decimal0 = value_t!(arg_matches.value_of("decimal0"), u8).unwrap_or_else(|e| {
                eprintln!("invalid decimal0: {}", e);
                exit(1);
            });

            // decode token1
            let mut token1 = [0u8; 20];
            let arg_token1 = arg_matches.value_of("token1").unwrap();
            token1.copy_from_slice(&arg_token1.from_hex::<Vec<u8>>().unwrap_or_else(|e| {
                eprintln!("decoding token1: {}", e);
                exit(1);
            }));
            let decimal1 = value_t!(arg_matches.value_of("decimal1"), u8).unwrap_or_else(|e| {
                eprintln!("invalid decimal1: {}", e);
                exit(1);
            });

            command_initialize(&config, token0, decimal0, token1, decimal1)
        }
        _ => unreachable!(),
    }
    .and_then(|transaction_info| {
        if let Some((minimum_balance_for_rent_exemption, instruction_batches)) = transaction_info {
            let fee_payer = Some(&config.fee_payer);
            let signer_info = config
                .default_signer
                .generate_unique_signers(bulk_signers, &matches, &mut wallet_manager)
                .unwrap_or_else(|e| {
                    eprintln!("error: {}", e);
                    exit(1);
                });

            for instructions in instruction_batches {
                let message = if let Some(nonce_account) = config.nonce_account.as_ref() {
                    Message::new_with_nonce(
                        instructions,
                        fee_payer,
                        nonce_account,
                        config.nonce_authority.as_ref().unwrap(),
                    )
                } else {
                    Message::new(&instructions, fee_payer)
                };
                let (recent_blockhash, fee_calculator) = config
                    .blockhash_query
                    .get_blockhash_and_fee_calculator(&config.rpc_client, config.commitment_config)
                    .unwrap_or_else(|e| {
                        eprintln!("error: {}", e);
                        exit(1);
                    });

                if !config.sign_only {
                    check_fee_payer_balance(
                        &config,
                        minimum_balance_for_rent_exemption + fee_calculator.calculate_fee(&message),
                    )?;
                }

                let mut transaction = Transaction::new_unsigned(message);

                if config.sign_only {
                    transaction.try_partial_sign(&signer_info.signers, recent_blockhash)?;
                    println!("{}", return_signers(&transaction, &OutputFormat::Display)?);
                } else {
                    transaction.try_sign(&signer_info.signers, recent_blockhash)?;
                    let signature = config
                        .rpc_client
                        .send_and_confirm_transaction_with_spinner_and_config(
                            &transaction,
                            config.commitment_config,
                            RpcSendTransactionConfig {
                                preflight_commitment: Some(config.commitment_config.commitment),
                                ..RpcSendTransactionConfig::default()
                            },
                        )?;
                    println!("Signature: {}", signature);
                }
            }
        }
        Ok(())
    })
    .map_err(|err| {
        eprintln!("{}", err);
        exit(1);
    });
}
