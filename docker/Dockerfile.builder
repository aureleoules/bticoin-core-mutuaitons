# Image used by worker in order to cache dependencies
FROM debian:bullseye

ENV DEBIAN_FRONTEND=noninteractive
RUN apt update && apt install -y build-essential libtool autotools-dev automake pkg-config bsdmainutils python3 libevent-dev libboost-dev libsqlite3-dev ccache git curl wget
RUN ccache -F 1000 && ccache -M 10G

# Initial build of the project to cache objects
RUN git clone https://github.com/bitcoin/bitcoin.git /tmp/bitcoin
RUN cd /tmp/bitcoin && \
    ./contrib/install_db4.sh `pwd` && \
    export BDB_PREFIX='/tmp/bitcoin/db4' && \
    ./autogen.sh && \
    ./configure --disable-fuzz --enable-fuzz-binary=no --with-gui=no --disable-zmq --disable-bench BDB_LIBS="-L${BDB_PREFIX}/lib -ldb_cxx-4.8" BDB_CFLAGS="-I${BDB_PREFIX}/include" && \
    make -j$(nproc)

WORKDIR /tmp/bitcoin

# Force run without cache (--build-arg CACHE_DATE="$(date)")
ARG CACHE_DATE
RUN echo $CACHE_DATE

RUN git pull origin master && make -j$(nproc)
