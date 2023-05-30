FROM ubuntu:22.04 AS quote-server-bulder

ARG DCAP_VERSION="1.16"

RUN apt-get update && apt-get install -y curl git vim build-essential  clang protobuf-compiler protobuf-c-compiler libprotobuf-c-dev libprotobuf-c1

RUN curl --proto '=https' --tlsv1.3 https://sh.rustup.rs -sSf > rustup.sh
RUN /bin/sh ./rustup.sh --profile minimal -y && rm ./rustup.sh

RUN curl https://download.01.org/intel-sgx/sgx-dcap/${DCAP_VERSION}/linux/distro/ubuntu22.04-server/sgx_debian_local_repo.tgz -o sgx_debian_local_repo.tgz
RUN tar zxvf sgx_debian_local_repo.tgz && \
    rm -f sgx_debian_local_repo.tgz

RUN echo "deb [trusted=yes arch=amd64] file:/sgx_debian_local_repo jammy main" |\
    tee /etc/apt/sources.list.d/sgx_debian_local_repo.list

RUN apt-get update && apt install -y libtdx-attest libtdx-attest-dev

COPY service/quote-server /quote-server
RUN cd /quote-server && /root/.cargo/bin/cargo build --release

# ======================================================================================================================

FROM ubuntu:22.04 as quote-server-base

ARG DCAP_VERSION="1.16"

ARG USERNAME=ccnp
ARG USER_UID=1000
ARG USER_GID=$USER_UID

RUN apt-get update && apt-get install -y curl git vim

RUN groupadd --gid $USER_GID $USERNAME \
    && useradd --uid $USER_UID --gid $USER_GID -m $USERNAME

RUN curl --proto '=https' --tlsv1.3 https://sh.rustup.rs -sSf > rustup.sh
RUN /bin/sh ./rustup.sh --profile minimal -y
RUN rm -f ./rustup.sh

RUN curl https://download.01.org/intel-sgx/sgx-dcap/${DCAP_VERSION}/linux/distro/ubuntu22.04-server/sgx_debian_local_repo.tgz -o sgx_debian_local_repo.tgz
RUN tar zxvf sgx_debian_local_repo.tgz && rm -f sgx_debian_local_repo.tgz
RUN echo "deb [trusted=yes arch=amd64] file:/sgx_debian_local_repo jammy main" | tee /etc/apt/sources.list.d/sgx_debian_local_repo.list
RUN apt-get update && apt install -y libtdx-attest libtdx-attest-dev

RUN apt-get -y clean && rm -rf /var/lib/apt/lists/*.

COPY --from=quote-server-bulder /quote-server/target/release/quote_server /bin

USER $USERNAME
EXPOSE 6789
CMD ["/bin/quote_server", "6789"]
