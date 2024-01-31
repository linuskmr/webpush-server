# webpush-server

A server for registering [Web Push](https://web.dev/articles/push-notifications-overview) endpoints and sending pushes using the [Push API](https://developer.mozilla.org/en-US/docs/Web/API/Push_API), written in Rust.

> Currently work in progress


## Usage

Clone this repository and [install Rust and Cargo](https://rustup.rs):

Install OpenSSL header files needed for compilation:

```
sudo apt-get install libssl-dev
```

### Push Application Keys

As described by <https://docs.rs/web-push/0.10.1/web_push/struct.VapidSignatureBuilder.html>

Generate a application private key pair:

```
openssl ecparam -name prime256v1 -genkey -noout -out private.pem
```

Derive the public key from the private key:

```
openssl ec -in private.pem -pubout -out vapid_public.pem
```

To get the byte form of the public key for the JavaScript client:

```
openssl ec -in private.pem -text -noout -conv_form uncompressed
```

...or a base64 encoded string, which the client should convert into byte form before using:

```
openssl ec -in private.pem -pubout -outform DER|tail -c 65|base64|tr '/+' '_-'|tr -d '\n'
```

### Running

```
cargo run
```

Set logging to the debug level:

```
RUST_LOG=webpush_server=debug cargo run
```