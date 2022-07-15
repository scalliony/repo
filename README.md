# Scalliony <!-- omit in toc -->

Opensource RTS programming game using WebAssembly

<!-- TODO: add cute screenshot -->

- [Built with](#built-with)
- [Monorepo](#monorepo)
- [Prerequisites](#prerequisites)
- [Usage](#usage)
  - [Authentication](#authentication)
  - [Log level](#log-level)
- [Docker](#docker)
- [Tests](#tests)
- [License](#license)


## Built with

* Rust
* TypeScript
* Axum
* Wasmtime
* Love and insomnia

## Monorepo

This project contains multiple folders:
- [/api](./api): Bot programming API
- [/bulb](./bulb): Shared types
- [/client](./client): Native game client
- [/engine](./engine): Core game logic
- [/server](./server): Game server
- [/sys](./sys): System binding
- [/web](./web): Web UI

## Prerequisites

* Rustup
* NPM

## Usage

- `git clone https://github.com/scalliony/repo.git`
- Copy and edit `.env.sample` to `.env`
- `npm start`
- Visit http://localhost:3000

### Authentication

Scalliony does not handle player's passwords but uses any compatible OAuth2 provider.
Default configuration in [.env.sample](./.env.sample) uses [Github](https://docs.github.com/en/developers/apps/building-oauth-apps/creating-an-oauth-app) configured using environment variables `AUTH_GITHUB_CLIENT_ID` and `AUTH_GITHUB_CLIENT_SECRET`.

Callback URL is `<domain>/auth/callback/github`. If your `<domain>` is not `http://localhost:3000`, you must defined `AUTH_BASE_URL` variable accordingly.

Also replace `AUTH_JWT_SECRET` value with a random secret !

### Log level

This project uses [RUST_LOG](https://rust-lang-nursery.github.io/rust-cookbook/development_tools/debugging/config_log.html) env variable for log level configuration. A good default is already in [.env.sample](./.env.sample).

## Docker

From DockerHub
```sh
docker run scalliony/server
```
[Dockerfile](Dockerfile) simply bundles following steps with a multi-stage image.
*No need to manually clone the full repo, copy just this Dockerfile*
```sh
docker build -t scalliony --build-arg COMMIT=<SHA> - < Dockerfile
docker run scalliony
```

## Tests

```
npm test
```

## License

Distributed under the MIT license to facilitate cooperation and knowledge sharing.
However, use with respect to contributors and end-users is strongly advised.
See [LICENSE](LICENSE) for more information.
