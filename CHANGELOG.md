# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Version 0.2.0-prerelease.2]

### Other

- Updated Blueprint SDK and other dependencies to latest versions

## [Version 0.2.0-prerelease.1]

### Added

- Automatic port allocation system via `next_available_port()` method
- Automatic `PORT` environment variable injection for all MCP servers
- Enhanced Docker container management with proper naming and restart policies
- Improved error handling for port allocation failures
- **NEW**: Docker image port discovery via `get_exposed_ports()` method
- **NEW**: Intelligent Docker port mapping that only configures bindings when images expose ports
- **NEW**: Enhanced Docker container `PORT` environment variable handling with container-internal port mapping

### Changed

- **BREAKING**: Removed `portBindings` field from MCP server configuration
- **BREAKING**: Port management is now handled automatically by the blueprint
- **BREAKING**: MCP servers must now read the port from the `PORT` environment variable
- Enhanced Docker container lifecycle management with proper cleanup
- Improved command execution with `kill_on_drop` for better process management

### Removed

- **BREAKING**: `port_bindings` field from [`McpServerConfig`](blueprint/src/lib.rs:38)
- **BREAKING**: Manual port binding configuration from fixture files
- **BREAKING**: `port_bindings` parameter from `McpRunner::start()` method

## Migration Guide

### For Configuration Files

**Before (v1.x):**

```json
{
  "config": {
    "runtime": "docker",
    "package": "tangle-mcp:0.1.0",
    "args": [],
    "portBindings": [[3000, 3000]],
    "env": [],
    "transportAdapter": "none"
  }
}
```

**After (v2.x):**

```json
{
  "config": {
    "runtime": "docker",
    "package": "tangle-mcp:0.1.0",
    "args": [],
    "env": [],
    "transportAdapter": "none"
  }
}
```

### For MCP Server Implementation

**Before (v1.x):**
MCP servers needed to manually bind to specified ports from configuration.

**After (v2.x):**
MCP servers **MUST** read the port from the `PORT` environment variable:

#### Python Example:

```python
import os
port = int(os.environ.get('PORT', 8000))
app.run(host='0.0.0.0', port=port)
```

#### JavaScript/Node.js Example:

```javascript
const port = process.env.PORT || 8000;
app.listen(port, "0.0.0.0");
```

#### Docker Example:

```dockerfile
# Your Dockerfile should use the PORT environment variable
EXPOSE $PORT
CMD ["your-app", "--port", "$PORT"]
```

### Transport Adapter Considerations

- **STDIO Transport**: No changes required. The blueprint automatically handles STDIO to SSE conversion.
- **SSE/HTTP Transport**: MCP servers **MUST** bind to the port specified in the `PORT` environment variable.
- **None Transport**: No network binding required, but the `PORT` environment variable is still provided.

### Runtime-Specific Changes

#### JavaScript Runtime (bun)

- The blueprint automatically sets the `PORT` environment variable
- Your MCP server should read `process.env.PORT` and bind to that port
- Example: `server.listen(process.env.PORT || 8000)`

#### Python Runtime (uvx)

- The blueprint automatically sets the `PORT` environment variable
- Your MCP server should read `os.environ['PORT']` and bind to that port
- Example: `app.run(port=int(os.environ.get('PORT', 8000)))`

#### Docker Runtime

- The blueprint automatically discovers exposed ports from your Docker image
- The blueprint sets the `PORT` environment variable to the container's internal exposed port
- The blueprint handles port binding from host to container automatically (only if ports are exposed)
- Your Docker container should expose and bind to the port from `$PORT`
- If no ports are exposed in your Docker image, no port mapping is configured
- Example Dockerfile:
  ```dockerfile
  # The blueprint will discover this exposed port automatically
  EXPOSE 8000
  ENV PORT=8000
  CMD ["python", "app.py", "--port", "$PORT"]
  ```

## Benefits of the Changes

1. **Simplified Configuration**: No need to manually specify port bindings
2. **Automatic Port Management**: Eliminates port conflicts through automatic allocation
3. **Improved Security**: Reduced surface area for configuration errors
4. **Better Resource Management**: Automatic cleanup and proper container lifecycle management
5. **Standardized Interface**: All MCP servers follow the same port configuration pattern
6. **Intelligent Docker Handling**: Automatic port discovery reduces configuration overhead and improves compatibility
7. **Flexible Port Mapping**: Only configures port bindings when Docker images actually expose ports

## Upgrading Your MCP Server

To ensure compatibility with the new version:

1. **Remove** any `portBindings` from your configuration files
2. **Update** your MCP server code to read the port from the `PORT` environment variable
3. **Test** your MCP server locally with the `PORT` environment variable set
4. **Verify** that your server binds to `0.0.0.0` or `127.0.0.1` on the specified port

### Testing Your Changes

You can test your updated MCP server locally:

```bash
# Set the PORT environment variable
export PORT=3000

# Run your MCP server
your-mcp-server-command

# Verify it's listening on the correct port
curl http://localhost:3000/health  # or appropriate endpoint
```
