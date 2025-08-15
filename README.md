# Server Blueprint for Tangle Network 🌐

## 🚀 Project Overview & Purpose

This project is a blueprint for Tangle Network, designed to deploy and manage any HTTP server, database, or containerized application. When requesting a new instance of this blueprint, users get a new service instance with a specified configuration that enables seamless server deployment and management across multiple runtime environments.

Blueprints are specifications for <abbr title="Actively Validated Services">AVS</abbr>s on the Tangle Network. An AVS is an off-chain service that runs arbitrary computations for a user-specified period of time. This Server Blueprint provides a powerful abstraction for deploying and managing any type of server across different runtime environments, allowing developers to create reusable server infrastructures.

For more details about Tangle Network Blueprints, please refer to the [project documentation](https://docs.tangle.tools/developers/blueprints/introduction).

## ⚙️ Configuration Details

The blueprint supports configurations for various runtimes with automatic port management and direct server deployment:

### Supported Runtimes

- **JavaScript (bun runtime)**: Executes any JavaScript/Node.js applications using `bunx` with automatic bun installation if needed
- **Python (python3)**: Executes any Python applications using `uvx` with automatic uv installation if needed  
- **Docker containers**: Runs any containerized application with intelligent port discovery, automatic port allocation, and environment variable injection

### Port Management & Direct Deployment

The blueprint automatically manages port allocation for direct server deployment:

**Automatic Port Management:**

- Ports are automatically allocated using the system's available port detection
- The `PORT` environment variable is automatically injected into all servers
- Servers **should** bind to the port specified in the `PORT` environment variable for optimal compatibility
- No manual port configuration required in blueprint requests
- Docker containers have intelligent port discovery and mapping

**Direct Server Access:**

- HTTP-based communication for standard web servers and APIs
- Built-in authentication and authorization layer
- Automatic container lifecycle management  
- Support for environment variable injection

## 🔐 Authentication Workflow

The authentication workflow uses the script [`generate-auth-token.ts`](generate-auth-token.ts) to generate an access token through a challenge-response mechanism:

1. **Challenge Request**: Client sends public key and key type to `/v1/auth/challenge`
2. **Challenge Response**: Server returns a cryptographic challenge with expiration
3. **Signature Generation**: Client signs the challenge using their private key
4. **Token Verification**: Client submits signature to `/v1/auth/verify` endpoint
5. **Access Token**: Server returns a time-limited access token

Authentication is performed via an `Authorization` header when accessing the deployed server URL. The token format is `{token_id}|{token_string}`.

## 📋 Changelog & Breaking Changes

This is a generalized Server Blueprint intended for deploying arbitrary HTTP servers and services. See [`CHANGELOG.md`](CHANGELOG.md) for recent changes and examples.

**Key Features:**

- Automatic port allocation and management
- Support for any HTTP server, database, or containerized application
- Built-in authentication and security layer
- Multi-runtime support (Python, JavaScript, Docker)

## 🚀 Usage Examples & Demos

## 📁 Examples

Ready-to-use examples organized by runtime type. See **[`examples/`](examples/)** directory for full documentation.

### Quick Start Examples

```bash
# Web server (Docker - recommended)
cargo tangle blueprint request-service examples/docker/nginx.json

# Database (Docker)  
cargo tangle blueprint request-service examples/docker/postgres.json

# Python HTTP server
cargo tangle blueprint request-service examples/python/http-server.json

# JavaScript HTTP server
cargo tangle blueprint request-service examples/javascript/http-server.json
```

### Directory Structure

```
examples/
├── docker/          # 🐳 Docker containers (recommended)
│   ├── nginx.json
│   ├── postgres.json
│   └── redis.json
├── python/          # 🐍 Python packages
│   └── http-server.json
├── javascript/      # 🟨 JavaScript/Node.js packages  
│   └── http-server.json
└── legacy/          # 🗂️ Legacy examples (historical)
```

**👉 For detailed documentation and more examples, see [`examples/README.md`](examples/README.md)**

> **Note**: All servers are deployed directly without transport conversion. Port allocation is handled automatically by the blueprint.

### Local Setup

To run the setup locally, follow the detailed instructions in [`DEMO.md`](DEMO.md). The demo covers:

1. Starting Tangle network locally
2. Spawning the blueprint service
3. Requesting a new blueprint instance
4. Accepting the service request
5. Starting the server
6. Generating authentication tokens
7. Testing with HTTP clients (curl, browser, Postman)

### Internal Workflow

For a detailed understanding of the internal workflow, see the code in `blueprint/src/manager/` which provides a clear overview of runtime initialization and port management:

- User request flow and configuration processing
- Runtime detection and package management
- Transport conversion from STDIO to SSE
- Server initialization and endpoint provision
- Authentication and client connection handling

## 🔄 Workflow Description

The blueprint process follows this high-level workflow:

1. **Request Reception**: Receives a server deployment request with configuration parameters
2. **Configuration Processing**: Analyzes runtime type, package/image, and environment variables  
3. **Port Allocation**: Automatically allocates an available port and injects it as `PORT` environment variable
4. **Runtime Initialization**:
   - **Python**: Installs/uses `uv` for package management and execution
   - **JavaScript**: Installs/uses `bun` for package management and execution
   - **Docker**: Pulls images, inspects for exposed ports, and creates containers with intelligent port binding
5. **Server Deployment**: Launches the server/application in the specified runtime environment
6. **Endpoint Exposure**: Provides HTTP URL for direct server access
7. **Authentication**: Secures access through token-based authentication system  
8. **Client Interaction**: Enables clients to connect directly to the deployed server via HTTP

## 📋 Prerequisites

Before you can run this project, you will need to have the following software installed on your machine:

- [Rust](https://www.rust-lang.org/tools/install)
- [Forge](https://getfoundry.sh)
- [Node.js](https://nodejs.org/) (for running the authentication token generator)

You will also need to install [cargo-tangle](https://crates.io/crates/cargo-tangle), our CLI tool for creating and deploying Tangle Blueprints:

To install the Tangle CLI, run the following command:

> Supported on Linux, MacOS, and Windows (WSL2)

```bash
cargo install cargo-tangle --git https://github.com/tangle-network/blueprint
```

## ✨ Key Features

### Automatic Port Management

- **Zero Configuration**: No need to specify port bindings in configuration files
- **Conflict Prevention**: Automatic port allocation prevents port conflicts
- **Environment Injection**: `PORT` environment variable automatically provided to servers
- **Universal Compatibility**: Works across all runtime types (Python, JavaScript, Docker)
- **Intelligent Docker Handling**: Automatically discovers exposed ports from Docker images and configures port mapping only when needed

### Enhanced Security & Reliability

- **Simplified Configuration**: Reduced attack surface through automatic port management
- **Process Lifecycle**: Proper cleanup and container management
- **Error Handling**: Comprehensive error handling for port allocation failures

## 🛠️ Development

Once you have the prerequisites installed, you can build and deploy the project:

```sh
cargo build
```

to build the project, and

```sh
cargo tangle blueprint deploy
```

to deploy the blueprint to the Tangle network.

### Quick Reference for Server Developers

When developing servers for this blueprint:

1. **Read Port from Environment**: Use `process.env.PORT` (JS) or `os.environ['PORT']` (Python) when possible
2. **Bind to All Interfaces**: Use `0.0.0.0` as the host for container compatibility
3. **Docker EXPOSE**: For Docker images, use `EXPOSE` directive for automatic port discovery
4. **Environment Variables**: Access injected environment variables for configuration
5. **Test Locally**: Set `PORT=3000` environment variable for local testing

```bash
# Testing your server locally
export PORT=3000
your-server-command

# Docker example with port environment variable
docker run -e PORT=8080 -p 8080:8080 your-image
```

### How to Use the Fixtures

**For Docker Applications:**
- **Ready-to-use servers**: Use `nginx_server.json`, `postgres_database.json`, etc. - these work immediately
- **Your custom app**: Replace `package` field with your Docker image name (e.g., `"your-registry/your-app:v1.0"`)
- **Build your image**: Create a Dockerfile for your application, build it, and reference it in the config

**For Python/JavaScript Applications:**
- **Install packages**: The runtime will install and run the specified package using `uvx`/`bunx`
- **Your custom package**: Publish to npm/PyPI or use git URLs (e.g., `"git+https://github.com/user/repo.git"`)

**Example Dockerfile for Node.js:**
```dockerfile
FROM node:18-alpine
WORKDIR /app
COPY package*.json ./
RUN npm install
COPY . .
EXPOSE 3000
CMD ["node", "server.js"]
```

Then use: `"package": "your-registry/your-node-app:latest"`

## 📜 License

Licensed under either of

- Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## 📬 Feedback and Contributions

We welcome feedback and contributions to improve this blueprint.
Please open an issue or submit a pull request on our GitHub repository.
Please let us know if you fork this blueprint and extend it too!

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
