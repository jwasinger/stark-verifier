contract MerkleVerifierKeccak is MerkleVerifier {
    function hashNode(bytes32 left, bytes32 right)
        internal pure
        returns (bytes32 hash)
    {
        uint256 lhashMask = getHashMask();
        assembly {
            mstore(0x00, left)
            mstore(0x20, right)
            hash := and(lhashMask, keccak256(0x00, 0x40))
        }
        return hash;
    }
}

