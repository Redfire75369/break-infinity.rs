#[macro_export]
macro_rules! impl_from {
	($from_type:ty) => {
		impl From<$from_type> for Decimal {
			fn from(num: $from_type) -> Decimal {
				Decimal::new(num as f64)
			}
		}
	};
}