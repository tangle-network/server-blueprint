# JavaScript Examples

Deploy JavaScript/Node.js applications using the `bunx` package manager.

## How It Works

1. **Package Installation** - Uses `bunx` to install npm packages
2. **Execution** - Runs your specified command with arguments  
3. **Port Injection** - Automatically provides `PORT` environment variable

## Available Examples

| Example | Description | Package Used |
|---------|-------------|--------------|
| `http-server.json` | Simple HTTP server | `http-server` npm package |

## Usage

```bash
# Deploy JavaScript HTTP server
cargo tangle blueprint request-service examples/javascript/http-server.json
```

## Package Sources

### npm Packages
```json
{
  "config": {
    "runtime": "javascript",
    "package": "express-generator", 
    "args": ["npx", "express-generator", "myapp"],
    "env": []
  }
}
```

### Git Repositories
```json
{
  "config": {
    "runtime": "javascript",
    "package": "git+https://github.com/user/node-app.git",
    "args": ["npm", "start"], 
    "env": []
  }
}
```

### Direct Commands
```json
{
  "config": {
    "runtime": "javascript",
    "package": "http-server",
    "args": ["-p", "$PORT", "-a", "0.0.0.0"],
    "env": []
  }
}
```

## Best Practices

1. **Use PORT environment variable:**
   ```javascript
   const port = process.env.PORT || 3000;
   app.listen(port, '0.0.0.0', () => {
     console.log(`Server running on port ${port}`);
   });
   ```

2. **Bind to all interfaces:** Use `0.0.0.0` not `localhost`

3. **Handle dependencies:** Ensure your package.json includes all dependencies
