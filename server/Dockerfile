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

# Description:
#   Builds an image to be used when developing in JavaScript. The default CMD is to run
#   build_javascript.
#
# Build:
#   $ cd sawtooth-core/docker
#   $ docker build . -f sawtooth-dev-javascript -t sawtooth-dev-javascript
#
# Run:
#   $ cd sawtooth-core
#   $ docker run -v $(pwd):/project/sawtooth-core sawtooth-dev-javascript

FROM ubuntu:xenial

LABEL "install-type"="mounted"

# Install Node and Ubuntu dependencies
RUN apt-get update && apt-get install -y -q --no-install-recommends \
    curl \
    ca-certificates \
    pkg-config \
    build-essential \
    libzmq3-dev \
 && curl -s -S -o /tmp/setup-node.sh https://deb.nodesource.com/setup_6.x \
 && chmod 755 /tmp/setup-node.sh \
 && /tmp/setup-node.sh \
 && apt-get install nodejs -y -q \
 && rm /tmp/setup-node.sh \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/* \
 && npm install -g prebuild-install

WORKDIR /usr/src/supply-chain

# Install Dependencies
COPY asset_client/package.json asset_client/
RUN cd asset_client/ && npm install && cd ..

COPY fish_client/package.json fish_client/
RUN cd fish_client/ && npm install && cd ..

COPY server/package.json server/
RUN cd server/ && npm install && cd ..

# Build Clients
COPY . .

RUN cd asset_client/ && npm run build && cd ..
RUN cd fish_client/ && npm run build && cd ..

EXPOSE 3000/tcp
