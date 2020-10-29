use ethers::contract::Abigen;

static MOEBIUS_ABI: &str = r#"[
    event MoebiusData(bytes32 _accountId, bytes _packedData)
    function execute(address _target, bytes memory _data) public returns (bytes memory _response)
]"#;

fn main() {
    let bindings = Abigen::new("Moebius", MOEBIUS_ABI)
        .expect("could not instantiate Abigen")
        .generate()
        .expect("could not generate bindings");

    bindings
        .write_to_file("./src/bindings/moebius.rs")
        .expect("could not write bindings to file");
}
