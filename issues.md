## Fix Issues

1.

```error: failed to parse lock file at: /Users/{user}/{project_dir}/anchor/Cargo.lock
Caused by:
  lock file version 4 requires `-Znext-lockfile-bump
```

update version to 3 in Cargo.lock
https://github.com/solana-foundation/anchor/issues/3392

2.

```
error: package solana-program v1.18.0 cannot be built because it requires rustc 1.72.0 or newer, while the currently active rustc version is 1.68.0-dev
```

https://github.com/solana-labs/solana/issues/34987

3.

```
error[E0432]: unresolved import `crate`
 --> programs/staking/src/lib.rs:9:1
  |
9 | #[program]
  | ^^^^^^^^^^
  | |
  | unresolved import
  | help: a similar path exists: `crate::contexts::__client_accounts_claim_reward`
  |
```

issue is with not everything being imported in `programs/staking/src/lib.rs`. worked after adding use::contexts::\*;

4. the version issue wasn't resolved easily. here are the steps

- try `cargo update -p solana-program --precise 1.18.26`
- if that doesn't work, manually change the version in Cargo.toml of anchor-lang and anchor-spl to 0.29.0 and then run `cargo update -p solana-program --precise 1.18.26`
- finally we'd have to update version in Cargo.lock to 3
