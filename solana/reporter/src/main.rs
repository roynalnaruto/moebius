use ethereum_types::U256;
use juniper::{graphql_object, EmptyMutation, EmptySubscription, FieldResult, RootNode};
use rustc_hex::ToHex;
use simple_program::state::SimpleProgram;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{program_pack::Pack, pubkey::Pubkey};
use std::str::FromStr;
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
    amount0: [u8; 32],
    token1: [u8; 20],
    amount1: [u8; 32],
}

#[graphql_object(context = "Database")]
impl Pricefeed {
    fn token0(&self) -> String {
        self.token0.to_hex()
    }

    fn amount0(&self) -> f64 {
        let amount0 = U256::from_big_endian(&self.amount0[..]);
        let amount0 = amount0.as_u128();
        amount0 as f64 / 10u128.pow(18u32) as f64
    }

    fn token1(&self) -> String {
        self.token1.to_hex()
    }

    fn amount1(&self) -> f64 {
        let amount1 = U256::from_big_endian(&self.amount1[..]);
        let amount1 = amount1.as_u128();
        amount1 as f64 / 10u128.pow(18u32) as f64
    }

    async fn priceToken0Token1(&self) -> f64 {
        let amount0 = U256::from_big_endian(&self.amount0[..]);
        let amount1 = U256::from_big_endian(&self.amount1[..]);
        amount0.as_u128() as f64 / amount1.as_u128() as f64
    }

    async fn priceToken1Token0(&self) -> f64 {
        let amount0 = U256::from_big_endian(&self.amount0[..]);
        let amount1 = U256::from_big_endian(&self.amount1[..]);
        amount1.as_u128() as f64 / amount0.as_u128() as f64
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

    async fn uniswap_oracle() -> FieldResult<Pricefeed> {
        let task = tokio::task::spawn_blocking(|| {
            let uniswap_program_account =
                Pubkey::from_str("APdzXvwkCGixBQdKbceGyfXjRCDe8q5R3dzJreJoRhDg").unwrap();

            let rpc_client = RpcClient::new(String::from("http://localhost:8899"));

            let data = rpc_client
                .get_account_data(&uniswap_program_account)
                .unwrap();

            let oracle = UniswapOracle::unpack_unchecked(data.as_slice()).unwrap();

            Pricefeed {
                token0: oracle.token0,
                amount0: oracle.amount0,
                token1: oracle.token1,
                amount1: oracle.amount1,
            }
        });

        Ok(task.await.unwrap())
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
            .body(format!(
                "<html><h1>juniper_warp</h1><div>visit <a href=\"/graphiql\">/graphiql</a></html>"
            ))
    });

    let state = warp::any().map(move || Database::new());
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
