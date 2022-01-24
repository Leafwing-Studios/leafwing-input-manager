# Running examples

You can run examples using `cargo run --example EXAMPLE_NAME`.

Start with the `minimal` example to get a basic sense of the plugin!

Note that you need three dependencies to use this crate succesfully:

1. `bevy`
2. `leafwing-input-manager`
3. `strum`, which handles iteration over enums
   1. The derive macro cannot be succesfully re-exported due to macro hygiene limitations :(

Sample `Cargo.toml` snippet:

```toml
[dependencies]
bevy = "0.6"
leafwing-input-manager = "0.1"
strum = {version = "0.23", features = ["derive"]}
```
