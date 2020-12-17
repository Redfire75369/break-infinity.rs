#[cfg(test)]
mod tests {
    use std::f64::{NAN, INFINITY, NEG_INFINITY};
	use std::ptr::null;

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
	}
}
