# Simple Smart Wallet Transaction Relayer

## Motivation

This project should make it possible to support relaying of transaction with our without payment. The service doesn't use the build in refund logic of the Safe instead it allows to specify a fixed fee that should be paid via a multisend transaction.

## Quickstart

This project requires `rustup` and `redis`

- Clone project and go to project folder
- `rustup default nightly` (Rocket currently requires a nightly version)
- `cp .env.sample .env`
- `redis-server`
- `cargo run`

## Configuration

Rocket specific configurations (including databases) can be configured via the `Rocket.toml` for local development (see https://rocket.rs/v0.4/guide/configuration/#rockettoml).

For configurations specific to this service the `.env` file can be used. See next section.

## Environment

Place a `.env` file in the root of the project containing URL pointing to the environment in which you want the gateway to run.

The contents of the file should be the following (see `.env.sample` for an example):

```
TRANSACTION_SERVICE_URL=<Transaction service host>
``` 
