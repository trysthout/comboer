### What is this?
A Rust node implemented based on the Erlang distributed protocol.

## How to run the examples
`cd crates/server/examples` to run the following example.

### Preparation

Running server or client depends on the epmd, so we need to start epmd first.
Find the path where epmd is located, then run the following command:

```bash
${epmd_path} -daemon
```

eg:

```bash
 /usr/lib64/erlang/erts-14.1.1/bin/epmd -daemon
```

### The rust node as a server

Please follow the steps below.

> Open a terminal run the following command to start a rust node.

``` bash
cargo run --example server
```

for TLS

```bash
cargo run --example server --features rustls
````

> Open a terminal run the following command to connect the rust node.

```bash
iex --sname a --cookie aaa  client.exs
```

for TLS

```bash
iex --sname a --cookie aaa --erl "-proto_dist inet_tls -ssl_dist_optfile `pwd`/ssl-rust.conf"  client.exs
````

### The rust node as a client

Please follow the steps below.

> Open a terminal run the following command to start a elixir node.

``` bash
iex --sname a --cookie aaa  server.exs
```

for TLS

```bash
 iex --sname a --cookie aaa --erl "-proto_dist inet_tls -ssl_dist_optfile `pwd`/ssl-rust.conf"  server.exs
```

> Open a terminal run the following command to connect the elixir node.

```bash
cargo run --example client
```

for TLS

```bash
cargo run --example client --features rustls
```
