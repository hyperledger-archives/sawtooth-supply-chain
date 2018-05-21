# Copyright 2018 Cargill Incorporated
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

FROM rust:1

RUN apt-get update && apt-get install -y unzip libzmq3-dev

# For Building Protobufs
RUN curl -OLsS https://github.com/google/protobuf/releases/download/v3.5.1/protoc-3.5.1-linux-x86_64.zip \
 && unzip protoc-3.5.1-linux-x86_64.zip -d protoc3 \
 && rm protoc-3.5.1-linux-x86_64.zip
RUN apt-get update && apt-get install -y protobuf-compiler

# Build TP with dummy source in order to cache dependencies in Docker image.
# Make sure not to use the `volumes` command to overwrite:
#   - /sawtooth-supply-chain/processor/target/
#   - /sawtooth-supply-chain/processor/src/messages/
WORKDIR /sawtooth-supply-chain
RUN USER=root cargo new --bin processor

WORKDIR /sawtooth-supply-chain/processor
COPY Cargo.toml Cargo.lock* ./
RUN cargo build

ENV PATH=$PATH:/sawtooth-supply-chain/processor/target/debug/

ENTRYPOINT ["/sawtooth-supply-chain/processor/target/debug/supply-chain-tp"]
