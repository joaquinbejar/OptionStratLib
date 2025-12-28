# OptionStratLib Open Issues - Prioritized Roadmap

This document tracks the 26 open GitHub issues and their recommended implementation order.

> **Note:** All issues are now tracked on GitHub. Local issue files have been removed.

## 🔴 Priority High (Implement First)

| Order | GitHub Issue | Title | Labels | Effort |
|-------|--------------|-------|--------|--------|
| 1 | [#227](https://github.com/joaquinbejar/OptionStratLib/issues/227) | refactor: Replace unwrap() calls with proper error handling in src/model | refactor, error-handling, priority-high | High |
| 2 | [#235](https://github.com/joaquinbejar/OptionStratLib/issues/235) | feat: Implement American option pricing model | enhancement, priority-high, pricing | High |

## 🟡 Priority Medium (Implement After High Priority)

### Error Handling & Code Quality

| Order | GitHub Issue | Title | Labels | Effort |
|-------|--------------|-------|--------|--------|
| 3 | [#225](https://github.com/joaquinbejar/OptionStratLib/issues/225) | refactor: Replace expect() calls with proper error handling | refactor, error-handling, priority-medium | Medium |
| 4 | [#215](https://github.com/joaquinbejar/OptionStratLib/issues/215) | fix: Resolve 4 TODO/FIXME items in model/position.rs | priority-medium | Low |

### Pricing Models (Common Exotics)

| Order | GitHub Issue | Title | Labels | Effort |
|-------|--------------|-------|--------|--------|
| 5 | [#236](https://github.com/joaquinbejar/OptionStratLib/issues/236) | feat: Implement Bermuda option pricing model | enhancement, priority-medium, pricing | High |
| 6 | [#237](https://github.com/joaquinbejar/OptionStratLib/issues/237) | feat: Implement Asian option pricing model | enhancement, priority-medium, pricing | High |
| 7 | [#238](https://github.com/joaquinbejar/OptionStratLib/issues/238) | feat: Implement Barrier option pricing model | enhancement, priority-medium, pricing | High |
| 8 | [#239](https://github.com/joaquinbejar/OptionStratLib/issues/239) | feat: Implement Binary option pricing model | enhancement, priority-medium, pricing | Medium |
| 9 | [#240](https://github.com/joaquinbejar/OptionStratLib/issues/240) | feat: Implement Lookback option pricing model | enhancement, priority-medium, pricing | High |
| 10 | [#245](https://github.com/joaquinbejar/OptionStratLib/issues/245) | feat: Implement Spread option pricing model | enhancement, priority-medium, pricing | Medium |
| 11 | [#247](https://github.com/joaquinbejar/OptionStratLib/issues/247) | feat: Implement Exchange option pricing model | enhancement, priority-medium, pricing | Medium |

### Strategies

| Order | GitHub Issue | Title | Labels | Effort |
|-------|--------------|-------|--------|--------|
| 12 | [#218](https://github.com/joaquinbejar/OptionStratLib/issues/218) | feat: Complete implementation of Collar strategy | enhancement, priority-medium, strategies | High |
| 13 | [#219](https://github.com/joaquinbejar/OptionStratLib/issues/219) | feat: Complete implementation of Protective Put strategy | enhancement, priority-medium, strategies | Medium |

### Refactoring & Performance

| Order | GitHub Issue | Title | Labels | Effort |
|-------|--------------|-------|--------|--------|
| 14 | [#216](https://github.com/joaquinbejar/OptionStratLib/issues/216) | refactor: Extract common strategy logic to reduce file sizes | refactor, priority-medium, strategies | High |
| 15 | [#217](https://github.com/joaquinbejar/OptionStratLib/issues/217) | perf: Reduce unnecessary clone() calls across the codebase | refactor, priority-medium, performance | Medium |

## 🟢 Priority Low (Nice-to-Have)

### Exotic Pricing Models

| Order | GitHub Issue | Title | Labels | Effort |
|-------|--------------|-------|--------|--------|
| 16 | [#241](https://github.com/joaquinbejar/OptionStratLib/issues/241) | feat: Implement Compound option pricing model | enhancement, priority-low, pricing | High |
| 17 | [#242](https://github.com/joaquinbejar/OptionStratLib/issues/242) | feat: Implement Chooser option pricing model | enhancement, priority-low, pricing | Medium |
| 18 | [#243](https://github.com/joaquinbejar/OptionStratLib/issues/243) | feat: Implement Cliquet option pricing model | enhancement, priority-low, pricing | High |
| 19 | [#244](https://github.com/joaquinbejar/OptionStratLib/issues/244) | feat: Implement Rainbow option pricing model | enhancement, priority-low, pricing | High |
| 20 | [#246](https://github.com/joaquinbejar/OptionStratLib/issues/246) | feat: Implement Quanto option pricing model | enhancement, priority-low, pricing | High |
| 21 | [#248](https://github.com/joaquinbejar/OptionStratLib/issues/248) | feat: Implement Power option pricing model | enhancement, priority-low, pricing | Medium |

### Infrastructure & Testing

| Order | GitHub Issue | Title | Labels | Effort |
|-------|--------------|-------|--------|--------|
| 22 | [#220](https://github.com/joaquinbejar/OptionStratLib/issues/220) | refactor: Add error context using anyhow at API boundaries | refactor, error-handling, priority-low | Low |
| 23 | [#221](https://github.com/joaquinbejar/OptionStratLib/issues/221) | test: Add comprehensive benchmarks for critical code paths | priority-low, testing, performance | Medium |
| 24 | [#222](https://github.com/joaquinbejar/OptionStratLib/issues/222) | test: Add property-based testing for mathematical invariants | priority-low, testing | Medium |
| 25 | [#223](https://github.com/joaquinbejar/OptionStratLib/issues/223) | feat: Add async feature flag for asynchronous I/O operations | enhancement, priority-low | Medium |
| 26 | [#224](https://github.com/joaquinbejar/OptionStratLib/issues/224) | docs: Add comprehensive documentation for metrics modules | documentation, priority-low | Low |

---

## Recommended Implementation Phases

### Phase 1: Stability & Error Handling (1-2 weeks)
- **#227** - Replace unwrap() in src/model
- **#225** - Replace expect() calls
- **#215** - Resolve TODO/FIXME in position.rs

### Phase 2: Core Pricing (2-3 weeks)
- **#235** - American options (most requested)
- **#236** - Bermuda options
- **#237** - Asian options
- **#238** - Barrier options
- **#239** - Binary options

### Phase 3: Strategies & Refactoring (1-2 weeks)
- **#218** - Collar strategy
- **#219** - Protective Put strategy
- **#216** - Extract common strategy logic
- **#217** - Reduce clone() calls

### Phase 4: Advanced Pricing (2-3 weeks)
- **#240** - Lookback options
- **#245** - Spread options
- **#247** - Exchange options
- Remaining exotic pricing models

### Phase 5: Polish & Infrastructure (Optional)
- Testing improvements (#221, #222)
- Documentation (#224)
- Async support (#223)
- Error context (#220)

---

## Labels Reference

| Label | Description | Color |
|-------|-------------|-------|
| `priority-high` | Critical issues that should be addressed first | 🔴 Red |
| `priority-medium` | Important but not blocking | 🟡 Yellow |
| `priority-low` | Nice-to-have improvements | 🟢 Green |
| `refactor` | Code refactoring without changing behavior | 🔵 Blue |
| `enhancement` | New feature or request | 🩵 Cyan |
| `error-handling` | Related to error handling improvements | 🟠 Orange |
| `performance` | Performance optimization | 🌊 Teal |
| `testing` | Testing improvements | 💜 Purple |
| `documentation` | Documentation improvements | 📘 Blue |
| `strategies` | Related to trading strategies | 🔷 Blue |
| `pricing` | Related to options pricing | ⬜ Light Blue |

## Effort Estimates

| Effort | Hours |
|--------|-------|
| Low | 2-3 hours |
| Medium | 4-6 hours |
| High | 6-12 hours |

## Total Estimated Effort

- **Phase 1 (Stability)**: 12-18 hours
- **Phase 2 (Core Pricing)**: 30-50 hours
- **Phase 3 (Strategies)**: 18-28 hours
- **Phase 4 (Advanced Pricing)**: 24-40 hours
- **Phase 5 (Polish)**: 12-18 hours
- **Total**: ~96-154 hours

---

*Updated on: 2025-12-28*
