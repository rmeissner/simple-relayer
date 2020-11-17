# Simple Smart Wallet Transaction Relayer

## Motivation

This project should make it possible to support relaying of transaction with or without payment. The service doesn't use the built-in refund logic of the Safe contracts instead it allows to specify a fixed fee that should be paid via a multisend transaction. This way it should be easy to support other Smart Wallets that also support delegate calls.

## Quickstart

This project requires `rustup`

- Clone project and go to project folder
- `rustup default nightly` (Rocket currently requires a nightly version)
- `cp .env.sample .env`
- `cargo run`

## Configuration

For configurations specific to this service the `.env` file can be used.

## Heroku deployment

Note: make sure that config variables are set

- Login to heroku
  - `heroku login`
- Add heroku remote
  - `heroku git:remote -a <app name>`
- Setup herokue buildpack: https://github.com/emk/heroku-buildpack-rust
  - `heroku buildpacks:set emk/rust`
- Push to heroku
  - `git push heroku main`
