FROM rust:1.46-alpine
RUN adduser --disabled-password --uid 501 hugodutka
USER hugodutka
RUN cd /home/hugodutka && USER=hugodutka cargo new skclient
WORKDIR /home/hugodutka/skclient/
COPY Cargo* /home/hugodutka/skclient/
RUN cargo fetch
COPY src/ /home/hugodutka/skclient/src/
RUN cargo build
