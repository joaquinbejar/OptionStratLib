:::::::::::::: width-limiter
::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[pnl](index.html)
:::

# Trait [TransactionAble]{.trait}Copy item path

[[Source](../../src/optionstratlib/pnl/traits.rs.html#92-98){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub trait TransactionAble {
    // Required methods
    fn add_transaction(
        &mut self,
        transaction: Transaction,
    ) -> Result<(), TransactionError>;
    fn get_transactions(&self) -> Result<Vec<Transaction>, TransactionError>;
}
```

Expand description

::: docblock
## [§](#transactionable){.doc-anchor}TransactionAble

A trait that defines the ability to manage financial transactions within
an entity.

This trait provides a standardized interface for adding and retrieving
transaction records, enabling consistent transaction management across
different implementations.

### [§](#required-methods-1){.doc-anchor}Required Methods {#required-methods-1}

- `add_transaction`: Adds a new transaction to the implementing entity
- `get_transactions`: Retrieves all transactions from the implementing
  entity

### [§](#error-handling){.doc-anchor}Error Handling

Both methods return a `Result` type that may contain a
`TransactionError` if the operation fails. This allows for proper error
propagation and handling in transaction-related operations.
:::

## Required Methods[§](#required-methods){.anchor} {#required-methods .section-header}

::::::: methods
::: {#tymethod.add_transaction .section .method}
[Source](../../src/optionstratlib/pnl/traits.rs.html#94){.src
.rightside}

#### fn [add_transaction](#tymethod.add_transaction){.fn}( &mut self, transaction: [Transaction](struct.Transaction.html "struct optionstratlib::pnl::Transaction"){.struct}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.86.0/std/primitive.unit.html){.primitive}, [TransactionError](../error/struct.TransactionError.html "struct optionstratlib::error::TransactionError"){.struct}\> {#fn-add_transaction-mut-self-transaction-transaction---result-transactionerror .code-header}
:::

::: docblock
Adds a new transaction to the implementing entity.
:::

::: {#tymethod.get_transactions .section .method}
[Source](../../src/optionstratlib/pnl/traits.rs.html#97){.src
.rightside}

#### fn [get_transactions](#tymethod.get_transactions){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Transaction](struct.Transaction.html "struct optionstratlib::pnl::Transaction"){.struct}\>, [TransactionError](../error/struct.TransactionError.html "struct optionstratlib::error::TransactionError"){.struct}\> {#fn-get_transactionsself---resultvectransaction-transactionerror .code-header}
:::

::: docblock
Retrieves all transactions from the implementing entity.
:::
:::::::

## Implementors[§](#implementors){.anchor} {#implementors .section-header}

:::: {#implementors-list}
::: {#impl-TransactionAble-for-Position .section .impl}
[Source](../../src/optionstratlib/model/position.rs.html#665-673){.src
.rightside}[§](#impl-TransactionAble-for-Position){.anchor}

### impl [TransactionAble](trait.TransactionAble.html "trait optionstratlib::pnl::TransactionAble"){.trait} for [Position](../model/position/struct.Position.html "struct optionstratlib::model::position::Position"){.struct} {#impl-transactionable-for-position .code-header}
:::
::::
:::::::::::::
::::::::::::::
