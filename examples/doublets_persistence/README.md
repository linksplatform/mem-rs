# Persistent doublets store

This reproduces [issue #36](https://github.com/linksplatform/mem-rs/issues/36)
using doublets 0.5.0 and the local platform-mem implementation. It creates two
links, closes and reopens the store, verifies their contents, appends a link,
and verifies a second reopen. The temporary store is removed on exit.

Run from the repository root:

```sh
CARGO_BUILD_JOBS=1 cargo run --manifest-path examples/doublets_persistence/Cargo.toml
```

Expected output:

```text
wrote count=2
reopened count=2
reopened after append count=3
```

Before the fix, the first reopen reports `reopened count=0` and the assertion
fails. This is a separate Cargo package so the core library's test suite does
not need to depend on its consumer.
