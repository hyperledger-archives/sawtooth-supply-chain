# Copyright 2017 Intel Corporation
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
# ------------------------------------------------------------------------------

FROM hyperledger/sawtooth-shell:nightly

# Install Python, Node.js, and Ubuntu dependencies
RUN echo "deb http://repo.sawtooth.me/ubuntu/1.0/stable bionic universe" >> /etc/apt/sources.list \
  && (apt-key adv --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys 44FC67F19B2466EA \
  || apt-key adv --keyserver hkp://p80.pool.sks-keyservers.net:80 --recv-keys 44FC67F19B2466EA) \
  && apt-get update \
  && apt-get install -y -q \
    apt-transport-https \
    build-essential \
    ca-certificates \
    curl \
    libzmq3-dev \
    pkg-config \
    python3 \
    python3-colorlog \
    python3-dev \
    python3-grpcio-tools \
    python3-grpcio \
    python3-nose2 \
    python3-pip \
    python3-protobuf \
    python3-pytest-runner \
    python3-pytest \
    python3-sawtooth-sdk \
    python3-sawtooth-signing \
    python3-setuptools-scm \
    python3-yaml \
    software-properties-common \
  && curl -s -S -o /tmp/setup-node.sh https://deb.nodesource.com/setup_8.x \
  && chmod 755 /tmp/setup-node.sh \
  && /tmp/setup-node.sh \
  && apt-get install nodejs -y -q \
  && rm /tmp/setup-node.sh \
  && apt-get clean \
  && rm -rf /var/lib/apt/lists/* \
  && npm install -g prebuild-install

WORKDIR /sawtooth-supply-chain

# Install NPM dependencies to central location, link to individual components
COPY bin/splice_json bin/
COPY asset_client/package.json asset_client/
COPY fish_client/package.json fish_client/
COPY server/package.json server/

RUN \
 if [ ! -z $HTTP_PROXY ] && [ -z $http_proxy ]; then \
  http_proxy=$HTTP_PROXY; \
 fi; \
 if [ ! -z $HTTPS_PROXY ] && [ -z $https_proxy ]; then \
  https_proxy=$HTTPS_PROXY; \
 fi; \
 if [ ! -z $http_proxy ]; then \
  npm config set proxy $http_proxy; \
 fi; \
 if [ ! -z $https_proxy ]; then \
  npm config set https-proxy $https_proxy; \
 fi

RUN mkdir /node_deps \
  && bin/splice_json \
    asset_client/package.json \
    fish_client/package.json \
    server/package.json \
    > /node_deps/package.json \
  && cd /node_deps && npm install && cd - \
  && ln -s /node_deps/node_modules asset_client/ \
  && ln -s /node_deps/node_modules fish_client/ \
  && ln -s /node_deps/node_modules server/

ENV PATH $PATH:/sawtooth-supply-chain/bin

CMD ["tail", "-f", "/dev/null"]
