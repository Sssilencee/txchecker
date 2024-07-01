use std::time::Duration;

use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{message::Message, pubkey::Pubkey, signature::{Keypair, Signature}, system_instruction, transaction::Transaction};
use tokio::time::sleep;

pub async fn send_transfer(
    client: &RpcClient,
    receiver_pk: &Pubkey, sender_kp: &Keypair, sender_pk: &Pubkey,
    amount: u64,
) -> anyhow::Result<Signature> {
    let airdrop_amount = amount * 2;
    client.request_airdrop(&sender_pk, airdrop_amount).await?;

    loop {
        let balance = client.get_balance(&sender_pk).await?;
        if balance == airdrop_amount {
            break;
        }

        sleep(Duration::from_secs(1)).await;
    }

    let instruction = system_instruction::transfer(sender_pk, receiver_pk, amount);
    let blockhash = client.get_latest_blockhash().await?;
    let message = Message::new(
        &[instruction],
        Some(sender_pk),
    );
    let tx = Transaction::new(&[sender_kp], message, blockhash);

    Ok(client.send_and_confirm_transaction(&tx).await?)
}