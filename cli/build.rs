// use blueprint_sdk::build;
use blueprint_sdk::tangle::blueprint;
use server_blueprint::server_start;
use server_blueprint::{BlueprintRequestParams, server_stop};
use std::path::Path;
use std::process;

fn main() {
    // Automatically update dependencies with `soldeer` (if available), and build the contracts.
    //
    // Note that this is provided for convenience, and is not necessary if you wish to handle the
    // contract build step yourself.
    // Temporarily skip contract building
    // let contracts_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
    //     .parent()
    //     .unwrap()
    //     .join("contracts");
    // 
    // let contract_dirs: Vec<&str> = vec![contracts_dir.to_str().unwrap()];
    // build::soldeer_install();
    // build::soldeer_update();
    // build::build_contracts(contract_dirs);

    println!("cargo::rerun-if-changed=../blueprint/src");

    // The `blueprint!` macro generates the info necessary for the `blueprint.json`.
    // See its docs for all available metadata fields.
    let blueprint = blueprint! {
        name: "server-blueprint",
        master_manager_revision: "Latest",
        manager: { Evm = "HelloBlueprint" },
        jobs: [server_start, server_stop],
        request_params: BlueprintRequestParams,
    };

    match blueprint {
        Ok(blueprint) => {
            // TODO: Should be a helper function probably
            let json = blueprint_sdk::tangle::metadata::macros::ext::serde_json::to_string_pretty(
                &blueprint,
            )
            .unwrap();
            std::fs::write(
                Path::new(env!("CARGO_WORKSPACE_DIR")).join("blueprint.json"),
                json.as_bytes(),
            )
            .unwrap();
        }
        Err(e) => {
            println!("cargo::error={e:?}");
            process::exit(1);
        }
    }
}
