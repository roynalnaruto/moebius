//SPDX-License-Identifier: Unlicense
pragma solidity ^0.7.0;


contract SimpleContract {
  bytes32 accountId;
  bytes32 valBytes32;
  address valAddress;
  uint256 valUint256;

  constructor(
    bytes32 _accountId,
    bytes32 _valBytes32,
    address _valAddress,
    uint256 _valUint256
  ) {
    accountId = _accountId;
    valBytes32 = _valBytes32;
    valAddress = _valAddress;
    valUint256 = _valUint256;
  }

  function setAndGetValues(
    bytes32 _newAccountId,
    bytes32 _newValBytes32,
    address _newValAddress,
    uint256 _newValUint256
  )
    public
    returns (bytes32, bytes memory)
  {
    accountId = _newAccountId;
    valBytes32 = _newValBytes32;
    valAddress = _newValAddress;
    valUint256 = _newValUint256;
    return getValues();
  }

  function getValues()
    public
    view
    returns (bytes32 _accountId, bytes memory _packedData)
  {
    _accountId = accountId;
    _packedData = abi.encode(valBytes32, valAddress, valUint256);
  }
}
