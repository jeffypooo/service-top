FROM rust:latest
ENV TARGET_ARCH=aarch64-unknown-linux-gnu
WORKDIR /usr/src/app
#COPY . .
RUN apt-get update
RUN apt-get install -y gcc-aarch64-linux-gnu rsync
RUN rustup target install ${TARGET_ARCH}
#FROM ubuntu:20.04
#LABEL name="foo"
#COPY install-deps.sh .
#
#RUN chmod u+x install-deps.sh
#RUN ./install-deps.sh
