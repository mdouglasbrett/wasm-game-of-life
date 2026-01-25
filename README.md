# WASM Game Of Life

## WIP:

This is the Rust/Wasm 'Game Of Life' project from a tutorial you can find here: [https://rustwasm.github.io/docs/book/](https://rustwasm.github.io/docs/book/).

The tutorial has been deprecated since I started this project, so I decided to try and build it in the most minimal way possible. At the moment I am building the `wasm` module manually using the `wasm32-unknown-unknown` target, and generating the JS glue code with `wasm-bindgen-cli`.

To run locally, make sure you have the `wasm-bindgen-cli` installed (see [this doc](https://wasm-bindgen.github.io/wasm-bindgen/reference/cli.html)). There are two helper scripts that I cribbed from C. Biffle's blogpost on building tiny wasm modules (https://cliffle.com/blog/bare-metal-wasm/). To build:

```sh
chmod +x build.sh && ./build.sh
```

To start the server, in the `client` directory run:

```sh
chmod +x server.py && ./server.py
```

