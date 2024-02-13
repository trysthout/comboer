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
iex --sname a --cookie aaa  server.exs
```
> Open a terminal run the following command to connect the elixir node.
```bash
cargo run --example client
```
