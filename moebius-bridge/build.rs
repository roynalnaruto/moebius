use ethers::contract::Abigen;

fn main() {
    let bindings = Abigen::new("Moebius", "./abi/moebius.json")
        .expect("could not instantiate Abigen")
        .generate()
        .expect("could not generate bindings");

    bindings
        .write_to_file("./src/bindings/moebius.rs")
        .expect("could not write bindings to file");
}
