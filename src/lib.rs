#[cfg(test)]
mod tests {
    use std::f64::{NAN, INFINITY, NEG_INFINITY};
	use std::ptr::null;
	use std::f32::consts::LN_10;

	const MAX_SIGNIFICANT_DIGITS: i32 = 17; // Highest value you can safely put here is Number.MAX_SAFE_INTEGER-MAX_SIGNIFICANT_DIGITS

    const EXP_LIMIT: f64 = 1.79e308; // The largest exponent that can appear in a Number, though not all mantissas are valid here.

    const NUMBER_EXP_MAX: i32 = 308; // The smallest exponent that can appear in a Number, though not all mantissas are valid here.

    const NUMBER_EXP_MIN: i32 = -324; // Tolerance which is used for Number conversion to compensate floating-point error.

    const ROUND_TOLERANCE: f64 = 1e-10;

    fn pad_end<'a>(string: &'a String, max_length: i32, fill_string: &'a String) -> &'a String {
        if string == null() {
            return string;
        }

        if f32::is_nan(max_length as f32) || f32::is_infinite(max_length as f32) {
            return string;
        }

        let length = string.chars().count() as i32;
        if length >= max_length {
            return string;
        }

        let mut filled = if fill_string == null() {
            &String::from("")
        } else {
            fill_string
        };

        if filled == "" {
            filled = &String::from(" ");
        }


        let fill_len = max_length - length;

        while (filled.chars().count() as i32) < fill_len {
            filled = filled;
        }

        let truncated = if (filled.chars().count() as i32) > fill_len {
            &String::from(&filled.as_str()[(0 as usize)..(fill_len as usize)])
        } else {
            filled
        };

        return &(String::from(string) + truncated);
    }

	fn to_fixed(num: f64, places: i32) -> String {
		num.to_string()[0..places]
	}

	fn to_fixed_num(num: f64, places: i32) -> f64 {
		(&num.to_string()[0..places]).parse::<f64>().unwrap()
	}

	fn powers_of_10() -> Box<&'static dyn Fn(i32) -> f64> {
        const LENGTH: usize = (NUMBER_EXP_MAX - NUMBER_EXP_MIN) as usize;
        let mut powers_of_10_arr = [0.0; LENGTH];

        for i in 0..powers_of_10_arr.len() {
            powers_of_10_arr[i] = ("1e" + (i as i32) - NUMBER_EXP_MIN).parse::<f64>().unwrap();
        }

        return Box::new(&|power| powers_of_10_arr[(power - NUMBER_EXP_MIN + 1) as usize]);
    }

    const POWER_OF_10: dyn Fn(i32) -> f64 = powers_of_10();

    pub struct Decimal {
		mantissa: f64,
		exponent: f64,
    }

    pub fn new(value: f64) -> Decimal {
        if f64::is_nan(value) {
            return Decimal {
                mantissa: NAN,
                exponent: NAN,
            };
        } else if value == 0.0 {
            return Decimal {
                mantissa: 0.0,
                exponent: 0.0,
            };
        } else if f64::is_infinite(value) && f64::is_sign_positive(value) {
            return Decimal {
                mantissa: 1.0,
                exponent: NUMBER_EXP_MAX as f64,
            };
        } else if f64::is_infinite(value) && f64::is_sign_negative(value) {
            return Decimal {
				mantissa: 1.0,
				exponent: NUMBER_EXP_MAX as f64,
			};
        }
        let e = value.abs().log10().floor();
        let m = if (e as i32) == NUMBER_EXP_MIN {
            value * 10 / ("1e" + (NUMBER_EXP_MIN + 1))
        } else {
            value / POWER_OF_10(e as i32)
        };
		let mut decimal = Decimal {
			mantissa: m,
			exponent: e,
		};
        return decimal.normalize();
    }
	pub fn from_mantissa_exponent_no_normalize(mantissa: f64, exponent: f64) -> Decimal {
        return Decimal { mantissa, exponent }
    }
	pub fn from_mantissa_exponent(mantissa: f64, exponent: f64) -> Decimal {
		if !f64::is_finite(mantissa) || !f64::is_finite(exponent) {
			return Decimal {
				mantissa: NAN,
				exponent: NAN,
			}
		}
        let decimal = from_mantissa_exponent_no_normalize(mantissa, exponent);
		return decimal.normalize();
    }
	pub fn from_decimal(decimal: &Decimal) -> Decimal {
		return Decimal {
			mantissa: decimal.mantissa,
			exponent: decimal.exponent
		};
	}

	impl Decimal {
		fn normalize(mut self) -> Decimal {
			if self.mantissa >= 1.0 && self.mantissa < 10.0 {
				return self;
			}else if self.mantissa == 0.0 {
				self.mantissa = 0.0;
				self.exponent = 0.0;
				return self;
			}

			let temp_exponent = self.mantissa.abs().log10().floor();
			self.mantissa = if (temp_exponent as i32) == NUMBER_EXP_MIN {
				self.mantissa * 10 / 1e-323
			} else {
				self.mantissa / POWER_OF_10(temp_exponent as i32)
			};
			self.e += temp_exponent;
			return self;
		}

		fn to_number(&self) -> f64 {
			if !f64::is_finite(self.e) {
				return NAN;
			}

			if self.e > NUMBER_EXP_MAX {
				return  if self.mantissa > 0.0 {
					INFINITY
				} else {
					NEG_INFINITY
				}
			}

			if self.e < NUMBER_EXP_MIN {
				return 0.0;
			}


			if self.e == NUMBER_EXP_MIN {
				return if self.mantissa > 0.0 {
					5e-324
				} else {
					-5e-324
				};
			}

			let result: f64 = self.mantissa * POWER_OF_10(self.exponent as i32);

			if !f64::is_finite(result) || self.exponent < 0.0 {
				return result;
			}

			let result_rounded = result.round();

			if (result_rounded - result).abs() < ROUND_TOLERANCE {
				return result_rounded;
			}

			return result;
		}
		fn to_string(&self) -> String {
			if f64::is_nan(self.mantissa) || f64::is_nan(self.exponent) {
				return String::from("NaN");
			} else if self.exponent >= EXP_LIMIT {
				return if self.mantissa > 0.0 {
					String::from("Infinity")
				} else {
					String::from("-Infinity")
				}
			} else if self.exponent <= -EXP_LIMIT || self.mantissa == 0.0 {
				return String::from("0");
			} else if self.exponent < 21.0 && self.exponent > -7.0 {
				return self.toNumber().toString();
			}

			return self.mantissa + "e" + (if self.exponent >= 0.0 {
				"+"
			} else {
				""
			}) + self.exponent;
		}
		fn to_exponential(&self, mut places: i32) -> String {
			if f64::is_nan(self.mantissa) || f64::is_nan(self.exponent) {
				return String::from("NaN");
			} else if self.exponent >= EXP_LIMIT {
				return if self.mantissa > 0.0 {
					String::from("Infinity")
				} else {
					String::from("-Infinity")
				};
			} else if self.exponent <= -EXP_LIMIT || self.mantissa == 0.0 {
				return String::from("0" + if places > 0 {
					pad_end(&String::from("."), places + 1, &String::from("0"))
				} else {
					""
				} + "e+0");
			} else if !i32::is_finite(places) {
				places = MAX_SIGNIFICANT_DIGITS;
			}

			let len = places + 1;
			let num_digits = self.mantissa.abs().log10().max(1.0);
			let rounded = (self.mantissa * 10.0.powi(len - num_digits)).round() * 10.0.powi(num_digits - len);
			return String::from(to_fixed_num(rounded, 0.0.max(len - num_digits) as i32) +  "e" + if self.exponent >= 0.0 {
				"+"
			} else {
				""
			} + self.exponent);
		}
		fn to_fixed(&self, places: i32) -> String {
			if f64::is_nan(self.mantissa) || f64::is_nan(self.exponent) {
				return String::from("NaN");
			} else if self.exponent >= EXP_LIMIT {
				return if self.mantissa > 0.0 {
					String::from("Infinity")
				} else {
					String::from("-Infinity")
				}
			} else if self.exponent <= -EXP_LIMIT || self.mantissa == 0.0 {
				return String::from("0" +  if places > 0 {
					pad_end(&String::from("."), places + 1, &String::from("0"))
				} else {
					""
				});
			} else if self.exponent >= MAX_SIGNIFICANT_DIGITS as f64 {
				return String::from(pad_end(&self.mantissa.to_string().replace(".", ""), self.exponent + 1, &String::from("0")) +
					if places > 0 {
						pad_end(&String::from("."), places + 1, &String::from("0"))
					} else {
						""
					});
			}

			return to_fixed(self.to_number(), places);
		}
		fn to_precision(&self, places: i32) -> String {
			if self.exponent <= -7.0 {
				return self.to_exponential(places - 1);
			}

			if (places as f64) > self.exponent {
				return self.to_fixed(places - self.exponent - 1);
			}

			return self.to_exponential(places - 1);
		}

		fn mantissa_with_decimal_places(&self, places: i32) -> f64 {
			if f64::is_nan(self.mantissa) || f64::is_nan(self.exponent) {
				return NAN;
			} else if self.mantissa == 0.0 {
				return 0.0
			}

			let len = places + 1;
			let num_digits = self.mantissa.abs().log10().ceil();
			let rounded = (self.mantissa * 10.0.powi(len - num_digits)).round() * 10.0.powi(num_digits - len);
			return to_fixed_num(rounded, 0.0.max(len - num_digits) as i32);
		}

		fn value_of(&self) -> String {
			return self.to_string();
		}
		fn to_json(&self) -> String {
			return self.to_string();
		}
		fn to_string_with_decimal_places(&self, places: i32) -> String {
			return self.to_exponential(places);
		}

		fn abs(&self) -> Decimal {
			return from_mantissa_exponent_no_normalize(self.mantissa.abs(), self.exponent);
		}

		fn neg(&self) -> Decimal {
			return from_mantissa_exponent_no_normalize(-self.mantissa, self.exponent);
		}
		fn negate(&self) -> Decimal {
			return self.neg();
		}
		fn negated(&self) -> Decimal {
			return self.neg();
		}

		fn sign(&self) -> i32 {
			return if self.mantissa.is_sign_positive() {
				1
			} else if  self.mantissa.is_sign_negative() {
				-1
			} else {
				0
			};
		}
		fn sgn(&self) -> i32 {
			return self.sign();
		}

		fn round(&self) -> Decimal {
			if self.exponent < -1.0 {
				return new(0.0);
			} else if self.exponent < MAX_SIGNIFICANT_DIGITS as f64 {
				return new(self.to_number().round());
			}

			return from_decimal(self);
		}
		fn trunc(&self) -> Decimal {
			if self.exponent < 0.0 {
				return new(0.0);
			} else if self.exponent < MAX_SIGNIFICANT_DIGITS as f64 {
				return new(self.to_number().trunc());
			}

			return from_decimal(self);
		}
		fn floor(&self) -> Decimal {
			if self.exponent < -1.0 {
				return if self.sign() >= 0 {
					new(0.0)
				} else {
					new(-1.0)
				}
			} else if self.exponent < MAX_SIGNIFICANT_DIGITS as f64 {
				return new(self.to_number().floor());
			}

			return from_decimal(self);
		}
		fn ceil(&self) -> Decimal {
			if self.exponent < -1.0 {
				return if self.sign() > 0 {
					new(1.0)
				} else {
					new(0.0)
				};
			} else if self.exponent < MAX_SIGNIFICANT_DIGITS as f64 {
				return new(self.to_number().ceil());
			}

			return from_decimal(self);
		}

		fn add(&self, decimal: &Decimal) -> Decimal {
			if self.mantissa == 0.0 {
				return from_decimal(decimal);
			} else if decimal.mantissa == 0.0 {
				return from_decimal(self);
			}

			let mut bigger_decimal;
			let mut smaller_decimal;

			if self.exponent >= decimal.exponent {
				bigger_decimal = from_decimal(self);
				smaller_decimal = from_decimal(&decimal);
			} else {
				bigger_decimal = from_decimal(&decimal);
				smaller_decimal = from_decimal(self);
			}

			if bigger_decimal.exponent - smaller_decimal.exponent > MAX_SIGNIFICANT_DIGITS as f64 {
				return bigger_decimal;
			}

			return from_mantissa_exponent((1e14 * bigger_decimal.mantissa) + 1e14 * &smaller_decimal * POWER_OF_10((&smaller_decimal.exponent - bigger_decimal.exponent) as i32).round(), bigger_decimal.exponent - 14);
		}
		fn plus(&self, decimal: &Decimal) -> Decimal {
			return self.add(decimal);
		}

		fn sub(&self, decimal: &Decimal) -> Decimal {
			return self.add(&decimal.neg());
		}
		fn subtract(&self, decimal: &Decimal) -> Decimal {
			return self.sub(decimal);
		}
		fn minus(&self, decimal: &Decimal) -> Decimal {
			return self.sub(decimal);
		}

		fn mul(&self, decimal: &Decimal) -> Decimal {
			return from_mantissa_exponent(self.mantissa * decimal.mantissa, self.exponent + decimal.exponent);
		}
		fn multiply(&self, decimal: &Decimal) -> Decimal {
			return self.mul(decimal);
		}
		fn times(&self, decimal: &Decimal) -> Decimal {
			return self.mul(decimal);
		}

		fn div(&self, decimal: &Decimal) -> Decimal {
			return self.mul(&decimal.recip());
		}
		fn divide(&self, decimal: &Decimal) -> Decimal {
			return self.div(decimal);
		}
		fn divide_by(&self, decimal: &Decimal) -> Decimal {
			return self.div(decimal);
		}
		fn divided_by(&self, decimal: &Decimal) -> Decimal {
			return self.div(decimal);
		}

		fn recip(&self) -> Decimal {
			return from_mantissa_exponent(1 / self.mantissa, -self.exponent);
		}
		fn reciprocal(&self) -> Decimal {
			return self.recip();
		}
		fn reciprocate(&self) -> Decimal {
			return self.recip();
		}


		fn cmp(&self, decimal: &Decimal) -> i32 {
			/*
			From smallest to largest:
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
			*/

			if self.mantissa == 0.0 {
				if decimal.mantissa == 0.0 {
					return 0;
				} else if decimal.mantissa < 0.0 {
					return 1;
				} else if decimal.mantissa > 0.0 {
					return -1;
				}
			}

			if decimal.mantissa == 0.0 {
				if self.mantissa < 0.0 {
					return -1;
				} else if self.mantissa > 0.0 {
					return 1;
				}
			}

			if self.mantissa > 0.0 {
				if decimal.mantissa < 0.0 {
					return 1;
				} else if self.exponent > decimal.exponent {
					return 1;
				} else if self.exponent < decimal.exponent {
					return -1;
				} else if self.mantissa > decimal.mantissa {
					return 1;
				} else if self.mantissa < decimal.mantissa {
					return -1;
				}

				return 0;
			}

			if self.mantissa < 0.0 {
				if decimal.mantissa > 0.0 {
					return -1;
				} else if self.exponent > decimal.exponent {
					return -1;
				} else if self.exponent < decimal.exponent {
					return 1;
				} else if self.mantissa > decimal.mantissa {
					return 1;
				} else if self.mantissa < decimal.mantissa {
					return -1;
				}

				return 0;
			}

			return NAN as i32;
		}
		fn compare(&self, decimal: &Decimal) -> i32 {
			return self.cmp(decimal);
		}

		fn eq(&self, decimal: &Decimal) -> bool {
			return self.exponent == decimal.exponent && self.mantissa == decimal.exponent;
		}
		fn equals(&self, decimal: &Decimal) -> bool {
			return self.equals(decimal);
		}

		fn neq(&self, decimal: &Decimal) -> bool {
			return !self.eq(decimal);
		}
		fn not_equals(&self, decimal: &Decimal) -> bool {
			return !self.neq(decimal);
		}

		fn lt(&self, decimal: &Decimal) -> bool {
			if self.mantissa == 0.0 {
				return decimal.mantissa > 0.0;
			} else if decimal.mantissa == 0.0 {
				return self.mantissa <= 0.0;
			} else if self.exponent == decimal.exponent {
				return self.mantissa < decimal.mantissa;
			} else if self.mantissa > 0.0 {
				return decimal.mantissa > 0.0 && self.exponent < decimal.exponent;
			}

			return decimal.mantissa > 0.0 || self.exponent > decimal.exponent;
		}
		fn lte(&self, decimal: &Decimal) -> bool {
			return !self.gt(decimal);
		}

		fn gt(&self, decimal: &Decimal) -> bool {
			if self.mantissa == 0.0 {
				return decimal.mantissa < 0.0;
			} else if decimal.mantissa == 0.0 {
				return self.mantissa > 0.0;
			} else if self.exponent == decimal.exponent {
				return self.mantissa > decimal.mantissa;
			} else if self.mantissa > 0.0 {
				return decimal.mantissa < 0.0 || self.exponent > decimal.exponent;
			}

			return decimal.mantissa < 0.0 && self.exponent < decimal.exponent;
		}
		fn gte(&self, decimal: &Decimal) -> bool {
			return !self.lt(decimal);
		}

		fn max(&self, decimal: &Decimal) -> Decimal{
			return if self.lt(decimal) {
				from_decimal(decimal);
			} else {
				from_decimal(self);
			}
		}

		fn min(&self, decimal: &Decimal) -> Decimal {
			return if self.gt(decimal) {
				from_decimal(decimal);
			} else {
				from_decimal(self);
			}
		}

		fn clamp(&self, min: &Decimal, max: &Decimal) -> Decimal {
			return self.max(min).min(max);
		}
		fn clamp_min(&self, min: &Decimal) -> Decimal {
			return self.max(min);
		}
		fn clamp_max(&self, max: &Decimal) -> Decimal {
			return self.min(max);
		}

		fn cmp_tolerance(&self, decimal: &Decimal, tolerance: &Decimal) -> i32 {
			return if self.eq_tolerance(decimal, tolerance) {
				0
			} else {
				self.cmp(decimal)
			}
		}
		fn compare_tolerance(&self, decimal: &Decimal, tolerance: &Decimal) -> i32 {
			return self.cmp_tolerance(decimal, tolerance);
		}

		fn eq_tolerance(&self, decimal: &Decimal, tolerance: &Decimal) -> bool {
			// return abs(a-b) <= tolerance * max(abs(a), abs(b))
			return self.sub(decimal).abs().lte(&self.abs().max(&decimal.abs().mul(tolerance)));
		}
		fn equals_tolerance(&self, decimal: &Decimal, tolerance: &Decimal) -> bool {
			return self.eq_tolerance(decimal, tolerance);
		}

		fn neq_tolerance(&self, decimal: &Decimal, tolerance: &Decimal) -> bool {
			return !self.eq_tolerance(decimal, tolerance);
		}
		fn not_equals_tolerance(&self, decimal: &Decimal, tolerance: &Decimal) -> bool {
			return self.neq_tolerance(decimal, tolerance);
		}

		fn lt_tolerance(&self, decimal: &Decimal, tolerance: &Decimal) -> bool {
			return !self.eq_tolerance(decimal, tolerance) && self.lt(decimal);
		}
		fn lte_tolerance(&self, decimal: &Decimal, tolerance: &Decimal) -> bool {
			return self.eq_tolerance(decimal, tolerance) || self.lt(decimal);
		}

		fn gt_tolerance(&self, decimal: &Decimal, tolerance: &Decimal) -> bool {
			return !self.eq_tolerance(decimal, tolerance) && self.gt(decimal);
		}
		fn gte_tolerance(&self, decimal: &Decimal, tolerance: &Decimal) -> bool {
			return self.eq_tolerance(decimal, tolerance) || self.gt(decimal);
		}
	}
}
