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
#   Builds server and client node dependencies, creating a server image
#   which can be run with root context

FROM ubuntu:bionic

LABEL "install-type"="mounted"

RUN apt-get update \
 && apt-get install gnupg -y

# Install Node and Ubuntu dependencies
RUN apt-get update && apt-get install -y -q --no-install-recommends \
    curl \
    ca-certificates \
    pkg-config \
    build-essential \
    libzmq3-dev \
 && curl -s -S -o /tmp/setup-node.sh https://deb.nodesource.com/setup_8.x \
 && chmod 755 /tmp/setup-node.sh \
 && /tmp/setup-node.sh \
 && apt-get install nodejs -y -q \
 && rm /tmp/setup-node.sh \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/* \
 && npm install -g prebuild-install

WORKDIR /sawtooth-supply-chain/server

COPY server/package.json .
RUN npm install

#Copy client sample data for script use
COPY asset_client/sample_data/ ../asset_client/sample_data/
COPY fish_client/sample_data/ ../fish_client/sample_data/

COPY protos/ ../protos/
COPY server/ .

EXPOSE 3000/tcp

CMD ["/usr/bin/node", "index.js"]
