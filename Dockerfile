FROM rust:latest

# Add a user called "chat" to run the application
RUN useradd --create-home --shell /bin/bash chat

# Add all files in the custom /app directory
WORKDIR /app

# Add directories required by the application
ADD src /app/src
ADD static /app/static
ADD templates /app/templates

# Add configuration files
ADD Cargo.toml /app/Cargo.toml
ADD Cargo.lock /app/Cargo.lock
ADD Rocket.toml /app/Rocket.toml

# Change all files to the application user
RUN chown -R chat:chat /app

# Swap over to the application user
USER chat

# Compile the application
RUN cargo build --release

# The application runs on port 8000
EXPOSE 8000

# Run the application
CMD ["cargo", "run", "--release"]
