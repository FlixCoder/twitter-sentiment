FROM docker.io/rust:latest as builder
ARG CARGO_NET_GIT_FETCH_WITH_CLI=true
ARG SQLX_OFFLINE=1

RUN apt update -yqq \
	&& apt install -yqq --no-install-recommends \
	build-essential cmake libssl-dev pkg-config git \
	&& rustup update \
	&& rustup toolchain add stable \
	&& rustup default stable

COPY . /app
WORKDIR /app
RUN cargo build --release

FROM debian:11-slim
RUN apt update && apt install ca-certificates -y
RUN mkdir -p /opt/app
WORKDIR /opt/app
COPY --from=builder /app/target/release/twitter-sentiment /usr/local/bin/twitter-sentiment
CMD ["/usr/local/bin/twitter-sentiment"]
