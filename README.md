
<div style="text-align: center;">
  <img src="doc/images/logo.png" alt="OptionStratLib" style="width: 100%; height: 200px;">
</div>

OptionStratLib is a comprehensive Rust library for options trading and strategy development across multiple asset classes. This versatile toolkit enables traders, quants, and developers to:

Price options on various underlying assets
Implement complex option strategies
Analyze risk metrics and Greeks
Backtest option trading strategies
Visualize option payoffs and risk profiles

Whether you're dealing with equity options, forex options, or commodity options, OptionStratLib provides the tools you need to develop, test, and optimize your option trading strategies.
Key Features:

Multi-asset support
Flexible strategy creation
Advanced pricing models
Risk analysis tools
Backtesting capabilities
Performance visualization

Empower your options trading with OptionStratLib – the all-in-one solution for option strategy development and analysis.


## Recent Updates

### Implementation of the Binomial Model for Option Pricing

We have successfully implemented a robust binomial model for option pricing. This implementation includes:

1. **Flexible Option Types**: Our model now supports various option types including European, American, and exotic options like Asian, Barrier, and more.

2. **Comprehensive Pricing Parameters**: We've introduced a `BinomialPricingParams` struct that encapsulates all necessary parameters for option pricing, including asset price, volatility, interest rate, strike price, time to expiry, number of steps, option type, option style (Call/Put), and trade side (Long/Short).

3. **Efficient Pricing Algorithm**: The `price_binomial` function implements an efficient binomial tree algorithm for option pricing. It handles special cases such as zero time to expiry and zero volatility.

4. **Support for Both Call and Put Options**: Our implementation allows pricing of both call and put options through the `OptionStyle` enum.

5. **Long and Short Positions**: The model accounts for both long and short positions in options through the `Side` enum.

6. **Payoff Trait**: We've introduced a `Payoff` trait that allows for easy extension to new option types in the future.

7. **Comprehensive Test Suite**: We've developed a suite of unit tests to ensure the accuracy of our pricing model under various scenarios, including edge cases.

8. **Code Optimization**: We've addressed Clippy warnings and optimized our code for better performance and readability.

9. **Detailed Documentation**: We've added comprehensive documentation to our main pricing function, explaining its usage, parameters, and providing examples.

### Future Work

- Implement additional exotic option types
- Enhance the model to handle dividends
- Develop a user-friendly interface for easy option pricing
- Implement additional pricing models (e.g., Black-Scholes, Monte Carlo) for comparison

