FROM rust:1.91.1-bookworm AS builder

# Set the working directory inside the container
WORKDIR /learncast
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Build the dependencies without the actual source code to cache dependencies separately
RUN cargo build --release

# Now copy the source code
COPY src ./src

# Build your application
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