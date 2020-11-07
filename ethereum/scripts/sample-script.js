// We require the Hardhat Runtime Environment explicitly here. This is optional
// but useful for running the script in a standalone fashion through `node <script>`.
//
// When running the script with `hardhat run <script>` you'll find the Hardhat
// Runtime Environment's members available in the global scope.
const bs58 = require("bs58");
const hre = require("hardhat");
const Logger = require("pretty-logger");

const customConfig = {
  showMillis: true,
  showTimestamp: true,
  info: "gray",
  error: ["bgRed", "bold"],
  debug: "rainbow"
};

function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

async function main() {
  // Hardhat always runs the compile task when running scripts with its command
  // line interface.
  //
  // If this script is run directly using `node` you may want to call compile
  // manually to make sure everything is compiled
  const log = new Logger(customConfig);
  await hre.run("compile");

  // We get the contract to deploy
  const Moebius = await hre.ethers.getContractFactory("Moebius");
  const moebius = await Moebius.deploy();
  await moebius.deployed();
  log.info("Moebius deployed at: ", moebius.address);

  // Solana program ID and account in the base58 format.
  simpleProgramId = "9rCXCJDsnS53QtdXvYhYCAxb6yBE16KAQx5zHWfHe9QF";
  simpleAccountId = "Bt9xbg8fz3mQCuk4jwso1Daj9pLwPiXtgHeMZqUhuS9A";

  const SimpleContract = await hre.ethers.getContractFactory("SimpleContract");
  const simpleContract = await SimpleContract.deploy(
    "0x".concat(bs58.decode(simpleProgramId).toString("hex")),   // programId
    "0x".concat(bs58.decode(simpleAccountId).toString("hex")),   // accountId
    hre.ethers.utils.hexlify(hre.ethers.utils.randomBytes(32)),  // valBytes32
    hre.ethers.utils.hexlify(hre.ethers.utils.randomBytes(20)),  // valAddress
    hre.ethers.BigNumber.from(hre.ethers.utils.randomBytes(32)), // valUint256
  );
  await simpleContract.deployed();
  log.info("SimpleContract deployed at: ", simpleContract.address);

  const target = simpleContract.address;
  while (true) {
    const data = simpleContract.interface.encodeFunctionData(
      "setAndGetValues", [
        hre.ethers.utils.hexlify(hre.ethers.utils.randomBytes(32)),
        hre.ethers.utils.hexlify(hre.ethers.utils.randomBytes(20)),
        hre.ethers.BigNumber.from(hre.ethers.utils.randomBytes(32)),
      ]);

    const tx = await moebius.execute(target, data);
    log.info("setAndGetValues: ", tx.hash);

    log.debug("sleeping...\n");
    await sleep(10000);
  }
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main()
  .then(() => process.exit(0))
  .catch(error => {
    console.error(error);
    process.exit(1);
  });
