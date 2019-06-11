pragma solidity ^0.5.2;

import "PrimeFieldElement0.sol";

contract StarkParameters is PrimeFieldElement0 {
    uint256 constant internal N_COEFFICIENTS = 244;
    uint256 constant internal MASK_SIZE = 129;
    uint256 constant internal N_ROWS_IN_MASK = 100;
    uint256 constant internal N_COLUMNS_IN_MASK = 10;
    uint256 constant internal CONSTRAINTS_DEGREE_BOUND = 2;
    uint256 constant internal N_OODS_VALUES = MASK_SIZE + CONSTRAINTS_DEGREE_BOUND;
    uint256 constant internal N_OODS_COEFFICIENTS = N_OODS_VALUES;
    uint256 constant internal MAX_FRI_STEP = 3;
}
