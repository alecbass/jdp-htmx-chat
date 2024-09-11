ARG RUSTUP_TARGET=x86_64-unknown-linux-gnu

FROM rust:latest AS builder

ARG DATABASE_NAME=jdp-db.db
ARG API_ADDRESS=0.0.0.0:8000
ARG WEBSOCKET_ADDRESS=0.0.0.0:8001

# Know which architecture to build to
ARG RUSTUP_TARGET

# Add a user called "chat" to run the application
RUN useradd --create-home --shell /bin/bash chat

# Add all files in the custom /app directory
WORKDIR /app

# Add directories required by the application
ADD src /app/src
ADD static /app/static
ADD templates /app/templates
ADD macros /app/macros
ADD database /app/database
RUN touch /app/database/${DATABASE_NAME}

# Add configuration files
ADD Cargo.toml /app/Cargo.toml
ADD Cargo.lock /app/Cargo.lock

# Change all files to the application user
RUN chown -R chat:chat /app

# Swap over to the application user
USER chat

# Compile the application
RUN cargo build --release --target ${RUSTUP_TARGET}

# Strip debug symbols
RUN strip /app/target/${RUSTUP_TARGET}/release/jdp-chat-room

# Run a small Ubuntu build to run the program
FROM ubuntu:latest AS runtime

WORKDIR /app

# Know which target architecture we've built to
ARG RUSTUP_TARGET

# Copy the executable and other files required to run
COPY --from=builder /app/database /app/database
COPY --from=builder /app/static /app/static
COPY --from=builder /app/target/${RUSTUP_TARGET}/release/jdp-chat-room /app/jdp-chat-room

# The application runs on port 8050
EXPOSE 8050

# The websocket server runs on port 8001
EXPOSE 8001

# Run the application
CMD ["/app/jdp-chat-room"]
