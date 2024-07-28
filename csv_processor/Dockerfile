FROM rust:1.79

# Set the working directory inside the container
WORKDIR /app

# Copy the Rust source code
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build the Rust application in release mode
RUN cargo build --release

RUN mkdir -p ./public

# Expose the port the app runs on
EXPOSE 8080

# Run the Rust application
ENTRYPOINT ["/app/target/release/csv_processor"]
