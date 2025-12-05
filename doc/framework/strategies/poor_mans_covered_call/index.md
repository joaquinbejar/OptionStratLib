::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[strategies](../index.html)
:::

# Module poor_mans_covered_call Copy item path

[[Source](../../../src/optionstratlib/strategies/poor_mans_covered_call.rs.html#1-2315){.src}
]{.sub-heading}
::::

Expand description

::: docblock
Poor Man's Covered Call strategy implementation

The "Poor Man's Covered Call" is an options strategy designed to
simulate a traditional covered call, but with a lower capital
requirement. In a standard covered call, an investor holds a long
position in the underlying asset (e.g., a stock) and sells a call option
against it to generate premium income. This strategy works well for
neutral to slightly bullish market outlooks. However, instead of
purchasing the underlying asset (which can be capital-intensive), the
"Poor Man's Covered Call" involves buying a deep-in-the-money LEAP
(Long-term Equity Anticipation Security) call option with a long
expiration date and selling a short-term out-of-the-money call option
against it. By using a LEAP, the investor still benefits from the
movement of the underlying asset while avoiding the need to purchase it
outright. The premium collected from selling the short-term call
generates income and helps offset the cost of the LEAP.

The strategy has two main components:

1.  **Long LEAP Call**: This serves as a substitute for holding the
    underlying asset. The deep-in-the-money LEAP behaves similarly to
    the underlying asset's price movement but costs a fraction of its
    price. The LEAP should have a delta close to 1, meaning it moves
    nearly dollar-for-dollar with the underlying asset.
2.  **Short Call**: A short-term out-of-the-money call is sold against
    the long LEAP. This generates premium income, and if the underlying
    asset's price rises above the strike price of the short call, the
    investor may need to sell the asset (or close the position), locking
    in potential gains.

The goal is to capture some upside potential of the underlying asset
while reducing risk through a lower capital commitment. The key risks
involve the loss of the premium collected if the underlying asset does
not move favorably and potential limitations on profits if the
underlying asset's price rises sharply, triggering the short call. This
strategy is often used by investors who are moderately bullish on an
asset but wish to reduce the cost and risk associated with traditional
covered call strategies.
:::

## Structs[§](#structs){.anchor} {#structs .section-header}

[PoorMansCoveredCall](struct.PoorMansCoveredCall.html "struct optionstratlib::strategies::poor_mans_covered_call::PoorMansCoveredCall"){.struct}
:   PoorMansCoveredCall
::::::
:::::::
