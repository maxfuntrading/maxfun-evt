pub mod log;
mod error;
mod period;

pub use error::LibError;
pub use period::PeriodType;

pub type LibResult<T> = Result<T, LibError>;


// #[inline]
// // pub fn i128_decimal(amount: i128) -> LibResult<Decimal> {
// //     let amount = Decimal::try_from_i128_with_scale(amount, consts::DECIMAL)?;
// //     Ok(amount)
// }


// #[inline]
// pub fn day_ts(ts: i64) -> i64 {
//     ts - ts % 86400
// }
//
//
// pub fn min5_ts(ts: i64) -> i64 {
//     ts - ts % 300
// }

