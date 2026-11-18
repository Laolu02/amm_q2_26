# AMM — Solana Automated Market Maker

A Solana automated market maker (AMM) program built with **Anchor**, using a
constant-product pricing curve and SPL tokens. The program creates liquidity
pools for two tokens, issues liquidity-provider (LP) tokens, executes swaps,
collects trading fees, and allows the configured treasury to withdraw or burn
collected fees.

## How it works

Each pool holds reserves of **Token X** and **Token Y** in program-controlled
associated token accounts. The pool uses a constant-product curve to calculate
swap output. Liquidity providers deposit both assets and receive LP tokens
representing their proportional share of the pool. When they withdraw, their LP
tokens are burned and the corresponding share of both reserves is returned.

The pool configuration is a program-derived address (PDA). It controls the pool
vaults and the LP mint. The configured treasury receives swap fees in the input
token.

## Features

- Create a pool for a pair of SPL tokens.
- Store pool configuration and authority information in a PDA.
- Create Token X and Token Y vaults owned by the pool configuration.
- Create a six-decimal LP token mint controlled by the pool configuration.
- Deposit liquidity and mint LP tokens.
- Burn LP tokens and withdraw proportional liquidity.
- Swap in either direction using a constant-product curve.
- Enforce a minimum-output parameter for swaps.
- Collect swap fees in the input token and route them to treasury token accounts.
- Allow the configured treasury signer to withdraw collected fees to a recipient.
- Allow the configured treasury signer to burn collected fee tokens.
- Update the pool fee through the configured authority.
- Exercise instructions using LiteSVM integration tests.

## Architecture

### Pool state

The `Config` account stores:

| Field         | Description                                          |
| ------------- | ---------------------------------------------------- |
| `seed`        | User-supplied pool seed                              |
| `authority`   | Optional authority permitted to update the fee       |
| `treasury`    | Public key authorized to manage collected fees       |
| `mint_x`      | Token X mint                                         |
| `mint_y`      | Token Y mint                                         |
| `fee`         | Swap fee in basis points                             |
| `locked`      | Pool lock flag checked by liquidity and swap instructions |
| `config_bump` | PDA bump for the config account                      |
| `lp_bump`     | PDA bump for the LP mint                             |

The initializer must supply distinct token mints in the order required by the
program: `mint_x` must have a greater public-key value than `mint_y`. This
canonical ordering ensures a pool has one consistent mint ordering. Clients must
sort the mints before deriving and submitting initialization accounts.

### Pool configuration

The initializer supplies:

- A unique `seed` used to derive the pool config PDA.
- The two SPL token mints.
- A fee, expressed in basis points.
- A treasury public key.
- An optional fee-update authority.

The fee must be less than `10,000` basis points. For example:

| Fee   | Basis points |
| ----- | ------------ |
| 0.30% | 30           |
| 0.75% | 75           |
| 1.00% | 100          |

The program rejects a fee of `10,000` or greater.

## Instructions

The public instruction handlers are exposed from `src/lib.rs`.

### `initialize`

Creates the pool configuration PDA, LP mint, and the two pool vaults.

Inputs: `seed: u64`, `fee: u16`, `treasury: Pubkey`, `authority: Option<Pubkey>`

The LP mint is initialized with six decimal places and the config PDA as its
mint authority.

### `deposit`

Adds liquidity to the pool and mints LP tokens to the provider.

Inputs:

- `amount`: LP tokens requested
- `max_x`: maximum Token X the provider permits the program to take
- `max_y`: maximum Token Y the provider permits the program to take

The initial deposit path is used only when the LP supply **and** both vault
balances are zero; in that case `max_x` and `max_y` are taken as the initial
token amounts, so the first depositor sets the starting price and LP supply.
For all other deposits, the curve crate calculates the required proportional
amounts, and the instruction rejects deposits that exceed either maximum.

The user's LP token account is created if it does not exist.

### `withdraw`

Burns the provider's LP tokens and transfers their proportional share of Token X
and Token Y from the pool vaults.

Inputs:

- `amount`: LP tokens to burn
- `min_x`: minimum Token X expected
- `min_y`: minimum Token Y expected

The instruction rejects a withdrawal if either calculated output is below its
minimum.

### `swap`

Exchanges one pool token for the other.

Inputs:

- `is_x`: `true` when Token X is supplied; `false` when Token Y is supplied
- `amount_in`: gross input amount
- `min_amount_out`: minimum acceptable output amount

The program calculates the swap using the constant-product curve, transfers the
net input to the pool, routes the fee to the treasury, and transfers the output
token to the user.

The treasury's token accounts for X and Y are created on first use
(`init_if_needed`), with the swapping user paying the rent.

### `update_fee`

Updates the pool's swap fee.

Inputs: `new_fee: u16`

The signer must match the optional authority stored in the pool configuration.
The new fee must be below `10,000` basis points. If the pool was initialized with
`authority = None`, no signer can match and the fee can never be updated (calls
fail with `InvalidAuthority`).

### `withdraw_treasury`

Transfers collected fees from the treasury's token account to the recipient's
corresponding token account.

Inputs:

- `is_x`: selects the Token X or Token Y treasury balance
- `amount`: amount to transfer

The signer must be the treasury public key stored in the pool configuration. The
recipient's token accounts for **both** X and Y must already exist, even if only
one side is being withdrawn.

### `burn_treasury`

Burns collected fee tokens held by the treasury.

Input: `amount`: amount to burn

The mint supplied to the instruction (which must be `mint_x` or `mint_y`)
determines whether the program burns from the treasury's Token X or Token Y
account. Burning reduces both the treasury token-account balance and the
selected mint's total supply.

## Swap and fee mechanics

The pool uses a constant-product curve:

```
x * y = k
```

where `x` and `y` are the pool's token reserves and `k` is the invariant,
subject to integer arithmetic and fees.

The fee is denominated in basis points. With a fee of 30 basis points, the fee
rate is 0.30%.

For a gross input amount `amount_in`, the net amount credited to the pool is:

```
amount_net = floor(amount_in * (10_000 - fee) / 10_000)
```

The fee is the difference between the gross input and the net input:

```
fee_amount = amount_in - amount_net
```

Because the net amount is rounded down, the fee is effectively rounded up: a
very small swap can pay a fee of at least 1 unit.

The program transfers the net amount into the pool vault and transfers the fee
separately to the treasury. The `min_amount_out` argument protects the trader
against receiving less than their specified minimum.


> **Known issue — fee handling differs by direction.** In the crate's `swap()`,
> the X → Y path prices the trade on the *net* amount (after the fee), but the
> Y → X path prices it on the *gross* amount. For Y → X swaps the pool therefore
> pays out as if it had received the full input while actually receiving only
> the net amount, so `k` can decrease over time. Fixing this requires patching or
> replacing the curve crate.


## Build and test

```bash
cd amm-video
anchor build
cargo test
```

The integration tests use LiteSVM and create their test mints, pool, token
accounts, and transactions locally. They do not require a live validator or
funded devnet wallet.

## Test coverage

The current integration test suite covers:

- Pool initialization.
- Liquidity deposit and withdrawal.
- Swaps from X to Y and Y to X.
- Rejection of swaps whose minimum output is too high.
- Successful fee updates.
- Rejection of fees at or above 100%.
- Rejection of fee updates from an unauthorized signer.
- Treasury withdrawal of collected swap fees.
- Burning collected treasury fees and checking the token-account balance and
  mint supply.

Not currently covered: pool locking, initialization rejections (fee, treasury,
mint ordering), zero-fee or near-maximum-fee pools, very small swaps, and
invariant (`k`) checks across swap directions.
