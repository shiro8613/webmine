FROM rust:latest as build
COPY . .
# Build
RUN cargo install --config net.git-fetch-with-cli=true --path .

FROM debian:bullseye-slim
# Copy executable
COPY --from=build /usr/local/cargo/bin/webmine /usr/local/bin/webmine
# Run a server
ENTRYPOINT [ "webmine" ]