FROM ubuntu:22.04

ARG DCAP_VERSION="1.16"

RUN apt-get update && apt-get install -y curl git vim

RUN curl --proto '=https' --tlsv1.3 https://sh.rustup.rs -sSf > rustup.sh
RUN /bin/sh ./rustup.sh --profile minimal -y

RUN curl https://download.01.org/intel-sgx/sgx-dcap/${DCAP_VERSION}/linux/distro/ubuntu22.04-server/sgx_debian_local_repo.tgz -o sgx_debian_local_repo.tgz
RUN tar zxvf sgx_debian_local_repo.tgz && rm -f sgx_debian_local_repo.tgz
RUN echo "deb [trusted=yes arch=amd64] file:/sgx_debian_local_repo jammy main" | tee /etc/apt/sources.list.d/sgx_debian_local_repo.list
RUN apt-get update && apt install -y libtdx-attest libtdx-attest-dev

COPY ./bin/quote_server /bin
EXPOSE 6789
CMD ["/bin/quote_server"]