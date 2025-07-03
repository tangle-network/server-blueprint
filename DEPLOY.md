# Deployment Instructions to Live Testnet

This document provides step-by-step instructions to deploy and test the MCP Blueprint on a live testnet.

## Prerequisites

Before you begin, ensure you have the following:

- Tangle CLI installed (see [Tangle CLI Installation](https://github.com/tangle-network/blueprint#-option-2-install-from-source) for details)
- Bun installed, or Nodejs v22+
- Access to an sr25519 and ECDSA Keypair that have enough balance to pay for the deployment and testing

## Step 1: Clone the Repository

Start by cloning the MCP Blueprint repository to your local machine:

```bash
git clone https://github.com/tangle-network/mcp-blueprint.git
cd mcp-blueprint
```

## Step 2: Install Dependencies

Install the required dependencies using npm:

```bash
bun install
```

## Step 3: Build the Project

Build the project to ensure all components are compiled and ready for deployment:

```bash
cargo build --workspace
```

## Step 4: Verify the build

Ensure that the build was successful and all components are ready:

```bash
ls -la blueprint.json
```

You should see the `blueprint.json` file in the output, indicating that the build was successful.

## Step 5: Deploy the Blueprint

Deploy the MCP Blueprint to the live testnet using the Tangle CLI:

> [!NOTE]
> Make sure to replace `target/keystore` with the path to your keystore containing the sr25519 and ECDSA keypairs.
>
> If you do not have a keystore, you can create one using the Tangle CLI, see Section [Creating a Keystore](#creating-a-keystore) below.

```bash
cargo tangle blueprint deploy tangle --http-rpc-url https://testnet-rpc.tangle.tools --ws-rpc-url wss://testnet-rpc.tangle.tools -k target/keystore
```

## Step 6: Verify the Deployment

After deploying the blueprint, verify that it has been successfully deployed by checking the output of the deployment command. You should see a confirmation message indicating that the blueprint has been deployed.
And you can also list all deployed blueprints to verify:

```bash
cargo tangle blueprint list-blueprints --ws-rpc-url wss://testnet-rpc.tangle.tools
```

## Step 7: Register yourself as an Operator on the newly deployed MCP Blueprint

To register yourself as an operator on the newly deployed MCP Blueprint, you can use the Tangle CLI to call the `register` command. Make sure to replace `<blueprint_id>` with the actual ID of your deployed blueprint.

```bash
cargo tangle blueprint register --blueprint-id <blueprint_id> --keystore-uri ./target/keystore --ws-rpc-url wss://testnet-rpc.tangle.tools
```

> This may take a while to complete.

## Step 8: Request a new Instance of the MCP Blueprint

To request a new instance of the MCP Blueprint, use the Tangle CLI to call the `request-service` command. Again, replace `<blueprint_id>` with the actual ID of your deployed blueprint, your operator address, and ensure you have the correct parameters file ready, which should be in JSON format and contain the necessary parameters for the request.

> [!NOTE]
> For the `--params-file` option, you need to provide a path to a JSON file that contains the parameters for the request. This file should be structured according to the requirements of your MCP Blueprint. See [examples](./fixtures/) for sample parameter files.

```bash
cargo tangle blueprint request-service --blueprint-id <blueprint_id> --keystore-uri ./target/keystore --value 0 --target-operators <operator_address> --params-file <path_to_params_file> --ws-rpc-url wss://testnet-rpc.tangle.tools
```

## Step 9: Accept the Instance Request with your Operator account

To accept the instance request, you will need to use the Tangle CLI with your operator account. Make sure you have the keystore for your operator account ready and replace `<request_id>` with the actual ID of the request you want to accept.

```bash
cargo tangle blueprint accept-request --request-id <request_id> --keystore-uri ./target/keystore --ws-rpc-url wss://testnet-rpc.tangle.tools
```

## Step 10: Interact with the Instance

With the instance accepted, you can now interact with it. Use the Tangle CLI to call the `submit` command, which allows you to submit jobs to the instance. Make sure to replace `<blueprint_id>`, `<service_id>`, and `<job_id>` with the actual IDs of your blueprint, service, and job respectively.

```bash
cargo tangle blueprint submit --blueprint-id <blueprint_id> --service-id <service_id> --keystore-uri ./target/keystore --watcher --job <job_id> --params-file <path_to_params_file>
```

Example job submission command to start the MCP instance:

```bash
cargo tangle blueprint submit --blueprint-id 0 --service-id 0 --keystore-uri ./target/keystore --watcher --job 0 --params-file ./fixtures/alice_ecdsa.json
```

> [!TIP]
> You can use the `--watcher` flag to monitor the job status in real-time. This is useful for debugging and ensuring that your job is running as expected.

## Misc Instructions

### Creating a Keystore

If you do not have a keystore, you can create one using the Tangle CLI. Run the following command to create a new keystore:

```bash
cargo tangle key generate --key-type sr25519 --show-secret --output target/keystore
cargo tangle key generate --key-type ecdsa --show-secret --output target/keystore
```

This will generate a new keystore in the `target/keystore` directory containing both sr25519 and ECDSA keypairs.
Take note of the secret keys as you will need them to interact with the deployed blueprint later on. Once you get the secret keys, you can use any wallet to import them and send some balance to the addresses.

### List all accounts in the Keystore

To list all accounts in your keystore, you can use the following command:

```bash
cargo tangle key list --keystore-path ./target/keystore
```
