::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[strategies](../index.html)
:::

# Module baseCopy item path

[[Source](../../../src/optionstratlib/strategies/base.rs.html#6-2184){.src}
]{.sub-heading}
::::

Expand description

::: docblock
Options trading strategies module collection

This module provides implementations of various options trading
strategies and utility functions for options trading analysis. Each
submodule represents a specific strategy or utility.
:::

## Structs[§](#structs){.anchor} {#structs .section-header}

[Strategy](struct.Strategy.html "struct optionstratlib::strategies::base::Strategy"){.struct}
:   Represents a complete options trading strategy with risk-reward
    parameters.

[StrategyBasics](struct.StrategyBasics.html "struct optionstratlib::strategies::base::StrategyBasics"){.struct}
:   Represents basic information about a trading strategy.

## Enums[§](#enums){.anchor} {#enums .section-header}

[StrategyType](enum.StrategyType.html "enum optionstratlib::strategies::base::StrategyType"){.enum}
:   Represents different option trading strategies.

## Traits[§](#traits){.anchor} {#traits .section-header}

[BreakEvenable](trait.BreakEvenable.html "trait optionstratlib::strategies::base::BreakEvenable"){.trait}
:   Trait for strategies that can calculate and update break-even
    points.

[Optimizable](trait.Optimizable.html "trait optionstratlib::strategies::base::Optimizable"){.trait}
:   This trait defines methods for optimizing and validating option
    strategies. It combines the `Validable` and `Strategies` traits,
    requiring implementors to provide functionality for both validation
    and strategy generation.

[Positionable](trait.Positionable.html "trait optionstratlib::strategies::base::Positionable"){.trait}
:   The `Positionable` trait defines methods for managing positions
    within a trading strategy. These methods allow for adding,
    retrieving, and modifying positions, providing a common interface
    for different strategies to interact with position data.

[Strategable](trait.Strategable.html "trait optionstratlib::strategies::base::Strategable"){.trait}
:   This trait defines common functionalities for all trading
    strategies. It combines several other traits, requiring
    implementations for methods related to strategy information,
    construction, optimization, profit calculation, graphing,
    probability analysis, Greeks calculation, delta neutrality, and P&L
    calculation.

[Strategies](trait.Strategies.html "trait optionstratlib::strategies::base::Strategies"){.trait}
:   Defines a set of strategies for options trading. Provides methods
    for calculating key metrics such as profit/loss, cost, break-even
    points, and price ranges. Implementations of this trait must also
    implement the `Validable`, `Positionable`, and `BreakEvenable`
    traits.

[Validable](trait.Validable.html "trait optionstratlib::strategies::base::Validable"){.trait}
:   This trait defines a way to validate a strategy.
::::::
:::::::
