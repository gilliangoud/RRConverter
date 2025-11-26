# RRConverter

RRConverter is a tool for converting RaceReady passing data to a format that can be used by simple javascript web applications.

It's written in rust for a small footprint and fast performance.

## Features

- Connects automatically to a Rase|Result decoder on the local network
- hosts a simple websocket server that emits passing data in real time at /ws
- emits passing data in JSON format

## Websocket Data Format

```json
{
    "event": "passing",
    "data": {
        "lane": 1,
        "time": 123456789,
        "speed": 12.34
    }
}
```

## Running the sofware
Download the latest release from the [releases](https://github.com/gillian/rrconverter/releases) page and run it with:

```sh
./rrconverter
```

## Built on

- [tokio](https://tokio.rs)
- [warp](https://warp.rs)
- [serde](https://serde.rs)
- [tokio-util](https://docs.rs/tokio-util)
- [timing](https://github.com/critrace/timing)

## Dev Building instructions

```sh
cargo build --release
```

## Dev Usage

```sh
cargo run
```

