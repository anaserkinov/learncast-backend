FROM rust:1.91.1-bookworm AS builder

WORKDIR /learncast
RUN mkdir src && \
    echo "fn main() {}" > src/dummy.rs

COPY Cargo.toml Cargo.lock build.rs ./
RUN sed -i 's#src/main.rs#src/dummy.rs#' Cargo.toml

RUN cargo build --release

RUN sed -i 's#src/dummy.rs#src/main.rs#' Cargo.toml
RUN rm src/dummy.rs

COPY src ./src

RUN cargo build --release
RUN strip target/release/learncast

FROM debian:bookworm-slim

# Install FFmpeg
RUN apt-get update && \
    apt-get install -y --no-install-recommends ffmpeg ca-certificates && \
    rm -rf /var/lib/apt/lists/* && \
    apt-get clean

# Set the working directory
WORKDIR /learncast

# Copy the built binary from the previous stage
COPY --from=builder /learncast/target/release/learncast .

# Command to run the application
CMD ["./learncast"]