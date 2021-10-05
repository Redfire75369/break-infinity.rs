# break-infinity.rs
A port of [Patashu's break_infinity.js](https://github.com/Patashu/break_infinity.js) to Rust.
It has the `Decimal` struct which is able to reach a maximum value of 1e1.79e308 instead of `f64`'s maximum of 1.79e308.

## Installation
You can install this package via Cargo by adding these lines to your `Cargo.toml`.
```toml
[dependencies]
break_infinity="0.3.0"
# ...
```

## Usage
This library allows simple creation of `Decimal`'s through many different methods.
```rust
use break_infinity as bi;

fn main() {
	let x = bi::new(123.4567);
	let y = bi::from_string(&String::from("123456.7e-3"));
	let z = bi::from_decimal(x);
}
```
Methods that return a `Decimal` can also be chained
```rust
use break_infinity as bi;

fn main() {
	let short = ((x / &y + &z) * &bi::new(9.0)).floor();
	let long = x.ceil()
		.exp()
		.log10();
}
```
For a complete list of functions and methods, refer to the [docs](https://docs.rs/break_infinity).

## Acknowledgements
Patashu and Razenpok for creating the original `break_infinity.js` that this is based off of.
