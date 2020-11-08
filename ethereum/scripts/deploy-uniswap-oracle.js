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

async function main() {
  const log = new Logger(customConfig);

  // Solana program ID and account in the base58 format.
  uniswapProgramId = "G33TSUoKH1xM7bPXTMoQhGQhfwWkWT8dGaW6dunDQoen";
  uniswapAccountId = "DyYDszBZ8m92i9bJeQMhErkqQ4UPBG4pVZxmQL3CNnC";

  const UniswapOracle = await hre.ethers.getContractFactory("UniswapOracle");
  const uniswapOracle = await UniswapOracle.deploy(
    "0x".concat(bs58.decode(uniswapProgramId).toString("hex")), // programId
    "0x".concat(bs58.decode(uniswapAccountId).toString("hex")), // accountId
    "0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f",               // factory
    "0xc778417e063141139fce010982780140aa0cd5ab",               // weth
    "0x1f9840a85d5af5bf1d1762f925bdaddc4201f984",               // uni
  );
  await uniswapOracle.deployed();
  log.info("UniswapOracle deployed at: ", uniswapOracle.address);
}

main()
  .then(() => process.exit(0))
  .catch(error => {
    console.error(error);
    process.exit(1);
  });
