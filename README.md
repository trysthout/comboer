
## What is this?
A Rust node implemented based on the Erlang distributed protocol.

## How to run the examples
`cd crates/server/examples` to run the following example.
### The rust node as a server
Please follow the steps below.

> Open a terminal run the following command  to start a rust node.
``` bash
cargo run --example server
```
> Open a terminal run the following command to connect the rust node.
```bash
iex --sname a --cookie aaa  client.exs
```


### The rust node as a client
Please follow the steps below.

> Open a terminal run the following command  to start a elixir node.
``` bash
iex --sname b --cookie aaa  server.exs
```
> Open a terminal run the following command to connect the elixir node.
```bash
cargo run --example client
```