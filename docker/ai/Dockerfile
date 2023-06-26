# Build crust image
FROM ubuntu:20.04

RUN apt-get update
RUN apt-get install -y openssl
COPY ai /opt/ai/ai
COPY run.sh /opt/run.sh

WORKDIR /opt/ai/
CMD /opt/run.sh
