// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.5;
pragma experimental ABIEncoderV2;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "./RewardSource.sol";
import "./ScaleCodec.sol";
import "./OutboundChannel.sol";

enum ChannelId {Basic, Incentivized}

contract ETHApp is RewardSource, AccessControl {
    using ScaleCodec for uint128;

    uint128 public balance;

    mapping(ChannelId => Channel) public channels;

    event Locked(address sender, bytes32 recipient, uint128 amount);

    event Unlocked(bytes32 sender, address recipient, uint128 amount);

    bytes2 constant MINT_CALL = 0x4101;

    bytes32 public constant REWARD_ROLE = keccak256("REWARD_ROLE");

    // Minimum amount per lockup.
    uint256 public constant MIN_LOCK_VALUE = 0.001 ether;

    // Maximum amount per lockup
    uint256 public constant MAX_LOCK_VALUE = 2**128 - 1;

    struct Channel {
        address inbound;
        address outbound;
    }

    bytes32 public constant INBOUND_CHANNEL_ROLE =
        keccak256("INBOUND_CHANNEL_ROLE");

    constructor(
        address rewarder,
        Channel memory _basic,
        Channel memory _incentivized
    ) {
        balance = 0;

        Channel storage c1 = channels[ChannelId.Basic];
        c1.inbound = _basic.inbound;
        c1.outbound = _basic.outbound;

        Channel storage c2 = channels[ChannelId.Incentivized];
        c2.inbound = _incentivized.inbound;
        c2.outbound = _incentivized.outbound;

        _setupRole(REWARD_ROLE, rewarder);
        _setupRole(INBOUND_CHANNEL_ROLE, _basic.inbound);
        _setupRole(INBOUND_CHANNEL_ROLE, _incentivized.inbound);
    }

    function lock(bytes32 _recipient, ChannelId _channelId) public payable {
        require(msg.value >= MIN_LOCK_VALUE, "Value must be more than MIN_LOCK_VALUE");
        require(msg.value <= MAX_LOCK_VALUE, "Value must be less than MAX_LOCK_VALUE");
        require(
            _channelId == ChannelId.Basic ||
                _channelId == ChannelId.Incentivized,
            "Invalid channel ID"
        );

        // Can safely reduce precision due to the check against MAX_LOCK_VALUE above
        uint128 amount = uint128(msg.value);

        balance = balance + amount;

        emit Locked(msg.sender, _recipient, amount);

        bytes memory call = encodeCall(msg.sender, _recipient, amount);

        OutboundChannel channel =
            OutboundChannel(channels[_channelId].outbound);
        channel.submit(msg.sender, call);
    }

    function unlock(
        bytes32 _sender,
        address payable _recipient,
        uint128 _amount
    ) public onlyRole(INBOUND_CHANNEL_ROLE) {
        require(_amount > 0, "Must unlock a positive amount");
        require(
            balance >= _amount,
            "ETH token balances insufficient to fulfill the unlock request"
        );

        balance = balance - _amount;
        _recipient.transfer(_amount);
        emit Unlocked(_sender, _recipient, _amount);
    }

    // SCALE-encode payload
    function encodeCall(
        address _sender,
        bytes32 _recipient,
        uint128 _amount
    ) private pure returns (bytes memory) {
        return
            abi.encodePacked(
                MINT_CALL,
                _sender,
                bytes1(0x00), // Encode recipient as MultiAddress::Id
                _recipient,
                _amount.encode128()
            );
    }

    function reward(address payable _recipient, uint128 _amount)
        external
        override
        onlyRole(REWARD_ROLE)
    {
        _recipient.transfer(_amount);
    }
}
