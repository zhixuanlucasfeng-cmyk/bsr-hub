FROM rust:1-bookworm AS build
WORKDIR /app
COPY . .
RUN cargo build --release -p core-api

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=build /app/target/release/core-api /usr/local/bin/core-api
ENV PORT=8080
EXPOSE 8080
USER 65532:65532
CMD ["core-api"]

