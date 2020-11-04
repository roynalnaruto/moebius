pub use moebius_mod::*;
mod moebius_mod {
    #![allow(dead_code)]
    #![allow(unused_imports)]
    use ethers::{
        contract::{
            builders::{ContractCall, Event},
            Contract, Lazy,
        },
        core::{
            abi::{parse_abi, Abi, Detokenize, InvalidOutputType, Token, Tokenizable},
            types::*,
        },
        providers::Middleware,
    };
    #[doc = "Moebius was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    pub static MOEBIUS_ABI: Lazy<Abi> = Lazy::new(|| {
        serde_json :: from_str ("[\n  {\n    \"anonymous\": false,\n    \"inputs\": [\n      {\n        \"indexed\": false,\n        \"internalType\": \"bytes32\",\n        \"name\": \"_programId\",\n        \"type\": \"bytes32\"\n      },\n      {\n        \"indexed\": false,\n        \"internalType\": \"bytes32\",\n        \"name\": \"_accountId\",\n        \"type\": \"bytes32\"\n      },\n      {\n        \"indexed\": false,\n        \"internalType\": \"bytes\",\n        \"name\": \"_packedData\",\n        \"type\": \"bytes\"\n      }\n    ],\n    \"name\": \"MoebiusData\",\n    \"type\": \"event\"\n  },\n  {\n    \"inputs\": [\n      {\n        \"internalType\": \"address\",\n        \"name\": \"_target\",\n        \"type\": \"address\"\n      },\n      {\n        \"internalType\": \"bytes\",\n        \"name\": \"_data\",\n        \"type\": \"bytes\"\n      }\n    ],\n    \"name\": \"execute\",\n    \"outputs\": [\n      {\n        \"internalType\": \"bytes\",\n        \"name\": \"response\",\n        \"type\": \"bytes\"\n      }\n    ],\n    \"stateMutability\": \"nonpayable\",\n    \"type\": \"function\"\n  }\n]\n") . expect ("invalid abi")
    });
    #[derive(Clone)]
    pub struct Moebius<M>(Contract<M>);
    impl<M> std::ops::Deref for Moebius<M> {
        type Target = Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M: Middleware> std::fmt::Debug for Moebius<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(Moebius))
                .field(&self.address())
                .finish()
        }
    }
    impl<'a, M: Middleware> Moebius<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<Address>>(address: T, client: Arc<M>) -> Self {
            let contract = Contract::new(address.into(), MOEBIUS_ABI.clone(), client);
            Self(contract)
        }
        #[doc = "Calls the contract's `execute` (0x1cff79cd) function"]
        pub fn execute(&self, target: Address, data: Vec<u8>) -> ContractCall<M, Vec<u8>> {
            self.0
                .method_hash([28, 255, 121, 205], (target, data))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Gets the contract's `MoebiusData` event"]
        pub fn moebius_data_filter(&self) -> Event<M, MoebiusDataFilter> {
            self.0
                .event("MoebiusData")
                .expect("event not found (this should never happen)")
        }
    }
    #[derive(Clone, Debug, Default, Eq, PartialEq)]
    pub struct MoebiusDataFilter {
        pub program_id: [u8; 32],
        pub account_id: [u8; 32],
        pub packed_data: Vec<u8>,
    }
    impl MoebiusDataFilter {
        #[doc = r" Retrieves the signature for the event this data corresponds to."]
        #[doc = r" This signature is the Keccak-256 hash of the ABI signature of"]
        #[doc = r" this event."]
        pub const fn signature() -> H256 {
            H256([
                223, 154, 9, 116, 160, 119, 53, 228, 130, 126, 152, 4, 29, 185, 188, 201, 234, 183,
                51, 63, 11, 175, 6, 70, 19, 215, 79, 136, 211, 255, 72, 16,
            ])
        }
        #[doc = r" Retrieves the ABI signature for the event this data corresponds"]
        #[doc = r" to. For this event the value should always be:"]
        #[doc = r""]
        #[doc = "`MoebiusData(bytes32,bytes32,bytes)`"]
        pub const fn abi_signature() -> &'static str {
            "MoebiusData(bytes32,bytes32,bytes)"
        }
    }
    impl Detokenize for MoebiusDataFilter {
        fn from_tokens(tokens: Vec<Token>) -> Result<Self, InvalidOutputType> {
            if tokens.len() != 3 {
                return Err(InvalidOutputType(format!(
                    "Expected {} tokens, got {}: {:?}",
                    3,
                    tokens.len(),
                    tokens
                )));
            }
            #[allow(unused_mut)]
            let mut tokens = tokens.into_iter();
            let program_id =
                Tokenizable::from_token(tokens.next().expect("this should never happen"))?;
            let account_id =
                Tokenizable::from_token(tokens.next().expect("this should never happen"))?;
            let packed_data =
                Tokenizable::from_token(tokens.next().expect("this should never happen"))?;
            Ok(MoebiusDataFilter {
                program_id,
                account_id,
                packed_data,
            })
        }
    }
}
