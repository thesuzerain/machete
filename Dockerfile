FROM rust:1.81.0 as build
ENV PKG_CONFIG_ALLOW_CROSS=1

WORKDIR /usr/src/machete
# Download and compile deps
COPY . .
COPY docker_utils/dummy.rs /usr/src/machete/dummy.rs

WORKDIR  /usr/src/machete

# Change temporarely the path of the code
RUN sed -i 's|src/main.rs|dummy.rs|' Cargo.toml
# Build only deps
RUN cargo build --release
# Now return the file back to normal
RUN sed -i 's|dummy.rs|src/main.rs|' Cargo.toml

# Copy everything
# Add the wait script
ADD https://github.com/ufoscout/docker-compose-wait/releases/download/2.2.1/wait /wait
RUN chmod +x /wait
# Build our code
ARG SQLX_OFFLINE=true
RUN cargo build --release

FROM ubuntu:latest

RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*

RUN update-ca-certificates

COPY --from=build /usr/src/machete/target/release/machete /machete/machete
COPY --from=build /wait /wait
WORKDIR /machete

CMD /machete/machete