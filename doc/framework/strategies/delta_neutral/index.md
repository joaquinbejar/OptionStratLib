::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[strategies](../index.html)
:::

# Module delta_neutralCopy item path

[[Source](../../../src/optionstratlib/strategies/delta_neutral/mod.rs.html#7-39){.src}
]{.sub-heading}
::::

Expand description

::: docblock
Delta-neutral strategy implementation and utilities

## [§](#delta-neutral-strategies-module){.doc-anchor}Delta Neutral Strategies Module

This module provides tools for managing delta-neutral strategies in
options trading. It includes definitions for representing delta
adjustments, calculating delta-neutrality, and suggesting adjustments to
achieve or maintain delta-neutrality.

### [§](#key-components){.doc-anchor}Key Components

- **DeltaAdjustment**: Represents the adjustments (buying/selling
  options or the underlying asset) required to move a strategy toward
  delta neutrality.
- **DeltaInfo**: Provides detailed information about the delta status of
  a strategy, including the net delta, individual position deltas, and
  current neutrality status.
- **DeltaNeutrality Trait**: A trait for calculating net delta, checking
  delta-neutrality, and suggesting adjustments to achieve
  delta-neutrality. It extends the `Greeks` trait for options
  calculations.

### [§](#overview){.doc-anchor}Overview

Delta neutrality is a core concept in options trading, where traders aim
to balance long and short deltas to minimize directional risk. Achieving
delta neutrality often involves adjusting position sizes, adding new
positions, or trading the underlying asset.

The module provides:

- Tools to calculate net delta for multi-position strategies.
- Utilities to evaluate whether a strategy is delta-neutral within a
  given threshold.
- Suggestions for adjustments (buy/sell options or the underlying) to
  achieve neutrality.
:::

## Structs[§](#structs){.anchor} {#structs .section-header}

[DeltaInfo](struct.DeltaInfo.html "struct optionstratlib::strategies::delta_neutral::DeltaInfo"){.struct}
:   Contains detailed information about an options strategy's delta
    status.

[DeltaNeutralResponse](struct.DeltaNeutralResponse.html "struct optionstratlib::strategies::delta_neutral::DeltaNeutralResponse"){.struct}
:   DeltaNeutralResponse

[DeltaPositionInfo](struct.DeltaPositionInfo.html "struct optionstratlib::strategies::delta_neutral::DeltaPositionInfo"){.struct}
:   Represents the delta and associated details for a single position in
    an options strategy.

## Enums[§](#enums){.anchor} {#enums .section-header}

[DeltaAdjustment](enum.DeltaAdjustment.html "enum optionstratlib::strategies::delta_neutral::DeltaAdjustment"){.enum}
:   The `DeltaAdjustment` enum is used to define how a trading strategy
    can be modified to achieve or maintain a delta-neutral state. Delta
    neutrality refers to a situation where the combined delta of all
    positions is close to zero, minimizing directional market risk.

## Constants[§](#constants){.anchor} {#constants .section-header}

[DELTA_THRESHOLD](constant.DELTA_THRESHOLD.html "constant optionstratlib::strategies::delta_neutral::DELTA_THRESHOLD"){.constant}
:   Delta Neutrality Threshold

## Traits[§](#traits){.anchor} {#traits .section-header}

[DeltaNeutrality](trait.DeltaNeutrality.html "trait optionstratlib::strategies::delta_neutral::DeltaNeutrality"){.trait}
:   A trait that provides functionality for managing and maintaining
    delta neutrality in trading strategies.
::::::
:::::::
