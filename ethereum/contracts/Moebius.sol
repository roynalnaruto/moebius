//SPDX-License-Identifier: Unlicense
pragma solidity ^0.7.0;


contract Moebius {
  event MoebiusData(bytes32 indexed _accountId, bytes _packedData);

  function execute(address _target, bytes memory _data)
    public
    returns (bytes memory response)
  {
    require(_target != address(0), "target cannot be zero address");

    bytes32 _topic = keccak256("MoebiusData(bytes32,bytes)");
    assembly {
      let succeeded := call(sub(gas(), 5000), _target, 0, add(_data, 0x20), mload(_data), 0, 0)
      let size := returndatasize()

      response := mload(0x40)
      mstore(0x40, add(response, and(add(add(size, 0x20), 0x1f), not(0x1f))))
      mstore(response, size)
      returndatacopy(add(response, 0x20), 0, size)

      switch succeeded
      case 0 {
        revert(add(response, 0x20), size)
      }
      default {
        let _accountId := mload(add(response, 0x20))
        log2(add(response, 0x20), size, _topic, _accountId)
      }
    }
  }
}
