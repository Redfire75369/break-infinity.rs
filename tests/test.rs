use break_infinity as bi;
use std::f64::{INFINITY, NAN, NEG_INFINITY};

#[test]
fn decimal() {
	assert_eq!(bi::new(0.0).to_string(), "0");
	assert_eq!(bi::new(NAN).to_string(), "NaN");
	assert_eq!(bi::new(INFINITY).to_string(), "Infinity");
	assert_eq!(bi::new(NEG_INFINITY).to_string(), "-Infinity");

	assert_eq!(bi::new(100.0).to_string(), "100");
	assert_eq!(bi::new(1e12).to_string(), "1000000000000");
	assert_eq!(bi::new(1e308).to_string(), "1e+308");
}

#[test]
fn ops() {
	let a = bi::from_mantissa_exponent_no_normalize(3.224, 54.0);
	let b = bi::from_mantissa_exponent_no_normalize(1.24, 53.0);
	let c = bi::from_mantissa_exponent_no_normalize(3.1, 52.0);

	assert_eq!(a + &b, bi::from_mantissa_exponent_no_normalize(3.348, 54.0));
	assert_eq!(a - &b, bi::from_mantissa_exponent_no_normalize(3.1, 54.0));
	assert_eq!(a * &b, bi::from_mantissa_exponent_no_normalize(3.9977600000000004, 107.0));
	assert_eq!(a / &b, bi::from_mantissa_exponent_no_normalize(2.6, 1.0));

	assert_eq!(a + &c, bi::from_mantissa_exponent_no_normalize(3.255, 54.0));
	assert_eq!(a - &c, bi::from_mantissa_exponent_no_normalize(3.193, 54.0));
	assert_eq!(a * &c, bi::from_mantissa_exponent_no_normalize(9.9944, 106.0));
	assert_eq!(a / &c, bi::from_mantissa_exponent_no_normalize(1.04, 2.0));

	assert_eq!(b + &c, bi::from_mantissa_exponent_no_normalize(1.55, 53.0));
	assert_eq!(b - &c, bi::from_mantissa_exponent_no_normalize(9.3, 52.0));
	assert_eq!(b * &c, bi::from_mantissa_exponent_no_normalize(3.844, 105.0));
	assert_eq!(b / &c, bi::from_mantissa_exponent_no_normalize(3.9999999999999996, 0.0));
}

#[test]
fn cmp() {
	let a = bi::from_mantissa_exponent_no_normalize(3.224, 54.0);
	let b = bi::from_mantissa_exponent_no_normalize(1.24, 53.0);
	let c = bi::from_mantissa_exponent_no_normalize(3.1, 52.0);
	let d = bi::from_mantissa_exponent_no_normalize(3.224, 54.0);

	assert_eq!(a == b, false);
	assert_eq!(a == d, true);
	assert_eq!(b == d, false);

	assert_eq!(a >= b, true);
	assert_eq!(a >= d, true);
	assert_eq!(b >= d, false);

	assert_eq!(a > b, true);
	assert_eq!(a > d, false);
	assert_eq!(b > d, false);

	assert_eq!(a <= b, false);
	assert_eq!(a <= d, true);
	assert_eq!(b <= d, true);

	assert_eq!(a < b, false);
	assert_eq!(a < d, false);
	assert_eq!(b < d, true);

	assert_eq!(a.max(b), b);
	assert_eq!(a.max(c), c);
	assert_eq!(b.max(c), c);

	assert_eq!(a.min(b), a);
	assert_eq!(a.min(c), a);
	assert_eq!(b.min(c), b);

	assert_eq!(a.clamp(c, b), c);
	assert_eq!(b.clamp(c, a), c);
	assert_eq!(c.clamp(b, b), b);
}

#[test]
fn neg_abs() {
	assert_eq!(-bi::new(456.7), bi::from_mantissa_exponent_no_normalize(-4.567, 2.0));
	assert_eq!(-bi::new(1.23e48), bi::from_mantissa_exponent_no_normalize(-1.23, 48.0));

	assert_eq!(bi::new(-456.7).abs(), bi::from_mantissa_exponent_no_normalize(4.567, 2.0));
	assert_eq!(bi::new(-1.23e48).abs(), bi::from_mantissa_exponent_no_normalize(1.23, 48.0));
}
