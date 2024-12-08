// SPDX-License-Identifier: GPL-3.0
pragma solidity ^0.8.0;

interface CirclesHub {
    function transferThrough(
        address[] memory tokenOwners,
        address[] memory srcs,
        address[] memory dests,
        uint256[] memory wads
    ) external;

    function signup() external;
    
    function organizationSignup() external;
    
    function trust(address user, uint256 limit) external;
    
    function updateTrust(address canSendTo, uint256 limit) external;
} 