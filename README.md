# Torus Backend

### Running dev

This project uses [auto reloading](https://actix.rs/docs/autoreload/) in dev.

You must have `systemfd` and `cargo-watch` installed

```
cargo install systemfd cargo-watch
```

```
$ systemfd --no-pid -s http::8088 -- cargo watch -x run
```