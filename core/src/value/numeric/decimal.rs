use super::Rational;

/// Decimal number.
/// 
/// This is wrapper around rational numbers with a finite decimal
/// representation.
pub struct Decimal(Rational);