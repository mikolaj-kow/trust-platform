use super::ConversionType;
use crate::datetime::{days_to_ticks, nanos_to_ticks, ticks_per_day, DivisionMode, NANOS_PER_DAY};
use crate::error::RuntimeError;
use crate::value::{
    DateTimeProfile, DateTimeValue, DateValue, LDateTimeValue, LTimeOfDayValue, TimeOfDayValue,
    Value,
};

pub(super) fn convert_to_time(value: &Value, dst: ConversionType) -> Result<Value, RuntimeError> {
    match (value, dst) {
        (Value::Time(duration), ConversionType::Time) => Ok(Value::Time(*duration)),
        (Value::LTime(duration), ConversionType::LTime) => Ok(Value::LTime(*duration)),
        (Value::Time(duration), ConversionType::LTime) => Ok(Value::LTime(*duration)),
        (Value::LTime(duration), ConversionType::Time) => Ok(Value::Time(*duration)),
        (Value::DWord(value), ConversionType::Time) => Ok(Value::Time(
            crate::value::Duration::from_millis(i64::from(*value)),
        )),
        // CODESYS numeric-to-duration parity: the value is the millisecond
        // magnitude, truncated; negative or non-finite text and magnitudes
        // beyond the vendor DWORD-ms range reject with an overflow fault
        // (IEC_DEVIATIONS DEV-025).
        (Value::Real(value), ConversionType::Time) => {
            let millis = crate::numeric::math::trunc(*value as f64);
            if !value.is_finite() || millis < 0.0 || millis > f64::from(u32::MAX) {
                return Err(RuntimeError::Overflow);
            }
            Ok(Value::Time(crate::value::Duration::from_millis(
                millis as i64,
            )))
        }
        (Value::UInt(value), ConversionType::Time) => Ok(Value::Time(
            crate::value::Duration::from_millis(i64::from(*value)),
        )),
        _ => Err(RuntimeError::TypeMismatch),
    }
}

pub(super) fn convert_to_date(value: &Value, dst: ConversionType) -> Result<Value, RuntimeError> {
    let profile = DateTimeProfile::default();
    match (value, dst) {
        (Value::Date(date), ConversionType::Date) => Ok(Value::Date(*date)),
        (Value::LDate(date), ConversionType::LDate) => Ok(Value::LDate(*date)),
        (Value::Dt(dt), ConversionType::Date) => {
            let days = dt_ticks_to_days(dt, profile)?;
            let ticks = days_to_ticks(days, profile)?;
            Ok(Value::Date(DateValue::new(ticks)))
        }
        (Value::Ldt(dt), ConversionType::Date) => {
            let days = ldt_nanos_to_days(dt)?;
            let ticks = days_to_ticks(days, profile)?;
            Ok(Value::Date(DateValue::new(ticks)))
        }
        _ => Err(RuntimeError::TypeMismatch),
    }
}

pub(super) fn convert_to_tod(value: &Value, dst: ConversionType) -> Result<Value, RuntimeError> {
    let profile = DateTimeProfile::default();
    match (value, dst) {
        (Value::Tod(tod), ConversionType::Tod) => Ok(Value::Tod(*tod)),
        (Value::LTod(tod), ConversionType::LTod) => Ok(Value::LTod(*tod)),
        // CODESYS DWORD_TO_TOD parity: the raw ms-of-day magnitude without
        // 24-hour validation, mirroring TOD's DWORD representation
        // (IEC_DEVIATIONS DEV-025).
        (Value::DWord(value), ConversionType::Tod) => {
            Ok(Value::Tod(TimeOfDayValue::new(i64::from(*value))))
        }
        (Value::LTod(tod), ConversionType::Tod) => {
            let ticks = nanos_to_ticks(tod.nanos(), profile, DivisionMode::Euclid)?;
            Ok(Value::Tod(TimeOfDayValue::new(ticks)))
        }
        (Value::Tod(tod), ConversionType::LTod) => {
            let nanos = ticks_to_nanos(tod.ticks(), profile)?;
            Ok(Value::LTod(LTimeOfDayValue::new(nanos)))
        }
        (Value::Dt(dt), ConversionType::Tod) => {
            let ticks = dt_ticks_to_tod_ticks(dt, profile)?;
            Ok(Value::Tod(TimeOfDayValue::new(ticks)))
        }
        (Value::Dt(dt), ConversionType::LTod) => {
            let nanos = dt_ticks_to_tod_nanos(dt, profile)?;
            Ok(Value::LTod(LTimeOfDayValue::new(nanos)))
        }
        (Value::Ldt(dt), ConversionType::Tod) => {
            let ticks = ldt_nanos_to_tod_ticks(dt, profile)?;
            Ok(Value::Tod(TimeOfDayValue::new(ticks)))
        }
        (Value::Ldt(dt), ConversionType::LTod) => {
            let nanos = ldt_nanos_to_tod_nanos(dt)?;
            Ok(Value::LTod(LTimeOfDayValue::new(nanos)))
        }
        _ => Err(RuntimeError::TypeMismatch),
    }
}

pub(super) fn convert_to_dt(value: &Value, dst: ConversionType) -> Result<Value, RuntimeError> {
    let profile = DateTimeProfile::default();
    match (value, dst) {
        (Value::Dt(dt), ConversionType::Dt) => Ok(Value::Dt(*dt)),
        (Value::Ldt(dt), ConversionType::Ldt) => Ok(Value::Ldt(*dt)),
        (Value::Dt(dt), ConversionType::Ldt) => {
            let nanos = dt_ticks_to_nanos(dt, profile)?;
            Ok(Value::Ldt(LDateTimeValue::new(nanos)))
        }
        (Value::Ldt(dt), ConversionType::Dt) => {
            let ticks = ldt_nanos_to_ticks(dt, profile)?;
            Ok(Value::Dt(DateTimeValue::new(ticks)))
        }
        _ => Err(RuntimeError::TypeMismatch),
    }
}

fn dt_ticks_to_days(dt: &DateTimeValue, profile: DateTimeProfile) -> Result<i64, RuntimeError> {
    let ticks = dt
        .ticks()
        .checked_sub(profile.epoch.ticks())
        .ok_or(RuntimeError::Overflow)?;
    let per_day = ticks_per_day(profile)?;
    Ok(ticks.div_euclid(per_day))
}

fn ldt_nanos_to_days(dt: &LDateTimeValue) -> Result<i64, RuntimeError> {
    Ok(dt.nanos().div_euclid(NANOS_PER_DAY))
}

fn dt_ticks_to_nanos(dt: &DateTimeValue, profile: DateTimeProfile) -> Result<i64, RuntimeError> {
    let res = profile.resolution.as_nanos();
    dt.ticks()
        .checked_sub(profile.epoch.ticks())
        .and_then(|v| v.checked_mul(res))
        .ok_or(RuntimeError::Overflow)
}

fn ldt_nanos_to_ticks(dt: &LDateTimeValue, profile: DateTimeProfile) -> Result<i64, RuntimeError> {
    let res = profile.resolution.as_nanos();
    let ticks = dt.nanos().div_euclid(res);
    ticks
        .checked_add(profile.epoch.ticks())
        .ok_or(RuntimeError::Overflow)
}

fn dt_ticks_to_tod_ticks(
    dt: &DateTimeValue,
    profile: DateTimeProfile,
) -> Result<i64, RuntimeError> {
    let ticks = dt
        .ticks()
        .checked_sub(profile.epoch.ticks())
        .ok_or(RuntimeError::Overflow)?;
    let per_day = ticks_per_day(profile)?;
    Ok(ticks.rem_euclid(per_day))
}

fn dt_ticks_to_tod_nanos(
    dt: &DateTimeValue,
    profile: DateTimeProfile,
) -> Result<i64, RuntimeError> {
    let ticks = dt_ticks_to_tod_ticks(dt, profile)?;
    ticks_to_nanos(ticks, profile)
}

fn ldt_nanos_to_tod_ticks(
    dt: &LDateTimeValue,
    profile: DateTimeProfile,
) -> Result<i64, RuntimeError> {
    let nanos = ldt_nanos_to_tod_nanos(dt)?;
    Ok(nanos_to_ticks(nanos, profile, DivisionMode::Euclid)?)
}

fn ldt_nanos_to_tod_nanos(dt: &LDateTimeValue) -> Result<i64, RuntimeError> {
    Ok(dt.nanos().rem_euclid(NANOS_PER_DAY))
}

fn ticks_to_nanos(ticks: i64, profile: DateTimeProfile) -> Result<i64, RuntimeError> {
    let res = profile.resolution.as_nanos();
    ticks.checked_mul(res).ok_or(RuntimeError::Overflow)
}
