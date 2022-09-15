FROM rust as build
RUN set -ex; apt-get update; apt-get install -y --no-install-recommends jq; rm -rf /var/lib/apt/lists/*
RUN cargo install just
ARG COMMIT=main
RUN git clone https://github.com/scalliony/repo.git /repo
WORKDIR /repo
RUN git checkout ${COMMIT}
RUN just build-web && just server-build

FROM debian:stable-slim
COPY --from=build /repo/client/dist .
COPY --from=build /repo/target/release/scalliony-server .
CMD ["./scalliony-server"]
EXPOSE 3000/tcp
