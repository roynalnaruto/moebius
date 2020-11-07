const hre = require("hardhat");
const Logger = require("pretty-logger");

const customConfig = {
  showMillis: true,
  showTimestamp: true,
  info: "gray",
  error: ["bgRed", "bold"],
  debug: "rainbow"
};

async function main() {
  const log = new Logger(customConfig);

  const Moebius = await hre.ethers.getContractFactory("Moebius");
  const moebius = await Moebius.deploy();
  await moebius.deployed();
  log.info("Moebius deployed at: ", moebius.address);
}

main()
  .then(() => process.exit(0))
  .catch(error => {
    console.error(error);
    process.exit(1);
  });
