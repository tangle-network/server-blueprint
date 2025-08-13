# Docker Examples

Docker containers are the **recommended approach** for deploying servers on Tangle.

## Why Docker?

✅ **Consistent Environment** - Works the same everywhere  
✅ **Wide Compatibility** - Any language, any framework  
✅ **Pre-built Images** - Use existing images from Docker Hub  
✅ **Isolated Dependencies** - No version conflicts  

## Available Examples

| Example | Description | Use Case |
|---------|-------------|----------|
| `nginx.json` | Nginx web server | Static websites, reverse proxy |
| `postgres.json` | PostgreSQL database | Relational database |
| `redis.json` | Redis cache | Caching, session storage |

## Usage

```bash
# Deploy nginx web server
cargo tangle blueprint request-service examples/docker/nginx.json

# Deploy PostgreSQL database  
cargo tangle blueprint request-service examples/docker/postgres.json

# Deploy Redis cache
cargo tangle blueprint request-service examples/docker/redis.json
```

## Common Docker Images

| Image | Description | Ports |
|-------|-------------|-------|
| `nginx:alpine` | Web server | 80 |
| `postgres:15-alpine` | Database | 5432 |
| `redis:7-alpine` | Cache | 6379 |
| `node:18-alpine` | Node.js runtime | - |
| `python:3.11-alpine` | Python runtime | - |

## Creating Your Own

1. **Find or build a Docker image**
2. **Create configuration:**
   ```json
   [
     {
       "config": {
         "runtime": "docker",
         "package": "your-image:tag",
         "args": [],
         "env": [
           ["ENV_VAR", "value"]
         ]
       }
     }
   ]
   ```
3. **Deploy:** `cargo tangle blueprint request-service your-config.json`
