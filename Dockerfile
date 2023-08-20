FROM rust:latest

WORKDIR /usr/src/dewpoint
COPY . .

RUN cargo install --path .

CMD ["dewpoint"]
