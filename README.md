
![Hyperledger Sawtooth](images/sawtooth_logo_light_blue-small.png)

# Sawtooth Supply Chain

This is a distributed application to help you trace the provenance and other
contextual information of any asset. It can be used as-is or customized for
different use cases. This distributed application runs on top of Hyperledger
Sawtooth, an enterprise blockchain. To learn more about Hyperledger Sawtooth
please see its
[sawtooth-core repo](https://github.com/hyperledger/sawtooth-core) or its
[published docs](https://sawtooth.hyperledger.org/docs/).

Running alongside the core Sawtooth components, Supply Chain includes a number
of custom components:

- a **transaction processor** which handles Supply Chain transaction logic
- a **server** which provides an HTTP/JSON API, syncs blockchain state to a
  local db, and serves example clients
- the **AssetTrack** example client for tracking generic assets
- the **FishNet** example client for tracking fish from catch to table
- a **shell** with the dependencies to run any commands and scripts

## Usage

This project utilizes [Docker](https://www.docker.com/what-docker) to simplify
dependencies and deployment. After cloning this repo, follow the instructions
specific to your OS to install and run whatever components are required to run
`docker` and `docker-compose` from your command line. This is only dependency
required to run Supply Chain components.

Now, with Docker installed and this repo cloned, from the root project
directory, simply run:

```bash
docker-compose up
```

This will take awhile the first time it runs, but when complete will be running
all required components in separate containers. Many of the components will be
available through HTTP endpoints, including:

- The Supply Chain REST API will be at **http://localhost:8020/api**
- AssetTrack will be at **http://localhost:8020/asset**
- FishNet will be at **http://localhost:8020/fish**
- RethinkDB's admin panel will be available at **http://localhost:8021**
- Sawtooth's blockchain REST API will be available at **http://localhost:8022**

In bash you can shutdown these components with the key combination: `ctrl-C`.
You can shutdown _and_ remove the containers (destroying their data), with the
command:

```bash
docker-compose down
```

### Using the Shell to Setup Web App Data

The example web apps require that their respective "RecordTypes" be sent to the
blockchain before they will run properly. This can be done from within the
Supply Chain Shell docker container. Open a new terminal and run:

```bash
docker exec -it supply-shell bash
```

Once inside the shell container you can navigate to the server directory and
run the necessary scripts:

```bash
cd server/
npm run make-asset
npm run make-fish
```

After the RecordTypes are created, from the same directory you can also run
scripts to seed some sample data:

```bash
npm run seed-sample-assets
npm run seed-sample-fish
```

If you have seeded the apps with sample data, you can also run scripts to send
updates over time:

```bash
npm run update-sample-assets
npm run update-sample-fish
```

You can customize how many updates are submitted per minute with the `RATE`
environment variable (default 6), and use `LIMIT` to stop the updates after a
certain number are submitted (default 25):

```bash
RATE=3 LIMIT=10 npm run update-sample-assets
```

If you just want to  exit the shell, you can simply run:

```bash
exit
```

### Configuring API Keys and Secrets

While the Server runs out of the box with sensible defaults, there are a number
of secrets and API keys which will not be secure unless set explicitly. While
this is fine for demo purposes, any actual deployment set the following
properties:

- **JWT_SECRET**: can be any random string
- **PRIVATE_KEY**: must be 64 random hexadecimal characters
- **MAPS_API_KEY**: provided by [Google Maps](https://developers.google.com/maps/documentation/javascript/get-api-key)

These properties can be set one of two ways, through an environment variable,
or (preferably) by creating a file named `config.json` file in the `server/`
directory. A file named `config.json.example` is provided which should provide
a template to follow.

## Development

The default Docker containers use the `volumes` command to link directly to the
source code on your host machine. As a result any changes you make will
immediately be reflected in Supply Chain components without having to rebuild
them. However, typically you _will_ have to restart a component before it can
take advantage of any changes. Rather than restarting every container, you can
restart a single component from separate terminal using the container name. For
example:

```bash
docker restart supply-server
```

The available container names include:
- supply-shell
- supply-processor
- supply-server
- supply-rethink
- supply-validator
- supply-settings-tp
- supply-rest-api

### Generating Protobuf and Client Files

Files in the `protos/` directory are used to generate classes for other
components. This is done automatically on `up`, but if you make changes to
these files and wish to rebuild the generated files immediately, you can do so
from within the Supply Chain Shell:

```bash
docker exec -it supply-shell bash
```

Once in the shell, you can generate the necessary Python classes simply by
running:

```bash
protogen
```

For the example clients, in addition to rebuilding them on Protobuf changes,
any changes to their source code will require their static files be rebuilt.
Fortunately the server does _not_ need to be restart in order to reflect
these changes, just rebuild the static files and refresh your browser (the
browser cache may have to be emptied):

```bash
cd asset_client/
npm run build
```

```bash
cd fish_client/
npm run build
```

## Documentation

The latest documentation for Sawtooth Supply Chain is available within this
repo in the [docs](docs) subdirectory.

## License

Hyperledger Sawtooth software is licensed under the
[Apache License Version 2.0](LICENSE) software license.

Hyperledger Sawtooth Supply Chain documentation in the [docs](docs)
subdirectory is licensed under a Creative Commons Attribution 4.0 International
License.  You may obtain a copy of the license at:
http://creativecommons.org/licenses/by/4.0/.

![Open Source Award Badge](images/rookies16-small.png)
