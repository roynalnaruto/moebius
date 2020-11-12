use crate::{bindings::Moebius, broadcaster::Broadcaster};

use ethers::prelude::*;
use log::info;
use std::sync::Arc;

pub struct MoebiusWatcher<M> {
    #[allow(dead_code)]
    client: Arc<M>,
    moebius: Moebius<M>,
    broadcaster: Broadcaster,
}

impl<M: Middleware + 'static> MoebiusWatcher<M> {
    pub fn new(
        client: Arc<M>,
        moebius_addr: Address,
        broadcaster: Broadcaster,
    ) -> anyhow::Result<MoebiusWatcher<M>> {
        let moebius = Moebius::new(moebius_addr, Arc::clone(&client));

        Ok(Self {
            client,
            moebius,
            broadcaster,
        })
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        let mut stream = self.moebius.moebius_data_filter().stream().await?;

        while let Some(item) = stream.next().await {
            if let Ok(log) = item {
                let tx_sig = self
                    .broadcaster
                    .broadcast(log.program_id, log.account_id, log.packed_data)
                    .await?;
                info!("UpdateData: {}", tx_sig);
            }
        }

        Ok(())
    }
}
