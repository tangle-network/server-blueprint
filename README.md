# MCP Blueprint for Tangle Network 🌐

## � Project Overview & Purpose

This project is a blueprint for Tangle Network, designed to run remote MCPs (Model Context Protocol) services. When requesting a new instance of this blueprint, users get a new service instance with a specified configuration file that enables seamless MCP server deployment and management.

Blueprints are specifications for <abbr title="Actively Validated Services">AVS</abbr>s on the Tangle Network. An AVS is an off-chain service that runs arbitrary computations for a user-specified period of time. This MCP Blueprint provides a powerful abstraction for deploying and managing MCP servers across different runtime environments, allowing developers to create reusable MCP service infrastructures.

For more details about Tangle Network Blueprints, please refer to the [project documentation](https://docs.tangle.tools/developers/blueprints/introduction).

## ⚙️ Configuration Details

The blueprint supports configurations for various runtimes with automatic port management. All runtimes are internally converted to SSE (Server-Sent Events) for HTTP-based communication:

### Supported Runtimes

- **STDIO transport in JavaScript (bun runtime)**: Executes MCP servers using `bunx` with automatic bun installation if needed
- **STDIO transport in Python (python3)**: Executes MCP servers using `uvx` with automatic uv installation if needed
- **Docker containers**: Runs MCP servers in Docker containers with intelligent port discovery, automatic port allocation, and environment variable injection

### Port Management & Transport Conversion

The blueprint automatically manages port allocation and converts all runtime configurations to SSE transport:

**Automatic Port Management:**

- Ports are automatically allocated using the system's available port detection
- The `PORT` environment variable is automatically injected into all MCP servers
- MCP servers **must** bind to the port specified in the `PORT` environment variable
- No manual port configuration required in blueprint requests

**Transport Conversion:**

- HTTP-based communication for web clients
- Bidirectional message forwarding between STDIO and SSE transports
- Real-time streaming via Server-Sent Events
- POST endpoint for client message submission

## 🔐 Authentication Workflow

The authentication workflow uses the script [`generate-auth-token.ts`](generate-auth-token.ts) to generate an access token through a challenge-response mechanism:

1. **Challenge Request**: Client sends public key and key type to `/v1/auth/challenge`
2. **Challenge Response**: Server returns a cryptographic challenge with expiration
3. **Signature Generation**: Client signs the challenge using their private key
4. **Token Verification**: Client submits signature to `/v1/auth/verify` endpoint
5. **Access Token**: Server returns a time-limited access token

Authentication is performed via an `Authorization` header when accessing the MCP server URL. The token format is `{token_id}|{token_string}`.

## 📋 Changelog & Breaking Changes

**⚠️ Breaking Changes**: Recent updates introduce automatic port management. See [`CHANGELOG.md`](CHANGELOG.md) for detailed migration instructions and examples.

**Key Changes:**

- Removed `portBindings` field from configuration
- Added automatic `PORT` environment variable injection
- MCP servers must now read port from `PORT` environment variable

## 🚀 Usage Examples & Demos

### Sample Configurations

The [`fixtures`](fixtures) directory contains sample configurations for different runtime types:

- **[`fixtures/00_mcp_python3.json`](fixtures/00_mcp_python3.json)**: Python MCP server configuration
- **[`fixtures/01_mcp_js.json`](fixtures/01_mcp_js.json)**: JavaScript MCP server with Context7 package
- **[`fixtures/02_mcp_local_docker.json`](fixtures/02_mcp_local_docker.json)**: Local Docker MCP server with Redis environment
- **[`fixtures/03_tangle_mcp_docker.json`](fixtures/03_tangle_mcp_docker.json)**: Tangle-specific Docker MCP configuration

> **Note**: All sample configurations use the new format without `portBindings`. Port allocation is handled automatically by the blueprint.

### Local Setup

To run the setup locally, follow the detailed instructions in [`DEMO.md`](DEMO.md). The demo covers:

1. Starting Tangle network locally
2. Spawning the blueprint service
3. Requesting a new blueprint instance
4. Accepting the service request
5. Starting the MCP server
6. Generating authentication tokens
7. Testing with MCP Inspector

### Internal Workflow

For a detailed understanding of the internal workflow, see [`mcp-blueprint-flowchart.md`](mcp-blueprint-flowchart.md), which provides a comprehensive flowchart showing:

- User request flow and configuration processing
- Runtime detection and package management
- Transport conversion from STDIO to SSE
- Server initialization and endpoint provision
- Authentication and client connection handling

## 🔄 Workflow Description

The blueprint process follows this high-level workflow:

1. **Request Reception**: Receives a remote MCP request with configuration parameters
2. **Configuration Processing**: Analyzes runtime type, package/image, and environment variables
3. **Port Allocation**: Automatically allocates an available port and injects it as `PORT` environment variable
4. **Runtime Initialization**:
   - **Python**: Installs/uses `uv` for package management and execution
   - **JavaScript**: Installs/uses `bun` for package management and execution
   - **Docker**: Pulls images, inspects for exposed ports, and creates containers with intelligent port binding
5. **Transport Setup**: Converts STDIO communication to SSE for HTTP compatibility
6. **Endpoint Exposure**: Provides HTTP URL with `/sse` and `/message` endpoints
7. **Authentication**: Secures access through token-based authentication system
8. **Client Interaction**: Enables MCP clients to connect and communicate via HTTP/SSE

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
- **Environment Injection**: `PORT` environment variable automatically provided to MCP servers
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

### Quick Reference for MCP Server Developers

When developing MCP servers for this blueprint:

1. **Read Port from Environment**: Always use `process.env.PORT` (JS) or `os.environ['PORT']` (Python)
2. **Bind to All Interfaces**: Use `0.0.0.0` or `127.0.0.1` as the host
3. **No Port Configuration**: Remove any hardcoded ports from your server
4. **Test Locally**: Set `PORT=3000` environment variable for local testing

```bash
# Testing your MCP server locally
export PORT=3000
your-mcp-server-command
```

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
