
![Hyperledger Sawtooth](images/sawtooth_logo_light_blue-small.png)  

Sawtooth Supply Chain
-----------------

This is a distributed application to help you trace the provenance and other contextual information of any asset.
 It can be used as-is or customized for different usages.
This supply chain dApp runs on top of Hyperledger Sawtooth, an enterprise blockchain. 
To learn more about Hyperledger Sawtooth please see its [sawtooth-core repo](https://github.com/hyperledger/sawtooth-core)
 or its [published docs](https://sawtooth.hyperledger.org/docs/).

The scripts below will help you run the entire blockchain locally using containers. 

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
2. Navigate to sawtooth-supply-chain directory 
3. $ ./bin/protogen
4. $ docker-compose up
5. Open a new terminal
6. Navigate to sawtooth-supply-chain/server
7. $ npm install
8. $ npm run make-asset
9. $ npm run seed-sample-assets
10. Navigate your browser to localhost:3000/asset

Optionally run data feeds to update the assets (watch maps and graphs update in the browser):
11. $ npm run update-sample-assets

Shutdown Instructions:
1. Navigate to sawtooth-supply-chain directory 
2. $ docker-compose down

Subsequent Runs:
1. $ docker-compose up
2. Open a new terminal
3. Navigate to sawtooth-supply-chain/server
4. $ npm run make-asset
5. $ npm run seed-sample-assets
6. Navigate your browser to localhost:3000/asset
7. $ npm run update-sample-assets

Documentation
-------------

The latest documentation for Sawtooth Supply Chain is available within this repo in
the [docs](docs) subdirectory.

License
-------

Hyperledger Sawtooth software is licensed under the [Apache License Version 2.0](LICENSE) software license.

Hyperledger Sawtooth Supply Chain documentation in the [docs](docs) subdirectory is licensed under
a Creative Commons Attribution 4.0 International License.  You may obtain a copy of the
license at: http://creativecommons.org/licenses/by/4.0/.

![Open Source Award Badge](images/rookies16-small.png)
