# Dockerfile for summoner/daw containerized headless render nodes (Step 1092)
FROM alpine:3.19 AS builder

RUN apk add --no-grad build-base cargo rust alsa-lib-dev

WORKDIR /app
COPY . .

RUN cargo build --release -p summon

FROM alpine:3.19
RUN apk add --no-cache alsa-lib libgcc libstdc++
COPY --from=builder /app/target/release/summon /usr/local/bin/summon

ENTRYPOINT ["summon"]
CMD ["render", "--help"]
