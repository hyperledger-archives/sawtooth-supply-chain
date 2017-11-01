
![Hyperledger Sawtooth](images/sawtooth_logo_light_blue-small.png)  

Hyperledger Sawtooth
-------------

Hyperledger Sawtooth is an enterprise solution for building, deploying, and
running distributed ledgers (also called blockchains). 
It provides an extremely modular and flexible platform for implementing 
transaction-based updates to shared state between
untrusted parties coordinated by consensus algorithms.

Getting Started with Sawtooth Supply Chain
-----------------

These instructions will enable you to launch a supply chain focused blockchain with web interface, and seed it with some sample assets.

Requirements:

Git - https://git-scm.com/download/mac

Docker - https://www.docker.com/docker-mac 

NPM - https://nodejs.org/en/download/ 

Python3 - https://www.python.org/downloads/mac-osx/ 

GRPC - $ python3 -m pip install grpcio 

GRPC Tools - $ python3 -m pip install grpcio-tools


Setup Instructions:

1. $ git clone https://github.com/hyperledger/sawtooth-supply-chain.git
2. Navigate to sawtooth-supply-chain folder
3. $ ./bin/protogen
4. $ docker-compose up
5. Open a new terminal
6. Navigate to sawtooth-supply-chain/server
7. $ npm install
8. $ npm run make-asset
9. $ npm run seed-sample-assets
10. Navigate your browser to localhost:3000/asset


Documentation
-------------

The latest documentation for Hyperledger Sawtooth is available within this repo in
the [docs](docs) subdirectory.

Documentation for our stable release is available at: 
http://intelledger.github.io/.

License
-------

Hyperledger Sawtooth software is licensed under the [Apache License Version 2.0](LICENSE) software license.

Hyperledger Sawtooth documentation in the [docs](docs) subdirectory is licensed under
a Creative Commons Attribution 4.0 International License.  You may obtain a copy of the
license at: http://creativecommons.org/licenses/by/4.0/.

![Open Source Award Badge](images/rookies16-small.png)
