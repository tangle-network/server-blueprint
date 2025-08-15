1. Start Tangle in a terminal.
2. Spawn the blueprint using the tangle CLI:

```shell
RUST_LOG=blueprint-rejection=trace,tangle-producer=debug,tangle-consumer=trace,blueprint-router=trace,blueprint-runner=trace,blueprint_manager=debug,blueprint_manager_bridge=debug,blueprint_auth=debug,axum=debug,server_blueprint=debug,server_blueprint_cli=debug cargo tangle debug spawn --no-vm -p server-blueprint --binary ./target/debug/server-blueprint-cli 0 0 --http-rpc-url http://localhost:9944 --ws-rpc-url ws://localhost:9944
```

3. In another terminal, request a new blueprint instance with the following payload (request args):

```json
[
  {
    "config": {
      "runtime": "docker",
      "package": "nginx:alpine",
      "args": [],
      "env": []
    }
  }
]
```

> **Note**: Port binding is now handled automatically by the blueprint. The server will receive a `PORT` environment variable and should bind to that port when possible.

```shell
cargo tangle blueprint request-service --blueprint-id 0 --keystore-uri ./target/keystore --value 0 --target-operators 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY --params-file ./examples/docker/nginx.json
```

> Note: this example uses the nginx server configuration in `./examples/docker/nginx.json`

4. Accept the request:

```shell
cargo tangle blueprint accept-request --request-id 0 --keystore-uri ./target/keystore
```

5. Now start the server by sending a job-call with the allowed owner ECDSA Key as bytes.

```shell
cargo tangle blueprint submit --blueprint-id 0 --service-id 0 --keystore-uri ./target/keystore --watcher --job 0 --params-file ./examples/legacy/alice_ecdsa.json
```

6. You should see the Server URL as the job output, now we need to generate an access token for the server. This can be done by executing the
   following js script in `generate-auth-token.ts`:

```shell
bun run generate-auth-token.ts
```

Once the token is generated, you can use it to access the deployed server.

7. You can now test the deployed server by accessing it directly via HTTP. For example, if you deployed nginx:

```bash
# Access the server with authentication token
curl -H "Authorization: ${ACCESS_TOKEN}" http://localhost:8276/

# Or use any HTTP client to interact with your deployed server
```

You can also test with your browser or Postman using the returned server URL.
