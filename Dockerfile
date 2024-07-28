# Stage 1: Build the React frontend
FROM node:20 AS frontend-builder

# Set the working directory inside the container
WORKDIR /app/frontend

# Copy the package.json and yarn.lock or package-lock.json
COPY csv-processor-frontend/package.json csv-processor-frontend/package-lock.json ./

# Install dependencies
RUN npm install

# Copy the rest of the frontend code
COPY csv-processor-frontend/ ./

# Build the React app
RUN npm run build

# # Stage 2: Build the Rust backend
FROM rust:1.79 AS backend-builder

# # Set the working directory inside the container
WORKDIR /app/backend

# # Copy the Rust source code
COPY csv_processor/Cargo.toml csv_processor/Cargo.lock ./
COPY csv_processor/src ./src

# # Build the Rust application
RUN cargo build --release

# # Stage 3: Create the final image
FROM rust:slim

# Install required packages
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# # Set the working directory inside the container
WORKDIR /app

# # Copy the built frontend files
COPY --from=frontend-builder /app/frontend/build/ ./public

# Copy the Rust executable
COPY --from=backend-builder /app/backend/target/release/csv_processor .

# Expose the port the app runs on
EXPOSE 8080

# Set the entrypoint to the Rust application
ENTRYPOINT ["/app/csv_processor"]
