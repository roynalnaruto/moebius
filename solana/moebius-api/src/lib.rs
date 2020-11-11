use ethereum_types::{Address, U256};
use rustc_hex::FromHex;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{program_pack::Pack, pubkey::Pubkey};
use uniswap_program::state::UniswapOracle;

pub struct MoebiusApi {
    client: RpcClient,
}

impl MoebiusApi {
    pub fn new() -> Self {
        Self {
            client: RpcClient::new("http://127.0.0.1:8899".to_string()),
        }
    }

    pub fn with_rpc_url(mut self, url: String) -> Self {
        self.client = RpcClient::new(url);
        self
    }
}

#[derive(Debug, PartialEq)]
pub struct Pricefeed {
    pub token0: Address,
    pub amount0: U256,
    pub decimal0: u8,
    pub token1: Address,
    pub amount1: U256,
    pub decimal1: u8,
}

impl Pricefeed {
    pub fn price_token0_token1(&self) -> f64 {
        let amount0 = self.amount0.as_u128() as f64 / 10u128.pow(self.decimal0 as u32) as f64;
        let amount1 = self.amount1.as_u128() as f64 / 10u128.pow(self.decimal1 as u32) as f64;
        amount0 / amount1
    }

    pub fn price_token1_token0(&self) -> f64 {
        1.0f64 / self.price_token0_token1()
    }
}

impl MoebiusApi {
    pub fn uniswap_oracle(&self, token0: &str, token1: &str) -> anyhow::Result<Pricefeed> {
        let token0 = token0.from_hex::<Vec<u8>>()?;
        let token1 = token1.from_hex::<Vec<u8>>()?;

        let (uniswap_account, _) = Pubkey::find_program_address(
            &[&token0.as_slice(), &token1.as_slice()],
            &uniswap_program::id(),
        );

        let data = self.client.get_account_data(&uniswap_account)?;

        let oracle = UniswapOracle::unpack_unchecked(data.as_slice())?;

        Ok(Pricefeed {
            token0: Address::from_slice(&oracle.token0[..]),
            decimal0: oracle.decimal0,
            amount0: U256::from_big_endian(&oracle.amount0[..]),
            token1: Address::from_slice(&oracle.token1[..]),
            decimal1: oracle.decimal1,
            amount1: U256::from_big_endian(&oracle.amount1[..]),
        })
    }
}
