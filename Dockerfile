FROM --platform=$BUILDPLATFORM alpine:latest as certs
RUN apk --update add ca-certificates

FROM --platform=$BUILDPLATFORM messense/rust-musl-cross:aarch64-musl AS build-arm64

FROM --platform=$BUILDPLATFORM messense/rust-musl-cross:x86_64-musl AS build-amd64

FROM build-${TARGETARCH} AS build
COPY ./src ./src
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release

FROM scratch
COPY --from=certs /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=build /home/rust/src/target/*/release/bugcrowd-webhook-manager /
USER 1000
ENV RUST_LOG WARN
EXPOSE 3000/tcp
CMD ["./bugcrowd-webhook-manager"]
