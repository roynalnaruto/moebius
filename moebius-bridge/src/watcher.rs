use crate::bindings::Moebius;

use ethers::prelude::*;
use std::sync::Arc;

pub struct MoebiusWatcher<M> {
    client: Arc<M>,
    moebius: Moebius<M>,
}

impl<M: Middleware + 'static> MoebiusWatcher<M> {
    pub fn new(client: Arc<M>, moebius_addr: Address) -> anyhow::Result<MoebiusWatcher<M>> {
        let moebius = Moebius::new(moebius_addr, Arc::clone(&client));

        Ok(Self { client, moebius })
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        let mut stream = self.moebius.moebius_data_filter().stream().await?;

        while let Some(item) = stream.next().await {
            if let Ok(log) = item {
                dbg!("log: {:?}", log);
            }
        }

        Ok(())
    }
}
