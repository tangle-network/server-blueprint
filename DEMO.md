1. Start Tangle in a terminal.
2. Spawn the blueprint using the tangle CLI:

```shell
RUST_LOG=blueprint-rejection=trace,tangle-producer=debug,tangle-consumer=trace,blueprint-router=trace,blueprint-runner=trace,blueprint_manager=debug,blueprint_manager_bridge=debug,blueprint_auth=debug,axum=debug,mcp_blueprint=debug,mcp_blueprint_cli=debug cargo tangle debug spawn --no-vm -p mcp-blueprint --binary ./target/debug/mcp-blueprint-cli 0 0 --http-rpc-url http://localhost:9944 --ws-rpc-url ws://localhost:9944
```

3. In another terminal, request a new blueprint instance with the following payload (request args):

```json
[
  {
    "config": {
      "runtime": "docker",
      "package": "tangle-mcp:0.1.0",
      "args": [],
      "env": [],
      "transportAdapter": "none"
    }
  }
]
```

> **Note**: Port binding is now handled automatically by the blueprint. The MCP server will receive a `PORT` environment variable and must bind to that port.

```shell
cargo tangle blueprint request-service --blueprint-id 0 --keystore-uri ./target/keystore --value 0 --target-operators 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY --params-file ./fixtures/03_tangle_mcp_docker.json
```

> Note: also this file is in `./fixtures/03_tangle_mcp_docker.json`

4. Accept the request:

```shell
cargo tangle blueprint accept-request --request-id 0 --keystore-uri ./target/keystore
```

5. Now start the MCP server by sending a job-call with the allowed owner ECDSA Key as bytes.

```shell
cargo tangle blueprint submit --blueprint-id 0 --service-id 0 --keystore-uri ./target/keystore --watcher --job 0 --params-file ./fixtures/alice_ecdsa.json
```

6. You should see the MCP Server url as the job output, now we need to generate an access token for the MCP server. This can be done by executing the
   following js script in `generate-auth-token.ts`:

```shell
bun run generate-auth-token.ts
```

Once the token is generated, you can use it to access the MCP server.

7. We are going to test the MCP server using `@modelcontextprotocol/inspector` utility.

```shell
npx -y @modelcontextprotocol/inspector
```

Choose the `SSE` transport and enter the MCP server URL you got from the job output + `/sse`. Then, enter the generated access token in the Authorization header.
