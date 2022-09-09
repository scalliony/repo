FROM rust as build
RUN cargo install just
RUN git clone https://github.com/scalliony/repo.git /repo
WORKDIR /repo
ARG COMMIT=main
RUN git checkout ${COMMIT}
# TODO: remove
RUN just build-wasm explorer
RUN just build-web

FROM debian:stable-slim
COPY --from=build /client/dist .
COPY --from=build /target/release/scalliony-server .
CMD ["./scalliony-server"]
EXPOSE 3000/tcp
