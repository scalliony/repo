# Scalliony <!-- omit in toc -->

Opensource RTS programming game using WebAssembly

<!-- TODO: add cute screenshot -->

- [Built with](#built-with)
- [Monorepo](#monorepo)
- [Prerequisites](#prerequisites)
- [Just play](#just-play)
- [Multiplayer](#multiplayer)
  - [Authentication](#authentication)
  - [Log level](#log-level)
- [Docker](#docker)
- [Tests](#tests)
- [License](#license)


## Built with

* Rust
* Axum
* Wasmtime
* Macroquad
* Love and insomnia

## Monorepo

This project contains multiple folders:
- [/api](./api): Bot programming API
- [/bulb](./bulb): Shared types
- [/client](./client): Game client
- [/engine](./engine): Core game logic
- [/server](./server): Game server
- [/sys](./sys): System binding

## Prerequisites

* Rustup
* [jq](https://stedolan.github.io/jq/)

## Just play

- Install [just](https://github.com/casey/just): `cargo install just`
- `git clone https://github.com/scalliony/repo.git`
- Play: `just play`

## Multiplayer

- Copy and edit `.env.sample` to `.env`
- `just play-web`
- Visit http://127.0.0.1:3000

### Authentication

Scalliony does not handle player's passwords but uses any compatible OAuth2 provider.
Default configuration in [.env.sample](./.env.sample) uses [Github](https://docs.github.com/en/developers/apps/building-oauth-apps/creating-an-oauth-app) configured using environment variables `AUTH_GITHUB_CLIENT_ID` and `AUTH_GITHUB_CLIENT_SECRET`.

Callback URL is `<domain>/auth/callback/github`. If your `<domain>` is not `http://127.0.0.1:3000`, you must defined `AUTH_BASE_URL` variable accordingly.

Also replace `AUTH_JWT_SECRET` value with a random secret !

### Log level

This project uses [RUST_LOG](https://rust-lang-nursery.github.io/rust-cookbook/development_tools/debugging/config_log.html) env variable for log level configuration. A good default is already in [.env.sample](./.env.sample).

## Docker

```sh
docker run scalliony/server
```
[Dockerfile](Dockerfile) simply bundles previous steps with a multi-stage image.
*No need to manually clone the full repo, copy just this Dockerfile*
```sh
docker build -t scalliony - < Dockerfile
docker run scalliony
```

## Tests

```
just test
```

## License

Distributed under the MIT license to facilitate cooperation and knowledge sharing.
However, use with respect to contributors and end-users is strongly advised.
See [LICENSE](LICENSE) for more information.
