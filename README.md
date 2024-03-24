# DRACON

A RAFT implementation in Rust.

See the [documentation](https://lingkang.dev/dracon/) for more details.

## Libraries

### `logger`

A simple logger for the project.

[logger - Rust](https://lingkang.dev/dracon/logger/)

### `raft`

The RAFT implementation.

[raft - Rust](https://lingkang.dev/dracon/raft/)

### `rpc`

A simple remote procedure call (RPC) library for the project.

[rpc - Rust](https://lingkang.dev/dracon/rpc/)

There is an example of using the `rpc` library under the [`examples/ping`](./examples/ping) directory, basically simulating a ping service between different Docker containers. It can be run by the PowerShell script directly.

## Count the Lines of Code

``` TXT
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 Language            Files        Lines         Code     Comments       Blanks
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 Dockerfile              1           30           19            4            7
 Markdown                1          118            0           80           38
 PowerShell              1           89           59           13           17
 TOML                    6           73           64            0            9
───────────────────────────────────────────────────────────────────────────────
 Rust                    7          650          520           14          116
 |- Markdown             6          262            0          211           51
 (Total)                            912          520          225          167
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 Total                  16         1222          662          322          238
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

## Run with Docker containers

### 1. Build Docker Image

``` PowerShell
docker build . --file Dockerfile --tag dracon_img
```

### 2. Create Docker a Network

``` PowerShell
docker network create --driver bridge dracon_network
```

This will create a custom network called `dracon_network`.

Use `docker network rm dracon_network` to remove the network after use if needed.

### 3. Run Docker Containers

``` PowerShell
docker run -dit --name dracon1 --network dracon_network --rm -v ${PWD}:/proj -w /proj dracon_img
```

- `-d`: Run the container in the background.
- `-i`: Keep STDIN open even if not attached.
- `-t`: Allocate a pseudo-TTY.
- `--name dracon1`: Assign name as `dracon1` to the container.
- `--network dracon_network`: Connect the container to the `dracon_network`.
- `--rm`: Automatically remove the container when it exits.
- `-v ${PWD}:/proj`: Mount the current directory to the `/proj` directory in the container.
- `-w /proj`: Set the working directory to `/proj` in the container.
- `dracon_img`: The name of the image to run.

- `-p 8080:80`: Map port 80 in the container to port 8080 on the host machine.

### 4. Access the Container

``` PowerShell
docker exec -it dracon1 bash
```

This will open a bash shell in the `dracon1` container.

### 5. Get Containers' IP Addresses

``` PowerShell
docker inspect -f '{{range.NetworkSettings.Networks}}{{.IPAddress}}{{end}}' dracon1
```

This will return the IP address of the `dracon1` container.

``` PowerShell
docker network inspect dracon_network
```

This will display details of the `dracon_network` including the IP addresses of the containers connected to it.

> [!NOTE]
>
> TODO:
>
> Set up IPv6 for the containers.

### 6. Stop and Remove Containers

``` PowerShell
docker stop dracon1
```

As we added the `--rm` flag when running the container, it will be automatically removed when stopped.
