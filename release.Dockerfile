FROM rust:1.62 AS builder
COPY . /CONTEXT
WORKDIR /CONTEXT
RUN cargo build --release

FROM debian:buster-slim
COPY --from=builder /CONTEXT/Rocket.toml /Rocket.toml
COPY --from=builder /CONTEXT/target/release/front /front

#Exec form (vs shell form ie RUN /bin/bash ./front) does not invoke a command shell. Using exec lets you have a shell to return when you kill a running exec CMD.
#CMD ["./front"]

#docker run -p 8000:8000 --rm -it --name release_front front:release