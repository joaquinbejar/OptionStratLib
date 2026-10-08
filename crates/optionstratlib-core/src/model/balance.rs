/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 22/9/25
******************************************************************************/

use crate::error::DecimalError;
use crate::model::decimal::{d_div, d_mul};
use crate::model::types::UnderlyingAssetType;
use positive::{Positive, PositiveError};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Represents the balance of a specific option position in an exchange.
///
/// This struct encapsulates all the information needed to track an option position,
/// including quantity, premium information, and profit/loss calculations.
/// This balance is specifically designed for options trading.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
pub struct Balance {
    /// Symbol or Epic of the option contract
    pub symbol: String,
    /// Quantity of option contracts held
    pub quantity: Positive,
    /// Average premium at which the option was acquired
    pub average_premium: Positive,
    /// Current market premium of the option (if available)
    pub current_premium: Option<Positive>,
    /// Name of the exchange where the option is held
    pub exchange: String,
    /// Type of underlying asset (e.g., Stock, Forex, Index)
    pub underlying_asset_type: UnderlyingAssetType,
    /// Margin information for accounts that support leverage
    pub margin_info: Option<MarginInfo>,
}

crate::impl_json_debug_pretty!(Balance);
crate::impl_json_display!(Balance);

impl Balance {
    /// Creates a new Balance instance for an option position.
    ///
    /// # Arguments
    ///
    /// * `symbol` - The option contract symbol (e.g., "AAPL240315C00150000")
    /// * `quantity` - The quantity of option contracts held
    /// * `average_premium` - The average premium paid for the option
    /// * `current_premium` - The current market premium (optional)
    /// * `exchange` - The name of the exchange
    /// * `underlying_asset_type` - The type of the underlying asset
    /// * `margin_info` - Margin information for accounts that support leverage (optional)
    ///
    /// # Returns
    ///
    /// A new Balance instance for the option position
    #[must_use]
    pub fn new(
        symbol: String,
        quantity: Positive,
        average_premium: Positive,
        current_premium: Option<Positive>,
        exchange: String,
        underlying_asset_type: UnderlyingAssetType,
        margin_info: Option<MarginInfo>,
    ) -> Self {
        Self {
            symbol,
            quantity,
            average_premium,
            current_premium,
            exchange,
            underlying_asset_type,
            margin_info,
        }
    }

    /// Calculates the total value of the option position based on current premium.
    ///
    /// # Returns
    ///
    /// The total value (quantity * current_premium) if current_premium is available,
    /// otherwise returns the cost basis (quantity * average_premium). The
    /// product is formed in `Decimal`, with no `f64` step.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::ArithmeticError`] when the product overflows
    /// `Decimal`.
    pub fn get_total_value(&self) -> Result<Positive, PositiveError> {
        // Use average premium if current premium is not available
        let premium = self.current_premium.unwrap_or(self.average_premium);
        self.quantity.checked_mul(&premium)
    }

    /// Calculates the unrealized profit or loss of the position.
    ///
    /// # Returns
    ///
    /// The unrealized PnL as a Decimal. Positive values indicate profit,
    /// negative values indicate loss. Returns zero if current_price is not available.
    #[must_use]
    pub fn get_unrealized_pnl(&self) -> Decimal {
        match self.current_premium {
            Some(current_price) => {
                let current_value = self.quantity.value() * current_price.value();
                let cost_basis = self.quantity.value() * self.average_premium.value();
                current_value - cost_basis
            }
            None => Decimal::ZERO,
        }
    }

    /// Updates the current market premium of the option.
    ///
    /// # Arguments
    ///
    /// * `new_premium` - The new current market premium
    pub fn update_current_premium(&mut self, new_premium: Positive) {
        self.current_premium = Some(new_premium);
    }

    /// Checks if the position is currently profitable.
    ///
    /// # Returns
    ///
    /// `true` if the position has unrealized gains, `false` otherwise.
    /// Returns `false` if current_price is not available.
    #[must_use]
    pub fn is_profitable(&self) -> bool {
        self.get_unrealized_pnl() > Decimal::ZERO
    }

    /// Gets the cost basis of the option position.
    ///
    /// # Returns
    ///
    /// The total cost basis (quantity * average_premium), formed in
    /// `Decimal` with no `f64` step.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::ArithmeticError`] when the product overflows
    /// `Decimal`.
    pub fn get_cost_basis(&self) -> Result<Positive, PositiveError> {
        self.quantity.checked_mul(&self.average_premium)
    }

    /// Gets the percentage return of the position.
    ///
    /// # Returns
    ///
    /// The percentage return as a Decimal. Returns zero if current_price is
    /// not available, and zero for a zero cost basis, where no return is
    /// defined.
    ///
    /// # Errors
    ///
    /// Returns [`DecimalError::Overflow`] when the cost basis, the ratio of
    /// the P&L to it, or its scaling to a percentage overflows `Decimal`.
    pub fn get_percentage_return(&self) -> Result<Decimal, DecimalError> {
        match self.current_premium {
            Some(_current_price) => {
                let pnl = self.get_unrealized_pnl();
                let cost_basis = d_mul(
                    self.quantity.to_dec(),
                    self.average_premium.to_dec(),
                    "Balance::get_percentage_return/cost_basis",
                )?;
                if cost_basis > Decimal::ZERO {
                    let ratio = d_div(pnl, cost_basis, "Balance::get_percentage_return/ratio")?;
                    d_mul(
                        ratio,
                        Decimal::ONE_HUNDRED,
                        "Balance::get_percentage_return/percent",
                    )
                } else {
                    Ok(Decimal::ZERO)
                }
            }
            None => Ok(Decimal::ZERO),
        }
    }
}

/// Represents margin information for accounts that support leverage
#[derive(Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
pub struct MarginInfo {
    /// Available margin for new positions
    pub available_margin: Decimal,
    /// Currently used margin
    pub used_margin: Decimal,
    /// Maintenance margin requirement
    pub maintenance_margin: Decimal,
    /// Initial margin requirement
    pub initial_margin: Option<Decimal>,
    /// Current margin ratio (used/available)
    pub margin_ratio: Option<Decimal>,
}

crate::impl_json_debug_pretty!(MarginInfo);
crate::impl_json_display!(MarginInfo);

/// Represents a portfolio containing multiple option balances.
///
/// This struct provides functionality to manage and analyze a collection
/// of option positions across different exchanges.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
pub struct Portfolio {
    /// Collection of option balances
    pub balances: Vec<Balance>,
    /// Name or identifier for the portfolio
    pub name: String,
}

crate::impl_json_debug_pretty!(Portfolio);
crate::impl_json_display!(Portfolio);

impl Portfolio {
    /// Creates a new empty Portfolio.
    ///
    /// # Arguments
    ///
    /// * `name` - The name or identifier for the portfolio
    ///
    /// # Returns
    ///
    /// A new Portfolio instance with an empty balance collection
    #[must_use]
    pub fn new(name: String) -> Self {
        Self {
            balances: Vec::new(),
            name,
        }
    }

    /// Adds a balance to the portfolio.
    ///
    /// # Arguments
    ///
    /// * `balance` - The Balance to add to the portfolio
    pub fn add_balance(&mut self, balance: Balance) {
        self.balances.push(balance);
    }

    /// Removes a balance from the portfolio by symbol and exchange.
    ///
    /// # Arguments
    ///
    /// * `symbol` - The asset symbol to remove
    /// * `exchange` - The exchange name to match
    ///
    /// # Returns
    ///
    /// `true` if a balance was removed, `false` if not found
    pub fn remove_balance(&mut self, symbol: &str, exchange: &str) -> bool {
        if let Some(pos) = self
            .balances
            .iter()
            .position(|b| b.symbol == symbol && b.exchange == exchange)
        {
            self.balances.remove(pos);
            true
        } else {
            false
        }
    }

    /// Gets a reference to a balance by symbol and exchange.
    ///
    /// # Arguments
    ///
    /// * `symbol` - The asset symbol to find
    /// * `exchange` - The exchange name to match
    ///
    /// # Returns
    ///
    /// An optional reference to the Balance if found
    #[must_use]
    pub fn get_balance(&self, symbol: &str, exchange: &str) -> Option<&Balance> {
        self.balances
            .iter()
            .find(|b| b.symbol == symbol && b.exchange == exchange)
    }

    /// Gets a mutable reference to a balance by symbol and exchange.
    ///
    /// # Arguments
    ///
    /// * `symbol` - The asset symbol to find
    /// * `exchange` - The exchange name to match
    ///
    /// # Returns
    ///
    /// An optional mutable reference to the Balance if found
    pub fn get_balance_mut(&mut self, symbol: &str, exchange: &str) -> Option<&mut Balance> {
        self.balances
            .iter_mut()
            .find(|b| b.symbol == symbol && b.exchange == exchange)
    }

    /// Calculates the total portfolio value.
    ///
    /// # Returns
    ///
    /// The sum of all balance values in the portfolio, accumulated in
    /// `Decimal` with no `f64` step. An empty portfolio is worth zero.
    ///
    /// # Errors
    ///
    /// Returns [`PositiveError::ArithmeticError`] when a balance value or the
    /// running sum overflows `Decimal`.
    pub fn get_total_value(&self) -> Result<Positive, PositiveError> {
        self.balances
            .iter()
            .try_fold(Positive::ZERO, |total, balance| {
                total.checked_add(&balance.get_total_value()?)
            })
    }

    /// Calculates the total unrealized PnL for the portfolio.
    ///
    /// # Returns
    ///
    /// The sum of all unrealized PnL across all balances
    #[must_use]
    pub fn get_total_unrealized_pnl(&self) -> Decimal {
        self.balances
            .iter()
            .map(|balance| balance.get_unrealized_pnl())
            .sum()
    }

    /// Gets all balances from a specific exchange.
    ///
    /// # Arguments
    ///
    /// * `exchange` - The exchange name to filter by
    ///
    /// # Returns
    ///
    /// A vector of references to balances from the specified exchange
    #[must_use]
    pub fn get_balances_by_exchange(&self, exchange: &str) -> Vec<&Balance> {
        self.balances
            .iter()
            .filter(|balance| balance.exchange == exchange)
            .collect()
    }

    /// Checks if the portfolio has any profitable positions.
    ///
    /// # Returns
    ///
    /// `true` if any balance in the portfolio is profitable
    #[must_use]
    pub fn has_profitable_positions(&self) -> bool {
        self.balances.iter().any(|balance| balance.is_profitable())
    }

    /// Gets the number of balances in the portfolio.
    ///
    /// # Returns
    ///
    /// The count of balances in the portfolio
    #[must_use]
    pub fn balance_count(&self) -> usize {
        self.balances.len()
    }

    /// Checks if the portfolio is empty.
    ///
    /// # Returns
    ///
    /// `true` if the portfolio has no balances
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.balances.is_empty()
    }
}

impl Default for Portfolio {
    fn default() -> Self {
        Self::new("Default Portfolio".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use positive::pos_or_panic;

    use rust_decimal_macros::dec;

    #[test]
    fn test_balance_creation() {
        let balance = Balance::new(
            "AAPL240315C00150000".to_string(),
            pos_or_panic!(10.0),
            pos_or_panic!(5.50),
            None,
            "CBOE".to_string(),
            UnderlyingAssetType::Stock,
            None,
        );

        assert_eq!(balance.symbol, "AAPL240315C00150000");
        assert_eq!(balance.quantity, pos_or_panic!(10.0));
        assert_eq!(balance.average_premium, pos_or_panic!(5.50));
        assert_eq!(balance.exchange, "CBOE");
        assert_eq!(balance.underlying_asset_type, UnderlyingAssetType::Stock);
        assert!(balance.current_premium.is_none());
        assert!(balance.margin_info.is_none());
    }

    #[test]
    fn test_balance_total_value() {
        let balance = Balance::new(
            "TSLA240315C00200000".to_string(),
            pos_or_panic!(5.0),
            pos_or_panic!(8.00),
            Some(pos_or_panic!(12.50)),
            "CBOE".to_string(),
            UnderlyingAssetType::Stock,
            None,
        );

        let total_value = balance.get_total_value().expect("finite product");
        assert_eq!(total_value, pos_or_panic!(62.5)); // 5 * 12.50
    }

    #[test]
    fn test_balance_unrealized_pnl() {
        let balance = Balance::new(
            "AAPL240315C00150000".to_string(),
            pos_or_panic!(10.0),
            pos_or_panic!(5.50),
            Some(pos_or_panic!(7.00)),
            "CBOE".to_string(),
            UnderlyingAssetType::Stock,
            None,
        );

        let pnl = balance.get_unrealized_pnl();
        assert_eq!(pnl, dec!(15.0)); // (7.00 - 5.50) * 10
    }

    #[test]
    fn test_balance_is_profitable() {
        let profitable_balance = Balance::new(
            "GOOGL240315C00200000".to_string(),
            pos_or_panic!(5.0),
            pos_or_panic!(10.00),
            Some(pos_or_panic!(12.00)),
            "CBOE".to_string(),
            UnderlyingAssetType::Stock,
            None,
        );

        let losing_balance = Balance::new(
            "TSLA240315C00080000".to_string(),
            pos_or_panic!(3.0),
            pos_or_panic!(8.00),
            Some(pos_or_panic!(6.50)),
            "CBOE".to_string(),
            UnderlyingAssetType::Stock,
            None,
        );

        assert!(profitable_balance.is_profitable());
        assert!(!losing_balance.is_profitable());
    }

    #[test]
    fn test_portfolio_creation() {
        let portfolio = Portfolio::new("My Portfolio".to_string());
        assert_eq!(portfolio.name, "My Portfolio");
        assert!(portfolio.is_empty());
        assert_eq!(portfolio.balance_count(), 0);
    }

    #[test]
    fn test_portfolio_add_balance() {
        let mut portfolio = Portfolio::new("Test Portfolio".to_string());
        let balance = Balance::new(
            "AAPL240315C00150000".to_string(),
            pos_or_panic!(10.0),
            pos_or_panic!(5.50),
            Some(pos_or_panic!(7.00)),
            "CBOE".to_string(),
            UnderlyingAssetType::Stock,
            None,
        );

        portfolio.add_balance(balance);
        assert_eq!(portfolio.balance_count(), 1);
        assert!(!portfolio.is_empty());
    }

    #[test]
    fn test_portfolio_total_value() {
        let mut portfolio = Portfolio::new("Test Portfolio".to_string());

        let balance1 = Balance::new(
            "AAPL240315C00150000".to_string(),
            pos_or_panic!(10.0),
            pos_or_panic!(5.50),
            Some(pos_or_panic!(7.00)),
            "CBOE".to_string(),
            UnderlyingAssetType::Stock,
            None,
        );

        let balance2 = Balance::new(
            "TSLA240315C00200000".to_string(),
            pos_or_panic!(5.0),
            pos_or_panic!(8.00),
            Some(pos_or_panic!(12.50)),
            "CBOE".to_string(),
            UnderlyingAssetType::Stock,
            None,
        );

        portfolio.add_balance(balance1);
        portfolio.add_balance(balance2);

        let total_value = portfolio.get_total_value().expect("finite sum");
        assert_eq!(total_value, pos_or_panic!(132.5)); // 70.0 + 62.5
    }

    #[test]
    fn test_portfolio_get_balance() {
        let mut portfolio = Portfolio::new("Test Portfolio".to_string());
        let balance = Balance::new(
            "AAPL240315C00150000".to_string(),
            pos_or_panic!(10.0),
            pos_or_panic!(5.50),
            Some(pos_or_panic!(7.00)),
            "CBOE".to_string(),
            UnderlyingAssetType::Stock,
            None,
        );

        portfolio.add_balance(balance);

        let found_balance = portfolio.get_balance("AAPL240315C00150000", "CBOE");
        assert!(found_balance.is_some());
        assert_eq!(found_balance.unwrap().symbol, "AAPL240315C00150000");

        let not_found = portfolio.get_balance("TSLA240315C00200000", "CBOE");
        assert!(not_found.is_none());
    }

    fn overflowing_balance(current_premium: Option<Positive>) -> Balance {
        Balance::new(
            "BIG".to_string(),
            Positive::MAX,
            pos_or_panic!(2.0),
            current_premium,
            "CBOE".to_string(),
            UnderlyingAssetType::Stock,
            None,
        )
    }

    #[test]
    fn test_balance_values_are_exact_decimal_products() {
        let balance = Balance::new(
            "AAPL240315C00150000".to_string(),
            pos_or_panic!(3.0),
            pos_or_panic!(0.1),
            Some(pos_or_panic!(0.2)),
            "CBOE".to_string(),
            UnderlyingAssetType::Stock,
            None,
        );

        assert_eq!(
            balance.get_total_value().expect("finite product").to_dec(),
            dec!(0.6)
        );
        assert_eq!(
            balance.get_cost_basis().expect("finite product").to_dec(),
            dec!(0.3)
        );
        assert_eq!(
            balance.get_percentage_return().expect("finite ratio"),
            dec!(100)
        );
    }

    #[test]
    fn test_balance_without_current_premium_values_at_cost() {
        let balance = Balance::new(
            "AAPL240315C00150000".to_string(),
            pos_or_panic!(10.0),
            pos_or_panic!(5.50),
            None,
            "CBOE".to_string(),
            UnderlyingAssetType::Stock,
            None,
        );

        assert_eq!(
            balance.get_total_value().expect("finite product"),
            pos_or_panic!(55.0)
        );
        assert_eq!(
            balance.get_percentage_return().expect("no premium"),
            Decimal::ZERO
        );
    }

    #[test]
    fn test_balance_total_value_overflow_is_an_error_not_zero() {
        let balance = overflowing_balance(Some(pos_or_panic!(2.0)));
        assert!(matches!(
            balance.get_total_value(),
            Err(PositiveError::ArithmeticError { .. })
        ));
    }

    #[test]
    fn test_balance_cost_basis_overflow_is_an_error_not_zero() {
        let balance = overflowing_balance(None);
        assert!(matches!(
            balance.get_cost_basis(),
            Err(PositiveError::ArithmeticError { .. })
        ));
    }

    #[test]
    fn test_balance_percentage_return_ratio_overflow_is_an_error() {
        // A P&L of about 1e10 over a cost basis of 1e-20 is a ratio of 1e30,
        // past `Decimal::MAX`.
        let balance = Balance::new(
            "TINY".to_string(),
            Positive::ONE,
            pos_or_panic!(0.000_000_000_000_000_000_01),
            Some(pos_or_panic!(10_000_000_000.0)),
            "CBOE".to_string(),
            UnderlyingAssetType::Stock,
            None,
        );
        assert!(matches!(
            balance.get_percentage_return(),
            Err(DecimalError::Overflow { .. })
        ));
    }

    #[test]
    fn test_portfolio_total_value_overflow_is_an_error_not_zero() {
        let mut portfolio = Portfolio::new("Overflow".to_string());
        portfolio.add_balance(overflowing_balance(Some(pos_or_panic!(1.0))));
        portfolio.add_balance(overflowing_balance(Some(pos_or_panic!(1.0))));
        assert!(matches!(
            portfolio.get_total_value(),
            Err(PositiveError::ArithmeticError { .. })
        ));
    }

    #[test]
    fn test_empty_portfolio_total_value_is_zero() {
        let portfolio = Portfolio::new("Empty".to_string());
        assert_eq!(
            portfolio.get_total_value().expect("empty sum"),
            Positive::ZERO
        );
    }
}
