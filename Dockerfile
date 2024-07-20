# Use the official Rust image from the Docker Hub
FROM rust:latest

# Set the working directory inside the container
WORKDIR /usr/src/ln-address-server

# Copy the current directory contents into the container at /usr/src/myapp
COPY . .

# Build the Rust project
RUN cargo build --release --verbose

# Run the binary program produced by the build process
CMD ["./target/release/ln-address"]
