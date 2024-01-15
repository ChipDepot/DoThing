# Load rust image 
FROM rust:latest as builder

# Create new project
RUN USER=root cargo new --bin dothing
WORKDIR /dothing


# Copy the files
COPY ./dothing/Cargo.toml ./Cargo.toml
COPY ./dothing/src ./src

# Install cmake
RUN apt-get update
RUN apt-get install -y cmake

# Build the app with the release flag
RUN cargo build --release

# Create a lighter image using debian
FROM ubuntu:latest

RUN apt-get update && apt-get upgrade -y
RUN apt-get install -y openssl
RUN apt-get install -y build-essential

# Copy the bin
COPY --from=builder /dothing/target/release/dothing dothing

ENV RUST_LOG=dothing

# Run dothing
ENTRYPOINT [ "./dothing" ]

