FROM ubuntu:20.04 as builder

ENV DEBIAN_FRONTEND=noninteractive

# Install packages
RUN apt-get update  && apt-get -y upgrade
RUN apt-get install -y wget curl p7zip-full

# Install tracer
#  We copy from this machine since compiling it is quite difficult
#   Install PIN
RUN dpkg --add-architecture i386
RUN wget http://software.intel.com/sites/landingpage/pintool/downloads/pin-3.15-98253-gb56e429b1-gcc-linux.tar.gz
RUN tar xzf pin-3.15-98253-gb56e429b1-gcc-linux.tar.gz
RUN mv pin-3.15-98253-gb56e429b1-gcc-linux /opt
RUN export PIN_ROOT=/opt/pin-3.15-98253-gb56e429b1-gcc-linux
RUN echo -e "\nexport PIN_ROOT=/opt/pin-3.15-98253-gb56e429b1-gcc-linux" >> ~/.bashrc

# For this to work inside the container the docker needs to be run with --privileged
COPY Tracer/TracerPIN/ /TracerPIN
RUN cp -a /TracerPIN/Tracer /usr/local/bin
RUN	cp -a /TracerPIN/obj-* /usr/local/bin


# TracerGraph
RUN apt-get install -y build-essential qt5-qmake qtbase5-dev-tools qtbase5-dev libsqlite3-dev xvfb
COPY Tracer/TraceGraph/ /TraceGraph
COPY Tracer/TraceGraph/db.db /TraceGraph/db.sb
WORKDIR /TraceGraph
RUN qmake -qt=5
RUN make
RUN make install
# We test it
RUN Xvfb :1 -screen 0 1024x1024x16 & DISPLAY=:1 ./tracegraph db.db t.png 
# Send over telegram
RUN apt-get install -y curl



# Install wasi-sdk
COPY download_wasi_sdk.sh /download_wasi.sh
RUN bash /download_wasi.sh


WORKDIR /

# Copy our scripts
# Install python

# Install dtw_tools

# RUN wget -O dtw.gz https://github.com/Jacarte/RuSTRAC/releases/download/0.35/dtw-tools-x86_64-linux-0.35.gz
# RUN tar xf dtw.gz
# tar xfz do: z: uncompress, f: file, x: extract
# RUN mv dtw-tools* /usr/local/bin

# Install the collectors here
RUN apt-get install -y python3 python3-pip python2
# RUN python --version
COPY wbc/deadpool/deadpool_dca.py /wbc/deadpool/deadpool_dca.py
COPY wbc/deadpool/deadpool_dfa.py /wbc/deadpool/deadpool_dfa.py
COPY wbc/deadpool/deadpool_dfa_experimental.py /wbc/deadpool/deadpool_dfa_experimental.py
COPY wbc/ches2016/wb_challenge.wasm /wbc/ches2016/wb_challenge.wasm
COPY wbc/ches2016/trace_it.py /wbc/ches2016/trace_it.py

# Install daredevil here !
RUN apt-get install -y git clang cmake make libomp-dev --no-install-recommends
RUN git clone --recursive https://github.com/SideChannelMarvels/Daredevil.git
WORKDIR /Daredevil
RUN make
RUN make install

# Install binaryen
RUN git clone --recursive https://github.com/WebAssembly/binaryen.git /binaryen
WORKDIR /binaryen
# RUN git submodule init
# RUN git submodule update
RUN cmake . && make
RUN make install

# Install nodes

RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash -
RUN apt install -y nodejs
RUN node --version 

WORKDIR /


# Copy from previous image
# COPY --from=builder /go/src/github.com/alexellis/href-counter/app ./

