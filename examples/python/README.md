# Python Examples

Deploy Python applications using the `uvx` package manager.

## How It Works

1. **Package Installation** - Uses `uvx` to install Python packages
2. **Execution** - Runs your specified command with arguments
3. **Port Injection** - Automatically provides `PORT` environment variable

## Available Examples

| Example | Description | Package Used |
|---------|-------------|--------------|
| `http-server.json` | Simple HTTP server | Built-in `http.server` |

## Usage

```bash
# Deploy Python HTTP server
cargo tangle blueprint request-service examples/python/http-server.json
```

## Package Sources

### PyPI Packages
```json
{
  "config": {
    "runtime": "python",
    "package": "fastapi[all]",
    "args": ["python", "-m", "myapp"],
    "env": []
  }
}
```

### Git Repositories  
```json
{
  "config": {
    "runtime": "python", 
    "package": "git+https://github.com/user/python-app.git",
    "args": ["python", "app.py"],
    "env": []
  }
}
```

### Built-in Modules
```json
{
  "config": {
    "runtime": "python",
    "package": "python", 
    "args": ["-m", "http.server"],
    "env": []
  }
}
```

## Best Practices

1. **Use PORT environment variable:**
   ```python
   import os
   port = int(os.environ.get('PORT', 8000))
   app.run(host='0.0.0.0', port=port)
   ```

2. **Bind to all interfaces:** Use `0.0.0.0` not `localhost`

3. **Handle dependencies:** Include all dependencies in your package
