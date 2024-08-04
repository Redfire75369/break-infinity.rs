use std::borrow::Cow;
use std::cmp::Ordering::{self, *};
use std::f64::consts::{E, LN_10, LOG2_10, PI};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::num::ParseFloatError;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use std::str::FromStr;

mod macros;

#[cfg(test)]
mod test;

pub const MAX_SAFE_INTEGER: f64 = 9007199254740991.0;

pub const MAX_SIGNIFICANT_DIGITS: u32 = 17;

#[cfg(not(feature = "full-range"))]
pub const EXP_LIMIT: f64 = 9e15;

#[cfg(feature = "full-range")]
pub const EXP_LIMIT: f64 = 1.79e308;

/// Tolerance which is used for f64 conversion to compensate for floating-point error.
pub const ROUND_TOLERANCE: f64 = f64::EPSILON;

/// The smallest exponent that can appear in an f64, though not all mantissas are valid here.
pub const NUMBER_EXP_MIN: i32 = -324;

/// The largest exponent that can appear in an f64, though not all mantissas are valid here.
pub const NUMBER_EXP_MAX: i32 = 308;

/// The length of the cache used for powers of 10.
pub const LENGTH: usize = (NUMBER_EXP_MAX - NUMBER_EXP_MIN + 1) as usize;

// It might be worth turning this into a macro and embedding the cache right into the library,
// making it a lot faster while increasing the library size.
lazy_static::lazy_static! {
	pub static ref CACHED_POWERS : [f64; LENGTH] = {
		let mut arr = [0.0; LENGTH];
		for (i, item) in &mut arr.iter_mut().enumerate() {
			*item = 10.0_f64.powi((i as i32) + NUMBER_EXP_MIN);
		}
		arr
	};
}

/// Pads the given string with the fill string to the given max length.
pub fn pad_end(string: String, max_length: u32, fill_string: &'static str) -> String {
	if f32::is_nan(max_length as f32) || f32::is_infinite(max_length as f32) {
		return string;
	}

	let length = string.chars().count() as u32;
	if length >= max_length {
		return string;
	}

	let mut filled = Cow::Borrowed(fill_string);
	if filled.is_empty() {
		filled = Cow::Borrowed(" ");
	}

	let fill_len = max_length - length;
	while filled.chars().count() < fill_len as usize {
		filled = Cow::Owned(format!("{}{}", filled, filled));
	}

	let truncated = if filled.chars().count() > fill_len as usize {
		Cow::from(&filled[0..(fill_len as usize)])
	} else {
		filled
	};

	format!("{}{}", string, truncated)
}

/// Formats the given number to the given number of significant digits.
pub fn to_fixed(num: f64, places: u32) -> String {
	format!("{:.*}", places as usize, num)
}

/// Formats the given number to the given number of significant digits and parses it back to a number.
pub fn to_fixed_num(num: f64, places: u32) -> f64 {
	to_fixed(num, places).parse::<f64>().unwrap()
}

/// Returns the power of 10 with the given exponent from the cache.
fn power_of_10(power: i32) -> f64 {
	CACHED_POWERS[(power - NUMBER_EXP_MIN) as usize]
}

/// Creates a new instance of Decimal with the given mantissa and exponent without normalizing them.
pub fn from_mantissa_exponent_no_normalize(mantissa: f64, exponent: f64) -> Decimal {
	Decimal { mantissa, exponent }
}

/// Creates a new instance of Decimal with the given mantissa and exponent with normalizing them.
pub fn from_mantissa_exponent(mantissa: f64, exponent: f64) -> Decimal {
	if !f64::is_finite(mantissa) || !f64::is_finite(exponent) {
		return Decimal {
			mantissa: f64::NAN,
			exponent: f64::NAN,
		};
	}
	let decimal = from_mantissa_exponent_no_normalize(mantissa, exponent);
	decimal.normalize()
}

/// A struct representing a decimal number, which can reach a maximum of 1e1.79e308 instead of `f64`'s maximum of 1.79e308.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Decimal {
	mantissa: f64,
	exponent: f64,
}

impl Display for Decimal {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		if f64::is_nan(self.mantissa) || f64::is_nan(self.exponent) {
			return write!(f, "NaN");
		} else if self.exponent >= EXP_LIMIT {
			return if self.mantissa > 0.0 {
				write!(f, "Infinity")
			} else {
				write!(f, "-Infinity")
			};
		} else if self.exponent <= -EXP_LIMIT || self.mantissa == 0.0 {
			return write!(f, "0");
		} else if self.exponent < 21.0 && self.exponent > -7.0 {
			return if let Some(places) = f.precision() {
				write!(f, "{:.*}", places, self.to_number().to_string())
			} else {
				write!(f, "{}", self.to_number())
			};
		}

		let form = if let Some(places) = f.precision() {
			self.to_exponential(places as u32)
		} else {
			self.to_exponential(16)
		};

		write!(f, "{}", form)
	}
}

impl Add<Decimal> for Decimal {
	type Output = Decimal;

	fn add(self, decimal: Decimal) -> Decimal {
		// Figure out which is bigger, shrink the mantissa of the smaller
		// by the difference in exponents, add mantissas, normalize and return
		// TODO: Optimizations and simplification may be possible, see https://github.com/Patashu/break_infinity.js/issues/8
		if self.mantissa == 0.0 {
			return decimal;
		}

		if decimal.mantissa == 0.0 {
			return self;
		}

		let bigger_decimal;
		let smaller_decimal;

		if self.exponent >= decimal.exponent {
			bigger_decimal = self;
			smaller_decimal = decimal;
		} else {
			bigger_decimal = decimal;
			smaller_decimal = self;
		}

		if bigger_decimal.exponent - smaller_decimal.exponent > MAX_SIGNIFICANT_DIGITS as f64 {
			return bigger_decimal;
		}

		from_mantissa_exponent(
			(1e14 * bigger_decimal.mantissa)
				+ 1e14
					* smaller_decimal.mantissa
					* power_of_10((smaller_decimal.exponent - bigger_decimal.exponent) as i32),
			bigger_decimal.exponent - 14.0,
		)
	}
}

impl Add<&Decimal> for Decimal {
	type Output = Decimal;

	fn add(self, decimal: &Decimal) -> Decimal {
		self + *decimal
	}
}

impl Add<Decimal> for &Decimal {
	type Output = Decimal;

	fn add(self, decimal: Decimal) -> Decimal {
		*self + decimal
	}
}

impl Add<&Decimal> for &Decimal {
	type Output = Decimal;

	fn add(self, decimal: &Decimal) -> Decimal {
		*self + *decimal
	}
}

impl AddAssign<&Decimal> for Decimal {
	fn add_assign(&mut self, rhs: &Decimal) {
		*self = *self + rhs;
	}
}

impl AddAssign<Decimal> for Decimal {
	fn add_assign(&mut self, rhs: Decimal) {
		*self = *self + rhs;
	}
}

impl Sub<&Decimal> for &Decimal {
	type Output = Decimal;

	fn sub(self, decimal: &Decimal) -> Decimal {
		*self - *decimal
	}
}

impl Sub<&Decimal> for Decimal {
	type Output = Decimal;

	fn sub(self, decimal: &Decimal) -> Decimal {
		self - *decimal
	}
}

impl Sub<Decimal> for &Decimal {
	type Output = Decimal;

	fn sub(self, decimal: Decimal) -> Decimal {
		*self - decimal
	}
}

impl Sub<Decimal> for Decimal {
	type Output = Decimal;

	#[allow(clippy::suspicious_arithmetic_impl)]
	fn sub(self, decimal: Decimal) -> Decimal {
		self + decimal.neg()
	}
}

impl SubAssign<&Decimal> for Decimal {
	fn sub_assign(&mut self, rhs: &Decimal) {
		*self = *self - rhs;
	}
}

impl SubAssign<Decimal> for Decimal {
	fn sub_assign(&mut self, rhs: Decimal) {
		*self = *self - rhs;
	}
}

impl Mul<Decimal> for Decimal {
	type Output = Decimal;

	fn mul(self, decimal: Decimal) -> Decimal {
		from_mantissa_exponent(self.mantissa * decimal.mantissa, self.exponent + decimal.exponent)
	}
}

impl Mul<&Decimal> for Decimal {
	type Output = Decimal;

	fn mul(self, decimal: &Decimal) -> Decimal {
		self * *decimal
	}
}

impl Mul<Decimal> for &Decimal {
	type Output = Decimal;

	fn mul(self, decimal: Decimal) -> Decimal {
		*self * decimal
	}
}

impl Mul<&Decimal> for &Decimal {
	type Output = Decimal;

	fn mul(self, decimal: &Decimal) -> Decimal {
		*self * *decimal
	}
}

impl MulAssign<&Decimal> for Decimal {
	fn mul_assign(&mut self, rhs: &Decimal) {
		*self = *self * rhs;
	}
}

impl MulAssign<Decimal> for Decimal {
	fn mul_assign(&mut self, rhs: Decimal) {
		*self = *self * rhs;
	}
}

impl Div<Decimal> for Decimal {
	type Output = Decimal;

	#[allow(clippy::suspicious_arithmetic_impl)]
	fn div(self, decimal: Decimal) -> Decimal {
		self * decimal.recip()
	}
}

impl Div<&Decimal> for Decimal {
	type Output = Decimal;

	fn div(self, decimal: &Decimal) -> Decimal {
		self / *decimal
	}
}

impl Div<Decimal> for &Decimal {
	type Output = Decimal;

	fn div(self, decimal: Decimal) -> Decimal {
		*self / decimal
	}
}

impl Div<&Decimal> for &Decimal {
	type Output = Decimal;

	fn div(self, decimal: &Decimal) -> Decimal {
		*self / *decimal
	}
}

impl DivAssign<&Decimal> for Decimal {
	fn div_assign(&mut self, rhs: &Decimal) {
		*self = *self / rhs;
	}
}

impl DivAssign<Decimal> for Decimal {
	fn div_assign(&mut self, rhs: Decimal) {
		*self = *self / rhs;
	}
}

impl Neg for &Decimal {
	type Output = Decimal;

	fn neg(self) -> Decimal {
		from_mantissa_exponent_no_normalize(-self.mantissa, self.exponent)
	}
}

impl Neg for Decimal {
	type Output = Decimal;

	fn neg(self) -> Decimal {
		let decimal = &self.clone();
		from_mantissa_exponent_no_normalize(-decimal.mantissa, decimal.exponent)
	}
}

impl PartialOrd for Decimal {
	fn partial_cmp(&self, decimal: &Self) -> Option<Ordering> {
		/*
		From smallest to largest:
		-Infinity
		-3e100
		-1e100
		-3e99
		-1e99
		-3e0
		-1e0
		-3e-99
		-1e-99
		-3e-100
		-1e-100
		0
		1e-100
		3e-100
		1e-99
		3e-99
		1e0
		3e0
		1e99
		3e99
		1e100
		3e100
		Infinity
		*/

		if f64::is_nan(self.mantissa)
			|| f64::is_nan(self.exponent)
			|| f64::is_nan(decimal.mantissa)
			|| f64::is_nan(decimal.exponent)
		{
			None
		} else if (f64::is_infinite(self.mantissa) && self.mantissa.is_sign_negative())
			|| (f64::is_infinite(decimal.mantissa) && decimal.mantissa.is_sign_positive())
		{
			Some(Less)
		} else if (f64::is_infinite(self.mantissa) && self.mantissa.is_sign_negative())
			|| (f64::is_infinite(decimal.mantissa) && decimal.mantissa.is_sign_positive())
		{
			Some(Greater)
		} else if self.mantissa == 0.0 {
			if decimal.mantissa == 0.0 {
				Some(Equal)
			} else if decimal.mantissa < 0.0 {
				Some(Greater)
			} else {
				Some(Less)
			}
		} else if decimal.mantissa == 0.0 {
			if self.mantissa < 0.0 {
				Some(Less)
			} else {
				Some(Greater)
			}
		} else if self.mantissa > 0.0 {
			if self.exponent > decimal.exponent || decimal.mantissa < 0.0 {
				Some(Greater)
			} else if self.exponent < decimal.exponent {
				Some(Less)
			} else if self.mantissa > decimal.mantissa {
				Some(Greater)
			} else if self.mantissa < decimal.mantissa {
				Some(Less)
			} else {
				Some(Equal)
			}
		} else if self.exponent > decimal.exponent || decimal.mantissa > 0.0 {
			Some(Less)
		} else if self.mantissa > decimal.mantissa || self.exponent < decimal.exponent {
			Some(Greater)
		} else if self.mantissa < decimal.mantissa {
			Some(Less)
		} else {
			Some(Equal)
		}
	}
}

impl PartialEq<Decimal> for Decimal {
	fn eq(&self, decimal: &Decimal) -> bool {
		self.mantissa == decimal.mantissa && self.exponent == decimal.exponent
	}
}

impl Eq for Decimal {}

impl FromStr for Decimal {
	type Err = ParseFloatError;

	fn from_str(string: &str) -> Result<Decimal, ParseFloatError> {
		if let Some((mantissa, exponent)) = string.split_once('e') {
			let decimal = Decimal {
				mantissa: mantissa.parse()?,
				exponent: exponent.parse()?,
			};

			Ok(decimal.normalize())
		} else if string == "NaN" {
			Ok(Decimal::NAN)
		} else {
			string.parse::<f64>().map(Decimal::new)
		}
	}
}

impl Default for Decimal {
	fn default() -> Self {
		Decimal::ZERO
	}
}

// This allows converting virtually any number to a Decimal.
impl_from!(i8);
impl_from!(i16);
impl_from!(i32);
impl_from!(i64);
impl_from!(i128);
impl_from!(isize);
impl_from!(u8);
impl_from!(u16);
impl_from!(u32);
impl_from!(u64);
impl_from!(u128);
impl_from!(usize);
impl_from!(f32);
impl_from!(f64);

impl Decimal {
	pub const MIN_VALUE: Decimal = Decimal {
		mantissa: 1.0,
		exponent: -EXP_LIMIT,
	};
	pub const MAX_VALUE: Decimal = Decimal {
		mantissa: 1.0,
		exponent: EXP_LIMIT,
	};

	pub const ZERO: Decimal = Decimal {
		mantissa: 0.0,
		exponent: 0.0,
	};
	pub const ONE: Decimal = Decimal {
		mantissa: 1.0,
		exponent: 0.0,
	};
	pub const NEGATIVE_ONE: Decimal = Decimal {
		mantissa: -1.0,
		exponent: 0.0,
	};
	pub const E: Decimal = Decimal {
		mantissa: E,
		exponent: 0.0,
	};
	pub const NAN: Decimal = Decimal {
		mantissa: f64::NAN,
		exponent: 0.0,
	};

	/// Creates a new instance of Decimal with the given value.
	pub fn new(value: f64) -> Decimal {
		// SAFETY: Handle Infinity and NaN in a somewhat meaningful way.
		if f64::is_nan(value) {
			return Decimal {
				mantissa: f64::NAN,
				exponent: f64::NAN,
			};
		} else if value == 0.0 {
			return Decimal {
				mantissa: 0.0,
				exponent: 0.0,
			};
		} else if f64::is_infinite(value) && f64::is_sign_positive(value) {
			return Decimal {
				mantissa: 1.0,
				exponent: EXP_LIMIT,
			};
		} else if f64::is_infinite(value) && f64::is_sign_negative(value) {
			return Decimal {
				mantissa: -1.0,
				exponent: EXP_LIMIT,
			};
		}

		let e = value.abs().log10().floor();
		let m = if (e - NUMBER_EXP_MIN as f64).abs() < f64::EPSILON {
			value * 10.0 / format!("1e{}", NUMBER_EXP_MIN + 1).parse::<f64>().unwrap()
		} else {
			let power_10 = power_of_10(e as i32);
			// This essentially rounds the mantissa for very high numbers.
			((value / power_10) * 1000000000000000.0).round() / 1000000000000000.0
		};
		let decimal = Decimal {
			mantissa: m,
			exponent: e,
		};
		decimal.normalize()
	}

	pub fn pow10(power: f64) -> Decimal {
		if power.fract() == 0.0 {
			from_mantissa_exponent_no_normalize(1.0, power)
		} else {
			from_mantissa_exponent(10.0_f64.powf(power.fract()), power.trunc())
		}
	}

	/// Normalizes the mantissa when it is too denormalized.
	fn normalize(&self) -> Decimal {
		if self.mantissa >= 1.0 && self.mantissa < 10.0 {
			return *self;
		} else if self.mantissa == 0.0 {
			return Decimal {
				mantissa: 0.0,
				exponent: 0.0,
			};
		}

		let temp_exponent = self.mantissa.abs().log10().floor();
		Decimal {
			mantissa: if (temp_exponent as i32) == NUMBER_EXP_MIN {
				self.mantissa * 10.0 / 1e-323
			} else {
				self.mantissa / power_of_10(temp_exponent as i32)
			},
			exponent: self.exponent + temp_exponent,
		}
	}

	/// Converts the Decimal to an f64.
	pub fn to_number(&self) -> f64 {
		//  Problem: new(116.0).to_number() returns 115.99999999999999.
		//  TODO: How to fix in general case? It's clear that if to_number() is
		//	VERY close to an integer, we want exactly the integer.
		//	But it's not clear how to specifically write that.
		//	So I'll just settle with 'exponent >= 0 and difference between rounded
		//	and not rounded < 1e-9' as a quick fix.
		//  var result = self.mantissa * 10.0_f64.powf(self.exponent);
		if !f64::is_finite(self.exponent) {
			return f64::NAN;
		}

		if self.exponent > NUMBER_EXP_MAX as f64 {
			return if self.mantissa > 0.0 {
				f64::INFINITY
			} else {
				f64::NEG_INFINITY
			};
		}

		if self.exponent < NUMBER_EXP_MIN as f64 {
			return 0.0;
		}

		if (self.exponent - NUMBER_EXP_MIN as f64).abs() < f64::EPSILON {
			return if self.mantissa > 0.0 { 5e-324 } else { -5e-324 };
		}

		let result: f64 = self.mantissa * power_of_10(self.exponent as i32);

		if !f64::is_finite(result) || self.exponent < 0.0 {
			return result;
		}

		let result_rounded = result.round();

		if (result_rounded - result).abs() < ROUND_TOLERANCE {
			return result_rounded;
		}

		result
	}

	#[inline(always)]
	fn as_non_finite_string(&self) -> Option<String> {
		if f64::is_nan(self.mantissa) || f64::is_nan(self.exponent) {
			Some(String::from("NaN"))
		} else if self.exponent >= EXP_LIMIT {
			if self.mantissa > 0.0 {
				Some(String::from("Infinity"))
			} else {
				Some(String::from("-Infinity"))
			}
		} else {
			None
		}
	}

	/// Converts the Decimal into a string with the scientific notation.
	pub fn to_exponential(&self, mut places: u32) -> String {
		if let Some(string) = self.as_non_finite_string() {
			return string;
		}

		let tmp = pad_end(String::from("."), places + 1, "0");
		// 1) exponent is < 308 and > -324: use basic to_fixed
		// 2) everything else: we have to do it ourselves!
		if self.exponent <= -EXP_LIMIT || self.mantissa == 0.0 {
			let str = if places > 0 { &tmp } else { "" };
			return format!("0{}e+0", str);
		} else if !f32::is_finite(places as f32) {
			places = MAX_SIGNIFICANT_DIGITS;
		}

		let len = places + 1;
		let num_digits = self.mantissa.abs().log10().max(1.0) as u32;
		let rounded = (self.mantissa * 10.0_f64.powi(len as i32 - num_digits as i32)).round()
			* 10.0_f64.powi(num_digits as i32 - len as i32);

		let mantissa = to_fixed(rounded, 0_u32.max(len - num_digits));
		let sign = if self.exponent >= 0.0 { "+" } else { "" };
		format!("{}e{}{}", mantissa, sign, self.exponent)
	}

	/// Converts the Decimal into a string with the fixed notation.
	pub fn to_fixed(&self, places: u32) -> String {
		if let Some(string) = self.as_non_finite_string() {
			return string;
		}

		let tmp = pad_end(String::from("."), places + 1, "0");
		if self.exponent <= -EXP_LIMIT || self.mantissa == 0.0 {
			// Two Cases:
			// 1) exponent is 17 or greater: just print out mantissa with the appropriate number of zeroes after it
			// 2) exponent is 16 or less: use basic to_fixed
			let str = if places > 0 { &tmp } else { "" };
			return format!("0{}", str);
		} else if self.exponent >= MAX_SIGNIFICANT_DIGITS as f64 {
			let str = pad_end(
				self.mantissa.to_string().replace('.', ""),
				(self.exponent + 1.0) as u32,
				"0",
			);
			let decimals = if places > 0 { &tmp } else { "" };
			return format!("{}{}", str, decimals);
		}

		to_fixed(self.to_number(), places)
	}

	/// Converts the Decimal into a string with the scientific notation if the exponent is greater than the precision.
	pub fn to_precision(&self, places: u32) -> String {
		if self.exponent <= -7.0 {
			return self.to_exponential(places - 1);
		}

		if (places as f64) > self.exponent {
			return self.to_fixed((places as f64 - self.exponent - 1.0) as u32);
		}

		self.to_exponential(places - 1)
	}

	/// Returns the mantissa with the specified precision.
	pub fn mantissa_with_decimal_places(&self, places: u32) -> f64 {
		// https://stackoverflow.com/a/37425022
		if f64::is_nan(self.mantissa) || f64::is_nan(self.exponent) {
			return f64::NAN;
		} else if self.mantissa == 0.0 {
			return 0.0;
		}

		let len = places + 1;
		let num_digits = self.mantissa.abs().log10().ceil() as u32;
		let rounded = (self.mantissa * 10.0_f64.powi(len as i32 - num_digits as i32)).round()
			* 10.0_f64.powi(num_digits as i32 - len as i32);
		to_fixed_num(rounded, 0.max(len - num_digits))
	}

	/// Returns the absolute value of the Decimal.
	pub fn abs(&self) -> Decimal {
		from_mantissa_exponent_no_normalize(self.mantissa.abs(), self.exponent)
	}

	/// Returns the sign of the Decimal, according to [f64::signum].
	pub fn sign(&self) -> f64 {
		self.mantissa.signum()
	}

	/// Rounds the Decimal, if the exponent isn't greater than the maximum significant digits.
	pub fn round(&self) -> Decimal {
		if self.exponent < -1.0 {
			Decimal::ZERO
		} else if self.exponent < MAX_SIGNIFICANT_DIGITS as f64 {
			Decimal::new(self.to_number().round())
		} else {
			*self
		}
	}

	/// Truncates the Decimal, if the exponent isn't greater than the maximum significant digits.
	pub fn trunc(&self) -> Decimal {
		if self.exponent < 0.0 {
			Decimal::ZERO
		} else if self.exponent < MAX_SIGNIFICANT_DIGITS as f64 {
			Decimal::new(self.to_number().trunc())
		} else {
			*self
		}
	}

	/// Floors the Decimal, if the exponent isn't greater than the maximum significant digits.
	pub fn floor(&self) -> Decimal {
		if self.exponent < -1.0 {
			if self.sign() > 0.0 {
				Decimal::ZERO
			} else {
				Decimal::NEGATIVE_ONE
			}
		} else if self.exponent < MAX_SIGNIFICANT_DIGITS as f64 {
			Decimal::new(self.to_number().floor())
		} else {
			*self
		}
	}

	/// Rounds the Decimal to its ceiling, if the exponent isn't greater than the maximum significant digits.
	pub fn ceil(&self) -> Decimal {
		if self.exponent < -1.0 {
			if self.mantissa == 0.0 || self.sign() < 0.0 {
				Decimal::ZERO
			} else {
				Decimal::ONE
			}
		} else if self.exponent < MAX_SIGNIFICANT_DIGITS as f64 {
			Decimal::new(self.to_number().ceil())
		} else {
			*self
		}
	}

	/// Returns the reciprocal of the Decimal.
	pub fn recip(&self) -> Decimal {
		from_mantissa_exponent(1.0 / self.mantissa, -self.exponent)
	}

	pub fn max(&self, other: &Decimal) -> Decimal {
		if self > other {
			*self
		} else {
			*other
		}
	}

	pub fn min(&self, other: &Decimal) -> Decimal {
		if self < other {
			*self
		} else {
			*other
		}
	}

	pub fn clamp(&self, min: &Decimal, max: &Decimal) -> Decimal {
		self.max(min).min(max)
	}

	pub fn cmp_tolerance(&self, decimal: &Decimal, tolerance: &Decimal) -> Option<Ordering> {
		if self.eq_tolerance(decimal, tolerance) {
			Some(Equal)
		} else {
			self.partial_cmp(decimal)
		}
	}

	/// Tolerance is a relative tolerance, multiplied by the greater of the magnitudes of the two arguments.
	/// For example, if you put in 1e-9, then any number closer to the
	/// larger number than (larger number) * 1e-9 will be considered equal.
	pub fn eq_tolerance(&self, decimal: &Decimal, tolerance: &Decimal) -> bool {
		// return abs(a-b) <= tolerance * max(abs(a), abs(b))
		(self - decimal).abs().le(&self.abs().max(&(decimal.abs() * tolerance)))
	}

	pub fn neq_tolerance(&self, decimal: &Decimal, tolerance: &Decimal) -> bool {
		!self.eq_tolerance(decimal, tolerance)
	}

	pub fn lt_tolerance(&self, decimal: &Decimal, tolerance: &Decimal) -> bool {
		!self.eq_tolerance(decimal, tolerance) && self.lt(decimal)
	}
	pub fn le_tolerance(&self, decimal: &Decimal, tolerance: &Decimal) -> bool {
		self.eq_tolerance(decimal, tolerance) || self.lt(decimal)
	}

	pub fn gt_tolerance(&self, decimal: &Decimal, tolerance: &Decimal) -> bool {
		!self.eq_tolerance(decimal, tolerance) && self.gt(decimal)
	}
	pub fn ge_tolerance(&self, decimal: &Decimal, tolerance: &Decimal) -> bool {
		self.eq_tolerance(decimal, tolerance) || self.gt(decimal)
	}

	pub fn log10(&self) -> f64 {
		self.exponent + self.mantissa.log10()
	}

	pub fn abs_log10(&self) -> f64 {
		self.exponent + self.mantissa.abs().log10()
	}

	pub fn p_log10(&self) -> f64 {
		if self.mantissa <= 0.0 || self.exponent < 0.0 {
			0.0
		} else {
			self.log10()
		}
	}

	pub fn log(&self, base: f64) -> f64 {
		// UN-SAFETY: Most incremental game cases are log(number := 1 or greater, base := 2 or greater).
		// We assume this to be true and thus only need to return a number, not a Decimal,
		LN_10 / base.ln() * self.log10()
	}
	pub fn logarithm(&self, base: f64) -> f64 {
		self.log(base)
	}

	pub fn log2(&self) -> f64 {
		LOG2_10 * self.log10()
	}

	pub fn ln(&self) -> f64 {
		LN_10 * self.log10()
	}

	/// Raises the Decimal to the power of the given Decimal.
	pub fn pow(&self, decimal: &Decimal) -> Decimal {
		if self.mantissa == 0.0 {
			return *self;
		}

		//  UN-SAFETY: Accuracy not guaranteed beyond ~9-11 decimal places.
		//  TODO: Decimal.pow(new Decimal(0.5), 0); or Decimal.pow(new Decimal(1), -1);
		//	makes an exponent of -0! Is a negative zero ever a problem?

		//  TODO: Fast track seems about neutral for performance.
		//	It might become faster if an integer pow is implemented,
		//	or it might not be worth doing (see https://github.com/Patashu/break_infinity.js/issues/4 )
		//  Fast track: If (this.e*value) is an integer and mantissa ^ value
		//  fits in a Number, we can do a very fast method.

		// Fast track: If (this.e*value) is an integer and mantissa^value
		// fits in a Number, we can do a very fast method.
		let number = decimal.to_number();
		let temp = self.exponent * number;

		let mut new_mantissa;
		if temp < MAX_SAFE_INTEGER {
			// Same speed and usually more accurate.
			new_mantissa = self.mantissa.powf(number);

			if f64::is_finite(new_mantissa) && new_mantissa != 0.0 {
				return from_mantissa_exponent(new_mantissa, temp);
			}
		}

		let new_exponent = temp.trunc();
		let residue = temp - new_exponent;
		new_mantissa = 10.0_f64.powf(number * self.mantissa.log10() + residue);

		if f64::is_finite(new_mantissa) && new_mantissa != 0.0 {
			//  return Decimal.exp(value*this.ln());
			//  UN-SAFETY: This should return NaN when mantissa is negative and value is non-integer.
			return from_mantissa_exponent(new_mantissa, new_exponent);
		}

		let result = Decimal::pow10(number * self.abs_log10());

		if self.sign() == -1.0 {
			match (number % 2.0).abs() {
				0.0 => result,
				1.0 => result.neg(),
				_ => Decimal::NAN,
			}
		} else {
			result
		}
	}

	pub fn pow_base(&self, decimal: &Decimal) -> Decimal {
		decimal.pow(self)
	}

	pub fn factorial(&self) -> Decimal {
		//  Using Stirling's Approximation.
		//  https://en.wikipedia.org/wiki/Stirling%27s_approximation#Versions_suitable_for_calculators
		let n = self.to_number() + 1.0;
		Decimal::new(n / E * (n * f64::sinh(1.0 / n) + 1.0 / (810.0 * n.powi(6)))).pow(&Decimal::new(n))
			* Decimal::new(f64::sqrt(2.0 * PI / n))
	}

	pub fn exp(&self) -> Decimal {
		// Fast track: if -706 < this < 709, we can use regular exp.
		let number = self.to_number();
		if -706.0 < number && number < 709.0 {
			return Decimal::new(f64::exp(number));
		}
		Decimal::E.pow(self)
	}

	pub fn sqr(&self) -> Decimal {
		from_mantissa_exponent(self.mantissa.powi(2), self.exponent * 2.0)
	}

	pub fn sqrt(&self) -> Decimal {
		if self.mantissa < 0.0 {
			return Decimal::new(f64::NAN);
		} else if self.exponent % 2.0 != 0.0 {
			// Mod of a negative number is negative, so != means '1 or -1'
			return from_mantissa_exponent(
				f64::sqrt(self.mantissa) * 3.16227766016838,
				(self.exponent / 2.0).floor(),
			);
		}
		from_mantissa_exponent(f64::sqrt(self.mantissa), (self.exponent / 2.0).floor())
	}

	pub fn cube(&self) -> Decimal {
		from_mantissa_exponent(self.mantissa.powi(3), self.exponent * 3.0)
	}

	pub fn cbrt(&self) -> Decimal {
		let mut sign = 1;
		let mut mantissa = self.mantissa;

		if mantissa < 0.0 {
			sign = -1;
			mantissa = -mantissa;
		}

		let new_mantissa = sign as f64 * mantissa.powf(1.0 / 3.0);
		let remainder = (self.exponent % 3.0) as i32;

		if remainder == 1 || remainder == -2 {
			return from_mantissa_exponent(new_mantissa * 2.154_434_690_031_884, (self.exponent / 3.0).floor());
		}

		if remainder != 0 {
			// remainder != 0 at this point means 'remainder == 2 || remainder == -1'
			return from_mantissa_exponent(new_mantissa * 4.641_588_833_612_779, (self.exponent / 3.0).floor());
		}

		from_mantissa_exponent(new_mantissa, (self.exponent / 3.0).floor())
	}

	// Some hyperbolic trigonometry functions that happen to be easy
	pub fn sinh(&self) -> Decimal {
		(self.exp() - self.neg().exp()) / Decimal::new(2.0)
	}
	pub fn cosh(&self) -> Decimal {
		(self.exp() + self.neg().exp()) / Decimal::new(2.0)
	}
	pub fn tanh(&self) -> Decimal {
		self.sinh() / self.cosh()
	}

	pub fn asinh(&self) -> f64 {
		(self + (self.sqr() + Decimal::new(1.0)).sqrt()).ln()
	}
	pub fn acosh(&self) -> f64 {
		(self + (self.sqr() - Decimal::new(1.0)).sqrt()).ln()
	}
	pub fn atanh(&self) -> f64 {
		if self.abs().ge(&Decimal::new(1.0)) {
			return f64::NAN;
		}

		((Decimal::new(1.0) + self) / (Decimal::new(1.0) - self)).ln() / 2.0
	}

	/// Returns the number of decimal places in the number.
	pub fn dp(&self) -> Option<i32> {
		if !f64::is_finite(self.mantissa) {
			return None;
		} else if self.exponent >= MAX_SIGNIFICANT_DIGITS as f64 {
			return Some(0);
		}

		let mantissa = self.mantissa;
		let mut places = -self.exponent as i32;
		let mut e = 1.0;

		while (mantissa * e).round().abs() / e - mantissa > ROUND_TOLERANCE {
			e *= 10.0;
			places += 1;
		}

		Some(places.max(0))
	}

	/// Joke function from Realm Grinder
	pub fn ascension_penalty(&self, ascensions: f64) -> Decimal {
		if ascensions == 0.0 {
			return *self;
		}

		self.pow(&Decimal::pow10(-ascensions))
	}

	/// Joke function from Cookie Clicker. It's 'egg'
	pub fn egg(&self) -> Decimal {
		self + Decimal::new(9.0)
	}
}

/// If you're willing to spend 'resourcesAvailable' and want to buy something
/// with exponentially increasing cost each purchase (start at priceStart,
/// multiply by priceRatio, already own currentOwned), how much of it can you buy?
///
/// Adapted from Trimps source code.
pub fn afford_geometric_series(
	resources_available: &Decimal, price_start: &Decimal, price_ratio: &Decimal, current_owned: &Decimal,
) -> Decimal {
	let actual_start = price_start * price_ratio.pow(current_owned);
	Decimal::new(
		(resources_available / actual_start * (price_ratio - Decimal::new(1.0)) + Decimal::new(1.0)).log10()
			/ price_ratio.log10(),
	)
	.floor()
}

/// How much resource would it cost to buy (numItems) items if you already have currentOwned,
/// the initial price is priceStart and it multiplies by priceRatio each purchase?
pub fn sum_geometric_series(
	num_items: &Decimal, price_start: &Decimal, price_ratio: &Decimal, current_owned: &Decimal,
) -> Decimal {
	price_start * price_ratio.pow(current_owned) * (Decimal::new(1.0) - price_ratio.pow(num_items))
		/ (Decimal::new(1.0) - price_ratio)
}

/// If you're willing to spend 'resourcesAvailable' and want to buy something with additively
/// increasing cost each purchase (start at priceStart, add by priceAdd, already own currentOwned),
/// how much of it can you buy?
pub fn afford_arithmetic_series(
	resources_available: &Decimal, price_start: &Decimal, price_add: &Decimal, current_owned: &Decimal,
) -> Decimal {
	//  n = (-(a-d/2) + sqrt((a-d/2)^2+2dS))/d
	//  where `a` is actual_start, `d` is price_add and `S` is resources_available
	//  then floor it, and you're done!
	let actual_start = price_start + (current_owned * price_add);
	let b = actual_start - (price_add / Decimal::new(2.0));
	let b2 = b.pow(&Decimal::new(2.0));
	(b.neg() + ((b2 + ((price_add * resources_available) * Decimal::new(2.0))).sqrt() / price_add)).floor()
}

/// How much resource would it cost to buy (numItems) items if you already have currentOwned,
/// the initial price is priceStart and it adds priceAdd each purchase?
/// Adapted from http://www.mathwords.com/a/arithmetic_series.htm
pub fn sum_arithmetic_series(
	num_items: &Decimal, price_start: &Decimal, price_add: &Decimal, current_owned: &Decimal,
) -> Decimal {
	let actual_start = price_start + (current_owned * price_add); // (n/2)*(2*a+(n-1)*d)

	num_items / Decimal::new(2.0)
		* (actual_start * Decimal::new(2.0) + (num_items - Decimal::new(1.0)) + num_items - Decimal::new(1.0))
		* price_add
}

/// When comparing two purchases that cost (resource) and increase your resource/sec by (deltaRpS),
/// the lowest efficiency score is the better one to purchase.
///
/// From Frozen Cookies:
/// https://cookieclicker.wikia.com/wiki/Frozen_Cookies_(JavaScript_Add-on)#Efficiency.3F_What.27s_that.3F
pub fn efficiency_of_purchase(cost: &Decimal, current_rp_s: &Decimal, delta_rp_s: &Decimal) -> Decimal {
	cost / (current_rp_s + (cost / delta_rp_s))
}
