use rust_decimal::Decimal;

pub trait Helpers {
    fn get_percentage(value: Decimal, total: Decimal) -> Decimal;
}

pub struct Utils {}

impl Helpers for Utils {
    fn get_percentage(value: Decimal, total: Decimal) -> Decimal {
        if value.checked_div(total).is_none() {
            return Decimal::from(0);
        }

        return (value / total) * Decimal::from(100);
    }
}
