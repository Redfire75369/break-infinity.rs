# break-infinity.rs
A port of [Patashu's break_infinity.js](https://github.com/Patashu/break_infinity.js) to Rust.
It has the `Decimal` struct which is able to reach a maximum value of 1e9e15 instead of `f64`'s maximum of 1.79e308.

## Installation
You can install this package via Cargo by adding these lines to your `Cargo.toml`.
```toml
[dependencies]
break_infinity = "0.3.0"
# ...
```

## Features
- `full-range`: Increases maximum value to 1e1.79e308. Reduced accuracy above 1e9e15
- `serde`: Enables Serialization and Deserialization with Serde

## Usage
This library allows simple creation of `Decimal`'s through many different methods.
```rust
use break_infinity::Decimal;

fn main() {
	let x = Decimal::new(123.4567);
	let y = Decimal::from_str("123456.7e-3").expect("Failed to parse Decimal");
}
```

Methods that return a `Decimal` can also be chained
```rust
use break_infinity as bi;

fn main() {
	let short = ((x / &y + &z) * &Decimal::new(9.0)).floor();
	let long = x.ceil()
		.exp()
		.log10();
}
```
For a complete list of functions and methods, refer to the [docs](https://docs.rs/break_infinity).

## Acknowledgements
Patashu and Razenpok for creating the original `break_infinity.js` that this is based off of.
