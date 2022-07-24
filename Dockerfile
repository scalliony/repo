FROM alpine/git as git
ARG COMMIT=main
RUN git clone https://github.com/scalliony/repo.git /repo
WORKDIR /repo
RUN git checkout ${COMMIT}

FROM node AS client
COPY --from=git /repo/client/ /
RUN npm run build

FROM rust AS server
COPY --from=git /repo/ /
RUN cargo build --release

FROM debian:stable-slim
COPY --from=client /dist .
COPY --from=server /target/release/scalliony-server .
CMD ["./scalliony-server"]
EXPOSE 3000/tcp
