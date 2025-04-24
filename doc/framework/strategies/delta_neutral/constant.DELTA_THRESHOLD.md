::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[strategies](../index.html)::[delta_neutral](index.html)
:::

# Constant [DELTA_THRESHOLD]{.constant}Copy item path

[[Source](../../../src/optionstratlib/strategies/delta_neutral/model.rs.html#78){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub const DELTA_THRESHOLD: Decimal;
```

Expand description

::: docblock
## [§](#delta-neutrality-threshold){.doc-anchor}Delta Neutrality Threshold

The default threshold value used to determine if an options strategy is
considered delta neutral.

When evaluating delta neutrality, a strategy's net delta is compared
against this threshold value. If the absolute value of the net delta is
less than or equal to this threshold, the strategy is considered delta
neutral.

### [§](#value-significance){.doc-anchor}Value Significance

The small value (0.0001) represents a very tight threshold, meaning the
strategy must have extremely minimal directional exposure to be
considered neutral. This conservative threshold helps ensure strategies
maintain strict delta neutrality for effective risk management.

### [§](#usage-context){.doc-anchor}Usage Context

This constant is primarily used within delta neutrality calculations and
serves as a default when a custom threshold is not specified. Functions
that analyze or adjust strategies for delta neutrality may use this
value when determining if additional position adjustments are necessary.

### [§](#related-components){.doc-anchor}Related Components

Works in conjunction with the `DeltaInfo` and `DeltaNeutralResponse`
structures to provide consistent evaluation of delta neutrality across
the delta neutral strategies module.
:::
::::::
:::::::
