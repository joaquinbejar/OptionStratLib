::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[model](../index.html)
:::

# Module utils Copy item path

[[Source](../../../src/optionstratlib/model/utils.rs.html#6-607){.src}
]{.sub-heading}
::::

Expand description

::: docblock
Utility functions supporting various operations across the library.
:::

## Traits[§](#traits){.anchor} {#traits .section-header}

[ToRound](trait.ToRound.html "trait optionstratlib::model::utils::ToRound"){.trait}
:   Trait for rounding operations on numeric types, specifically for
    financial calculations.

## Functions[§](#functions){.anchor} {#functions .section-header}

[calculate_optimal_price_range](fn.calculate_optimal_price_range.html "fn optionstratlib::model::utils::calculate_optimal_price_range"){.fn}
:   Calculates the optimal price range for an option based on its
    underlying price, strike price, implied volatility, and expiration
    date.

[create_sample_option](fn.create_sample_option.html "fn optionstratlib::model::utils::create_sample_option"){.fn}
:   Creates a sample option contract with predefined parameters for
    testing or demonstration purposes.

[create_sample_option_simplest](fn.create_sample_option_simplest.html "fn optionstratlib::model::utils::create_sample_option_simplest"){.fn}
:   Creates a simplified sample option contract for testing or
    demonstration purposes.

[create_sample_option_simplest_strike](fn.create_sample_option_simplest_strike.html "fn optionstratlib::model::utils::create_sample_option_simplest_strike"){.fn}
:   Creates a sample option with specified parameters and default
    values.

[create_sample_option_with_date](fn.create_sample_option_with_date.html "fn optionstratlib::model::utils::create_sample_option_with_date"){.fn}
:   Creates a sample Options object with a specific expiration date.

[create_sample_position](fn.create_sample_position.html "fn optionstratlib::model::utils::create_sample_position"){.fn}
:   Creates a sample position for testing and demonstration purposes.

[generate_price_points](fn.generate_price_points.html "fn optionstratlib::model::utils::generate_price_points"){.fn}
:   Generates a price vector for the payoff graph

[mean_and_std](fn.mean_and_std.html "fn optionstratlib::model::utils::mean_and_std"){.fn}
:   Computes the mean and standard deviation of a vector containing
    `Positive` values.

[positive_f64_to_f64](fn.positive_f64_to_f64.html "fn optionstratlib::model::utils::positive_f64_to_f64"){.fn}
:   Converts a vector of `Positive` values to a vector of `f64` values.
::::::
:::::::
