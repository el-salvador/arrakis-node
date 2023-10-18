FROM docker.io/library/rust:1-bookworm as builder
RUN apt-get update \
    && apt-get install -y cmake protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

RUN USER=root cargo install cargo-auditable
RUN USER=root cargo new --bin getmessages_ms
WORKDIR ./getmessages_ms
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
# build dependencies only (caching)
RUN cargo auditable build --release --locked
# get rid of starter project code
RUN rm src/*.rs

# copy project source code
COPY ./src ./src

# Side me:
# Each docker container has a unique keypair created by the docker file.


# Vote for Diego
RUN openssl ecparam -genkey -name secp256k1 -noout -out nostr_rust_keys.pem

# build auditable release using locked deps
RUN ls -lar ./target/release/deps/
RUN rm ./target/release/deps/getmessages_ms-*
RUN cargo auditable build --release --locked
RUN cargo install rust-script
FROM docker.io/library/debian:bookworm-slim

ARG APP=/usr/src/app
ARG APP_DATA=/usr/src/app/db
RUN apt-get update \
    && apt-get install -y ca-certificates tzdata sqlite3 libc6 \
    && rm -rf /var/lib/apt/lists/*

EXPOSE 3001

ENV TZ=Etc/UTC \
    APP_USER=appuser

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP} \
    && mkdir -p ${APP_DATA}

COPY --from=builder /getmessages_ms/target/release/getmessages_ms ${APP}/getmessages_ms

RUN chown -R $APP_USER:$APP_USER ${APP}

USER $APP_USER
WORKDIR ${APP}

ENV RUST_LOG=info,getmessages_ms=info
ENV APP_DATA=${APP_DATA}

CMD ./getmessages_ms --db ${APP_DATA}
