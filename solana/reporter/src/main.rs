use ethereum_types::U256;
use juniper::{
    graphql_object, graphql_value, EmptyMutation, EmptySubscription, FieldError, FieldResult,
    RootNode,
};
use rustc_hex::{FromHex, ToHex};
use simple_program::state::SimpleProgram;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{program_pack::Pack, pubkey::Pubkey};
use std::str::FromStr;
use tokio::task::JoinHandle;
use uniswap_program::state::UniswapOracle;
use warp::{http::Response, Filter};

#[derive(Clone)]
pub struct Database;

impl juniper::Context for Database {}

impl Database {
    fn new() -> Self {
        Self {}
    }
}

struct Pricefeed {
    token0: [u8; 20],
    decimal0: u8,
    amount0: [u8; 32],
    token1: [u8; 20],
    decimal1: u8,
    amount1: [u8; 32],
}

#[graphql_object(context = "Database")]
impl Pricefeed {
    fn token0(&self) -> String {
        self.token0.to_hex()
    }

    fn decimal0(&self) -> i32 {
        self.decimal0 as i32
    }

    fn amount0(&self) -> f64 {
        let amount0 = U256::from_big_endian(&self.amount0[..]);
        let amount0 = amount0.as_u128();
        amount0 as f64 / 10u128.pow(self.decimal0 as u32) as f64
    }

    fn token1(&self) -> String {
        self.token1.to_hex()
    }

    fn decimal1(&self) -> i32 {
        self.decimal1 as i32
    }

    fn amount1(&self) -> f64 {
        let amount1 = U256::from_big_endian(&self.amount1[..]);
        let amount1 = amount1.as_u128();
        amount1 as f64 / 10u128.pow(self.decimal1 as u32) as f64
    }

    async fn priceToken0Token1(&self) -> f64 {
        let amount0 = U256::from_big_endian(&self.amount0[..]);
        let amount0 = amount0.as_u128();
        let amount0 = amount0 as f64 / 10u128.pow(self.decimal0 as u32) as f64;

        let amount1 = U256::from_big_endian(&self.amount1[..]);
        let amount1 = amount1.as_u128();
        let amount1 = amount1 as f64 / 10u128.pow(self.decimal1 as u32) as f64;

        amount0 / amount1
    }

    async fn priceToken1Token0(&self) -> f64 {
        let amount0 = U256::from_big_endian(&self.amount0[..]);
        let amount0 = amount0.as_u128();
        let amount0 = amount0 as f64 / 10u128.pow(self.decimal0 as u32) as f64;

        let amount1 = U256::from_big_endian(&self.amount1[..]);
        let amount1 = amount1.as_u128();
        let amount1 = amount1 as f64 / 10u128.pow(self.decimal1 as u32) as f64;

        amount1 / amount0
    }
}

struct SimpleData {
    val_bytes32: [u8; 32],
    val_address: [u8; 20],
    val_uint256: [u8; 32],
}

#[graphql_object(context = "Database")]
impl SimpleData {
    fn val_bytes32(&self) -> String {
        self.val_bytes32.to_hex()
    }

    fn val_address(&self) -> String {
        self.val_address.to_hex()
    }

    fn val_uint256(&self) -> String {
        self.val_uint256.to_hex()
    }

    async fn val_as_hex(&self) -> String {
        String::from("as_hex")
    }
}

struct Query;

#[graphql_object(context = "Database")]
impl Query {
    async fn simple_data() -> FieldResult<SimpleData> {
        let task = tokio::task::spawn_blocking(|| {
            let simple_program_account =
                Pubkey::from_str("Bt9xbg8fz3mQCuk4jwso1Daj9pLwPiXtgHeMZqUhuS9A").unwrap();

            let rpc_client = RpcClient::new(String::from("http://localhost:8899"));

            let data = rpc_client
                .get_account_data(&simple_program_account)
                .unwrap();

            let simple = SimpleProgram::unpack_unchecked(data.as_slice()).unwrap();

            SimpleData {
                val_bytes32: simple.val_bytes32,
                val_address: simple.val_address,
                val_uint256: simple.val_uint256,
            }
        });

        Ok(task.await.unwrap())
    }

    async fn uniswap_oracle(token0: String, token1: String) -> FieldResult<Pricefeed> {
        let token0 = token0.from_hex::<Vec<u8>>().map_err(|e| {
            FieldError::new(
                &format!("Could not decode hex: {}", e),
                graphql_value!({ "internal_error": "Connection refused" }),
            )
        })?;
        let token1 = token1.from_hex::<Vec<u8>>().map_err(|e| {
            FieldError::new(
                &format!("Could not decode hex: {}", e),
                graphql_value!({ "internal_error": "Connection refused" }),
            )
        })?;

        let (account, _) = Pubkey::find_program_address(
            &[token0.as_slice(), token1.as_slice()],
            &uniswap_program::id(),
        );

        let task: JoinHandle<Result<Pricefeed, FieldError>> =
            tokio::task::spawn_blocking(move || {
                let rpc_client = RpcClient::new(String::from("http://localhost:8899"));

                let data = rpc_client.get_account_data(&account).map_err(|e| {
                    FieldError::new(
                        &format!("Could not get account: {}", e),
                        graphql_value!({ "internal_error": "Connection refused" }),
                    )
                })?;

                let oracle = UniswapOracle::unpack_unchecked(data.as_slice()).map_err(|e| {
                    FieldError::new(
                        &format!("Could not unpack account data: {}", e),
                        graphql_value!({ "internal_error": "Connection refused" }),
                    )
                })?;

                Ok(Pricefeed {
                    token0: oracle.token0,
                    decimal0: oracle.decimal0,
                    amount0: oracle.amount0,
                    token1: oracle.token1,
                    decimal1: oracle.decimal1,
                    amount1: oracle.amount1,
                })
            });

        Ok(task.await??)
    }
}

type Schema = RootNode<'static, Query, EmptyMutation<Database>, EmptySubscription<Database>>;

fn schema() -> Schema {
    Schema::new(Query, EmptyMutation::new(), EmptySubscription::new())
}

#[tokio::main]
async fn main() {
    let homepage = warp::path::end().map(|| {
        Response::builder()
            .header("content-type", "text/html")
            .body(
                "<html><h1>juniper_warp</h1><div>visit <a href=\"/graphiql\">/graphiql</a></html>"
                    .to_string(),
            )
    });

    let state = warp::any().map(Database::new);
    let graphql_filter = juniper_warp::make_graphql_filter(schema(), state.boxed());

    warp::serve(
        warp::get()
            .and(warp::path("graphiql"))
            .and(juniper_warp::graphiql_filter("/graphql", None))
            .or(homepage)
            .or(warp::path("graphql").and(graphql_filter)),
    )
    .run(([127, 0, 0, 1], 8080))
    .await
}
