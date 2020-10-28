const { expect } = require("chai");

describe("Moebius", function() {
  const contracts = {};
  const moebiusABI = [
    "event MoebiusData(bytes32 _accountId, bytes _packedData)",
  ];
  const moebiusIface = new ethers.utils.Interface(moebiusABI);

  before(async function() {
    const Moebius = await ethers.getContractFactory("Moebius");
    const moebius = await Moebius.deploy();
    await moebius.deployed();

    const SimpleContract = await ethers.getContractFactory("SimpleContract");
    const simpleContract = await SimpleContract.deploy(
      ethers.utils.hexlify(ethers.utils.randomBytes(32)),
      ethers.utils.hexlify(ethers.utils.randomBytes(32)),
      ethers.utils.hexlify(ethers.utils.randomBytes(20)),
      ethers.BigNumber.from(ethers.utils.randomBytes(32)),
    );
    await simpleContract.deployed();

    contracts.moebius = moebius;
    contracts.simpleContract = simpleContract;
  })

  it("should emit appropriate event", async function() {
    const accountId = ethers.utils.hexlify(ethers.utils.randomBytes(32));
    const valBytes32 = ethers.utils.hexlify(ethers.utils.randomBytes(32));
    const valAddress = ethers.utils.hexlify(ethers.utils.randomBytes(20));
    const valUint256 = ethers.BigNumber.from(ethers.utils.randomBytes(32));
    await contracts.simpleContract.setAndGetValues(
      accountId,
      valBytes32,
      valAddress,
      valUint256,
    );
    const packedData = ethers.utils.defaultAbiCoder.encode(
      ['bytes32', 'address', 'uint256'],
      [valBytes32, valAddress, valUint256],
    );

    const target = contracts.simpleContract.address;
    const data = contracts.simpleContract.interface.encodeFunctionData("getValues", []);

    await expect(contracts.moebius.execute(target, data))
      .to.emit(contracts.moebius, 'MoebiusData');

    const blockNumber = await ethers.provider.getBlockNumber();
    const filter = {
      address: contracts.moebius.address,
      fromBlock: blockNumber,
    };
    const logs = await ethers.provider.getLogs(filter);
    let events = logs.map((log) => moebiusIface.parseLog(log));

    expect(events).to.have.lengthOf(1);
    expect(events[0].args._accountId).to.equal(accountId);
    expect(events[0].args._packedData).to.equal(packedData);
  })

  it("should set values and emit appropriate event", async function() {
    const accountId = ethers.utils.hexlify(ethers.utils.randomBytes(32));
    const valBytes32 = ethers.utils.hexlify(ethers.utils.randomBytes(32));
    const valAddress = ethers.utils.hexlify(ethers.utils.randomBytes(20));
    const valUint256 = ethers.BigNumber.from(ethers.utils.randomBytes(32));

    const target = contracts.simpleContract.address;
    const data = contracts.simpleContract.interface.encodeFunctionData(
      "setAndGetValues", [
        accountId,
        valBytes32,
        valAddress,
        valUint256,
      ]);

    const packedData = ethers.utils.defaultAbiCoder.encode(
      ['bytes32', 'address', 'uint256'],
      [valBytes32, valAddress, valUint256],
    );

    await expect(contracts.moebius.execute(target, data))
      .to.emit(contracts.moebius, 'MoebiusData');

    const blockNumber = await ethers.provider.getBlockNumber();
    const filter = {
      address: contracts.moebius.address,
      fromBlock: blockNumber,
    };
    const logs = await ethers.provider.getLogs(filter);
    let events = logs.map((log) => moebiusIface.parseLog(log));

    expect(events).to.have.lengthOf(1);
    expect(events[0].args._accountId).to.equal(accountId);
    expect(events[0].args._packedData).to.equal(packedData);
  })
})
