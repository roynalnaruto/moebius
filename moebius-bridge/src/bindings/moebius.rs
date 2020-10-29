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
        let abi_str = "[\n    event MoebiusData(bytes32 _accountId, bytes _packedData)\n    function execute(address _target, bytes memory _data) public returns (bytes memory _response)\n]" . replace ('[' , "") . replace (']' , "") . replace (',' , "") ;
        let split: Vec<&str> = abi_str
            .split("\n")
            .map(|x| x.trim())
            .filter(|x| !x.is_empty())
            .collect();
        parse_abi(&split).expect("invalid abi")
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
        #[doc = "Calls the contract's `execute` (0x4b64e492) function"]
        pub fn execute(&self, target: Address) -> ContractCall<M, Vec<u8>> {
            self.0
                .method_hash([75, 100, 228, 146], target)
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
    pub struct MoebiusDataFilter();
    impl MoebiusDataFilter {
        #[doc = r" Retrieves the signature for the event this data corresponds to."]
        #[doc = r" This signature is the Keccak-256 hash of the ABI signature of"]
        #[doc = r" this event."]
        pub const fn signature() -> H256 {
            H256([
                12, 186, 2, 153, 234, 218, 97, 112, 143, 230, 187, 185, 147, 160, 0, 188, 176, 159,
                105, 61, 15, 99, 29, 32, 245, 6, 55, 38, 229, 94, 224, 221,
            ])
        }
        #[doc = r" Retrieves the ABI signature for the event this data corresponds"]
        #[doc = r" to. For this event the value should always be:"]
        #[doc = r""]
        #[doc = "`MoebiusData()`"]
        pub const fn abi_signature() -> &'static str {
            "MoebiusData()"
        }
    }
    impl Detokenize for MoebiusDataFilter {
        fn from_tokens(tokens: Vec<Token>) -> Result<Self, InvalidOutputType> {
            if tokens.len() != 0 {
                return Err(InvalidOutputType(format!(
                    "Expected {} tokens, got {}: {:?}",
                    0,
                    tokens.len(),
                    tokens
                )));
            }
            #[allow(unused_mut)]
            let mut tokens = tokens.into_iter();
            Ok(MoebiusDataFilter())
        }
    }
}
