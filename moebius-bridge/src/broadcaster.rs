use anyhow::anyhow;
use moebius_program::instruction::update_data;
use solana_client::{
    client_error::ClientError, rpc_client::RpcClient, rpc_config::RpcSendTransactionConfig,
};
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::Instruction,
    message::Message,
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    transaction::Transaction,
};
use tokio::task::JoinHandle;

type BroadcastHandle = JoinHandle<Result<Signature, ClientError>>;

pub struct Broadcaster {
    authority: Keypair,
    moebius_account: Pubkey,
    rpc_url: String,
}

fn instruction_update_data(
    moebius_account: &Pubkey,
    authority: &Pubkey,
    target_program: &Pubkey,
    target_account: &Pubkey,
    data: Vec<u8>,
) -> Vec<Instruction> {
    let (caller_account, _) = Pubkey::find_program_address(
        &[&target_program.to_bytes(), &target_account.to_bytes()],
        &moebius_program::id(),
    );

    let instructions = vec![update_data(
        &moebius_program::id(),
        moebius_account,
        authority,
        &caller_account,
        target_program,
        target_account,
        data,
    )
    .unwrap()];

    instructions
}

impl Broadcaster {
    pub async fn new(
        rpc_url: String,
        authority: Keypair,
        moebius_account: Pubkey,
    ) -> anyhow::Result<Broadcaster> {
        Ok(Self {
            authority,
            moebius_account,
            rpc_url,
        })
    }

    pub async fn broadcast(
        &self,
        program_id: [u8; 32],
        account_id: [u8; 32],
        data: Vec<u8>,
    ) -> anyhow::Result<Signature> {
        // Data that will be moved into the blocking task.
        let rpc_url = self.rpc_url.clone();
        let moebius_account = self.moebius_account;
        let authority = Keypair::from_bytes(&self.authority.to_bytes()[..])?;
        let commitment_config = CommitmentConfig::single_gossip();
        let program_id = Pubkey::new_from_array(program_id);
        let account_id = Pubkey::new_from_array(account_id);

        let broadcast_task: BroadcastHandle = tokio::task::spawn_blocking(move || {
            // Initialize RPC client.
            let rpc_client = RpcClient::new(rpc_url);

            // Get the recent blockhash.
            let (recent_blockhash, _, _) = rpc_client
                .get_recent_blockhash_with_commitment(commitment_config)?
                .value;

            // Construct the instruction for updating data via Moebius.
            let instructions = instruction_update_data(
                &moebius_account,
                &authority.pubkey(),
                &program_id,
                &account_id,
                data,
            );

            // Construct transaction message.
            let message = Message::new(&instructions, Some(&authority.pubkey()));

            // Construct transaction.
            let mut transaction = Transaction::new_unsigned(message);

            // Sign the transaction using authority's key.
            transaction.try_sign(&[&authority], recent_blockhash)?;

            // Send transaction.
            Ok(rpc_client.send_transaction_with_config(
                &transaction,
                RpcSendTransactionConfig {
                    preflight_commitment: Some(commitment_config.commitment),
                    ..RpcSendTransactionConfig::default()
                },
            )?)
        });

        Ok(broadcast_task
            .await?
            .map_err(|e| anyhow!("Broadcast tx: {}", e.to_string()))?)
    }
}
