# just-ping

A ping server with a single purpose – to respond to any message with the "Ok" message.

## Test locally

1. Start the server

```sh
cargo run
```

2. Serve the demo app

```sh
servez -S ./example
```

3. Open in browser `https://localhost:8080`

## Build server

```
cargo build --all --release --target x86_64-unknown-linux-musl
```

## Run the server

```
./just-ping -cert=cert-file.pem -key=key-file.pem -port=7777
```