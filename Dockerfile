FROM alpine/git as git
ARG COMMIT=main
RUN git clone https://github.com/scalliony/repo.git /repo
WORKDIR /repo
RUN git checkout ${COMMIT}

FROM node AS web
COPY --from=git /repo/web/ /
RUN npm run build

FROM rust AS server
COPY --from=git /repo/ /
RUN cargo build -p scalliony-server --release

FROM debian:stable-slim
COPY --from=web /dist .
COPY --from=server /target/release/scalliony-server .
CMD ["./scalliony-server"]
EXPOSE 3000/tcp
