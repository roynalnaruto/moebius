use clap::{
    crate_description, crate_name, crate_version, App, AppSettings, Arg, ArgMatches, SubCommand,
};
use moebius::{
    instruction::{initialize, update_data},
    state::Moebius,
};
use rand::RngCore;
use solana_clap_utils::{
    fee_payer::fee_payer_arg,
    input_parsers::{pubkey_of, pubkey_of_signer, signer_of},
    input_validators::{is_url, is_valid_pubkey, is_valid_signer},
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
    native_token::*, program_pack::Pack, pubkey::Pubkey, signature::Signer, system_instruction,
    transaction::Transaction,
};
use std::{process::exit, sync::Arc};

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

fn rand_bytes(n: usize) -> Vec<u8> {
    let mut output = vec![0u8; n];
    rand::thread_rng().fill_bytes(output.as_mut_slice());
    output
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

fn command_initialize(config: &Config, account: &Pubkey) -> CommandResult {
    let minimum_balance_for_rent_exemption = if !config.sign_only {
        config
            .rpc_client
            .get_minimum_balance_for_rent_exemption(Moebius::LEN)?
    } else {
        0
    };

    let instructions = vec![
        system_instruction::create_account(
            &config.fee_payer,
            &account,
            minimum_balance_for_rent_exemption,
            Moebius::LEN as u64,
            &moebius::id(),
        ),
        initialize(&moebius::id(), account, &config.owner)?,
    ];

    Ok(Some((
        minimum_balance_for_rent_exemption,
        vec![instructions],
    )))
}

fn command_update_data(
    _config: &Config,
    moebius_account: &Pubkey,
    authority: &Pubkey,
    target_program: &Pubkey,
    target_account: &Pubkey,
    data: Vec<u8>,
) -> CommandResult {
    let (caller_account, _) = Pubkey::find_program_address(
        &[&target_program.to_bytes(), &target_account.to_bytes()],
        &moebius::id(),
    );

    let instructions = vec![update_data(
        &moebius::id(),
        moebius_account,
        authority,
        &caller_account,
        target_program,
        target_account,
        data,
    )?];

    Ok(Some((0u64, vec![instructions])))
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
                .about("Initialize Moebius")
                .arg(
                    Arg::with_name("account-keypair")
                        .long("account-keypair")
                        .value_name("ACCOUNT_KEYPAIR")
                        .validator(is_valid_signer)
                        .takes_value(true)
                        .required(true)
                        .help(
                            "Specify the moebius account. \
                             This may be a keypair file, the ASK keyword. \
                             Defaults to the client keypair.",
                        ),
                )
                .nonce_args(true)
                .offline_args(),
        )
        .subcommand(
            SubCommand::with_name("update-data")
                .about("Update data in an account via a moebius-compatible program")
                .arg(
                    Arg::with_name("moebius-account")
                        .long("moebius-account")
                        .value_name("MOEBIUS_ID")
                        .validator(is_valid_pubkey)
                        .takes_value(true)
                        .required(true)
                        .help("Specify the moebius account"),
                )
                .arg(
                    Arg::with_name("authority-keypair")
                        .long("authority-keypair")
                        .value_name("AUTHORITY_KEYPAIR")
                        .validator(is_valid_signer)
                        .takes_value(true)
                        .required(true)
                        .help(
                            "Specify the moebius authority key. \
                             This may be a keypair file, the ASK keyword. \
                             Defaults to the client keypair.",
                        ),
                )
                .arg(
                    Arg::with_name("target-program")
                        .long("target-program")
                        .value_name("TARGET_PROGRAM_ID")
                        .validator(is_valid_pubkey)
                        .takes_value(true)
                        .required(true)
                        .help("Specify the target program ID"),
                )
                .arg(
                    Arg::with_name("target-account")
                        .long("target-account")
                        .value_name("TARGET_ACCOUNT_ID")
                        .validator(is_valid_pubkey)
                        .takes_value(true)
                        .required(true)
                        .help("Specify the target account ID"),
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
            let (signer, account) = signer_of(&arg_matches, "account-keypair", &mut wallet_manager)
                .unwrap_or_else(|e| {
                    eprintln!("error: {}", e);
                    exit(1);
                });
            bulk_signers.push(signer);

            command_initialize(&config, &account.unwrap())
        }
        ("update-data", Some(arg_matches)) => {
            let moebius_account = pubkey_of(arg_matches, "moebius-account").unwrap();
            let target_program = pubkey_of(arg_matches, "target-program").unwrap();
            let target_account = pubkey_of(arg_matches, "target-account").unwrap();
            let (signer, authority) =
                signer_of(&arg_matches, "authority-keypair", &mut wallet_manager).unwrap_or_else(
                    |e| {
                        eprintln!("error: {}", e);
                        exit(1);
                    },
                );
            bulk_signers.push(signer);

            let mut data: Vec<u8> = vec![];
            let rand_val_bytes32 = rand_bytes(32usize);
            let rand_val_address = rand_bytes(20usize);
            let rand_val_uint256 = rand_bytes(32usize);
            data.extend_from_slice(rand_val_bytes32.as_slice());
            data.extend_from_slice(&[0u8; 12]);
            data.extend_from_slice(rand_val_address.as_slice());
            data.extend_from_slice(rand_val_uint256.as_slice());
            println!("data = {:?}", data);
            command_update_data(
                &config,
                &moebius_account,
                &authority.unwrap(),
                &target_program,
                &target_account,
                data,
            )
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
