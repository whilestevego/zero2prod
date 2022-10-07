# We use the latest Rust stable release as base image
FROM rust:1.63.0 AS builder

# Let's switch our working directory to `app` (equivalent to `cd app`)
# The `app` folder will be created for us by Docker in case it does not
# exist already.
WORKDIR /app

# Install the required system dependencies for our linking configuration
RUN apt update && apt install lld clang -y

# Copy all files from our working environment to our Docker image
COPY . .

# Have sqlx use cached query definition artifacts sqlx-data.json
# which allows compilation without a connection to postgres
ENV SQLX_OFFLINE true

# Let's build our binary!
# We'll use the release profile to make it faaaast
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim AS runtime

WORKDIR /app

# Install OpenSSL - it is dynamically linked by some of our dependencies
# Install ca-certificates - it is needed to verify TLS certificates
# when establishing HTTPS connections
RUN apt-get update -y \
  && apt-get install -y --no-install-recommends openssl ca-certificates \
  && apt-get autoremove -y \
  && apt-get clean -y \
  && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/zero2prod zero2prod

COPY config config

ENV APP_ENV production

# When `docker run` is executed, launch the binary!
ENTRYPOINT ["./zero2prod"]