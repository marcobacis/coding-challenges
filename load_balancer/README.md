# Coding Challenge - Build your own Load Balancer

[link to the challenge](https://codingchallenges.fyi/challenges/challenge-load-balancer)

The challenge is to build your own L7 (application layer) load balancer.

A load balancer sits in front of a group of servers (called *bakend* in this project) and routes request to them.
In this way, it minimises response time and maximise utilisation, ensuring no server is overloaded.

## Setup and Build

The project is implemented in Rust, using the [actix](https://actix.rs/) library to receive HTTP requests, and [Reqwest](https://docs.rs/reqwest/latest/reqwest/) to forward requests to the backends.

To build the server, run

```
cargo build --release
```

The binary will be in located at `target/release/lb`.

## How to run the load balancer

To run the load balancer, you first need a configuration file.
The configuration file is in a `.toml` file with this format: 

```toml
healthcheck_interval_secs = 30

[[backends]]
url = "http://127.0.0.1:8081"
healthcheck_path = "/health"

[[backends]]
url = "http://127.0.0.1:8082"
healthcheck_path = "/health"

```

Usage: 
```
lb [-p PORT] -c CONFIG_PATH
```

if no port is provided, the software will listen on port `8080`.


