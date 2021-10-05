# Changelog

This changelog format is based on the spec described on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) with the modification of the `YYYY-MM-DD` date format to `DD/MM/YYYY`.
This project adheres to [Semantic Versioning 2.0.0](https://semver.org/spec/v2.0.0.html).

<!-- This is an example of a update block
  ## [v1.0.0] - 01/01/2021
  ### Added
  ### Changed
  ### Deprecated
  ### Removed
  ### Fixed
  ### Security
-->

## [v0.3.0] - 05/10/2021

### Added

- Add/Sub/Mul/DivAssign operators
- From for a lot of number types
  - This allows conversions like `let decimal: Decimal = 1e308.into();`

### Changed

- Changed the `Display` implementation to accept precision arguments
  - The default amount for precision is 16.
- Cleaned up a lot of functions, that had little to no use
  - The library itself no longer shows any clippy warnings
  - The tests do due to not using `assert!` instead of `assert_eq!` in compare tests
- Moved functions about instantiation into the Decimal implementation
- `CACHED_POWERS` are now a lazy static value
- Operator functions to be implemented via their value instead of references
  - Operators on references now simply dereference and perform the operation
- Cleaned up changelog a little

### Fixed

- `to_exponential()` causes loop for precisions > 2
- `to_fixed()` causes panic for values without significant digits (like 10.0)
- Sort of fixed representation of large numbers (mainly the last decimal test) by multiplying with a large value, then rounding and then dividing again
  - It's not a pretty fix, but as the original library states: It's about speed, not precision.

### Removed

- The dedicated `to_string()` function in favour of the one supplied by `Display`
- `Ord` implementation in favour of `PartialOrd`
- `once_cell` is no longer needed and has been replaced by `lazy_static`

## [v0.2.1] - 06/03/2021

### Added

- Comparison Operators
- Negative Operator
- Integration Tests

### Changed

- Made Methods Public

## [v0.2.0] - 27/02/2021

### Added

- Binary Operators for Addition, Subtraction, Multiplication and Division

### Removed

- Functions for Addition, Subtraction, Multiplication and Division

## [v0.1.0] - 19/12/2020

### Added

- Decimal struct
- Function implementations for Decimal
- Static functions


[v0.3.0]: https://github.com/Redfire75369/break-infinity.rs/compare/f1fc9abefc158fff513dc9c5796947824e7abea2..master
[v0.2.1]: https://github.com/Redfire75369/break-infinity.rs/compare/087957eea4b35f8c6cfd3d6aba07c999e52a3dca..f1fc9abefc158fff513dc9c5796947824e7abea2
[v0.2.0]: https://github.com/Redfire75369/break-infinity.rs/compare/05b2c2e215296715d75fee23a018a3904e0808e4..087957eea4b35f8c6cfd3d6aba07c999e52a3dca
[v0.1.0]: https://github.com/Redfire75369/break-infinity.rs/tree/05b2c2e215296715d75fee23a018a3904e0808e4
