//! # Utils Module
//!
//! Cross-cutting helpers whose defining owner is the `core` layer: they
//! depend on nothing above `core` and every layer may call them. This
//! module is deliberately *not* a catch-all. Each file below names one
//! concern and one owner, and the ownership is machine-checked by
//! `scripts/check_module_boundaries.py` (`UTILS_FILE_LAYER`), so a helper
//! that grows a dependency on a higher layer fails the graph check instead
//! of quietly turning this module back into a shared dumping ground
//! (ADR-0001 D2, roadmap M1-09).
//!
//! | File | Owner | Contents |
//! |------|-------|----------|
//! | `numeric.rs` | core | `approx_equal`, `calculate_log_returns` |
//! | `rng.rs` | core | `deterministic_rng`, `get_random_element`, `random_decimal`, `DETERMINISTIC_RNG_DEFAULT_SEED` |
//! | `time.rs` | core | `TimeFrame`, `units_per_year`, `convert_time_frame`, date formatting |
//! | `traits.rs` | core | `Len` |
//!
//! Helpers that used to live here and now sit with their owner:
//!
//! - `prepare_file_path` is `visualization::prepare_file_path`: preparing a
//!   path on disk exists to write a rendered chart, and nothing below
//!   `visualization` calls it.
//! - CSV and OHLCV parsing belong to `market` (`chains`), re-homed in M1-08.
//! - `setup_logger` / `setup_logger_with_level` are **removed**. Installing a
//!   global `tracing` subscriber is an application decision, not a library
//!   one (`rules/global_rules.md`, "Logging & Observability"), so the library
//!   no longer depends on `tracing-subscriber` at all. Install one from your
//!   binary:
//!
//! ```rust,ignore
//! tracing_subscriber::fmt()
//!     .with_max_level(tracing::Level::INFO)
//!     .init();
//! ```
//!
//! The example binaries take theirs from the unpublished `osl-example-support`
//! workspace member, which is where application-side wiring belongs.
//!
//! Every other module keeps its own `utils.rs` (`greeks::utils`,
//! `chains::utils`, `strategies::utils`, ...). Those are owned by their
//! parent module by construction and are checked as part of it; nothing is
//! promoted here just because more than one caller wants it.
//!
//! ## Time frames
//!
//! ```rust
//! use positive::pos_or_panic;
//! use optionstratlib::utils::time::TimeFrame;
//!
//! let daily = TimeFrame::Day;
//! let trading_days_per_year = daily.periods_per_year(); // Returns 252.0
//!
//! let custom = TimeFrame::Custom(pos_or_panic!(365.0));
//! let periods = custom.periods_per_year(); // Returns 365.0
//! ```
//!
//! Time-frame conversions are constant-time; they read the predefined
//! period constants in `crate::constants`.
//!
//! ## Numeric comparison
//!
//! ```rust
//! use optionstratlib::utils::numeric::approx_equal;
//!
//! assert!(approx_equal(1.0, 1.000_000_01));
//! assert!(!approx_equal(1.0, 1.1));
//! ```
//!
//! ## Randomness
//!
//! Every Monte-Carlo or simulation test seeds deterministically, so an
//! upstream precision shift cannot flip an assertion by luck:
//!
//! ```rust
//! use optionstratlib::utils::deterministic_rng;
//! use rand::RngExt;
//!
//! let mut rng = deterministic_rng(42);
//! let _: u64 = rng.random();
//! ```
//!
//! Random element selection is O(n) in the size of the set.

/// Tolerance-based `f64` comparison and the logarithmic-return transform.
pub mod numeric;
/// Deterministic seeding and generic random-sampling helpers.
pub mod rng;
/// Time frames, period conversions and date formatting.
pub mod time;
/// Traits shared across the library.
mod traits;

pub use numeric::{approx_equal, calculate_log_returns};
pub use rng::{
    DETERMINISTIC_RNG_DEFAULT_SEED, deterministic_rng, get_random_element, random_decimal,
};
pub use time::TimeFrame;
pub use traits::Len;
