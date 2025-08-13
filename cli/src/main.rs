use blueprint_sdk::Job;
use blueprint_sdk::Router;
use blueprint_sdk::contexts::tangle::TangleClientContext;
use blueprint_sdk::crypto::sp_core::SpSr25519;
use blueprint_sdk::crypto::tangle_pair_signer::TanglePairSigner;
use blueprint_sdk::keystore::backends::Backend;
use blueprint_sdk::runner::BlueprintRunner;
use blueprint_sdk::runner::config::BlueprintEnvironment;
use blueprint_sdk::runner::tangle::config::TangleConfig;
use blueprint_sdk::tangle::consumer::TangleConsumer;
use blueprint_sdk::tangle::filters::MatchesServiceId;
use blueprint_sdk::tangle::layers::TangleLayer;
use blueprint_sdk::tangle::producer::TangleProducer;
use server_blueprint::{SERVER_START_JOB_ID, SERVER_STOP_JOB_ID, MyContext, server_start, server_stop};
use tower::filter::FilterLayer;
use tracing::error;
use tracing::level_filters::LevelFilter;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    setup_log();

    let env = BlueprintEnvironment::load()?;
    blueprint_sdk::info!("Starting server blueprint...");
    blueprint_sdk::info!("Version: {}", env!("CARGO_PKG_VERSION"));
    let sr25519_signer = env.keystore().first_local::<SpSr25519>()?;
    let sr25519_pair = env.keystore().get_secret::<SpSr25519>(&sr25519_signer)?;
    let sr25519_signer = TanglePairSigner::new(sr25519_pair.0);

    let tangle_client = env.tangle_client().await?;
    let tangle_producer =
        TangleProducer::finalized_blocks(tangle_client.rpc_client.clone()).await?;
    let tangle_consumer = TangleConsumer::new(tangle_client.rpc_client.clone(), sr25519_signer);

    let tangle_config = TangleConfig::default();

    let service_id = env.protocol_settings.tangle()?.service_id.unwrap();
    let ctx = MyContext::new(env.clone()).await?;
    let result = BlueprintRunner::builder(tangle_config, env.clone())
        .router(
            Router::new()
                .route(SERVER_START_JOB_ID, server_start.layer(TangleLayer))
                .route(SERVER_STOP_JOB_ID, server_stop.layer(TangleLayer))
                .layer(FilterLayer::new(MatchesServiceId(service_id)))
                .with_context(ctx),
        )
        .producer(tangle_producer)
        .consumer(tangle_consumer)
        .with_shutdown_handler(async { println!("Shutting down!") })
        .run()
        .await;

    if let Err(e) = result {
        error!("Runner failed! {e:?}");
    }

    Ok(())
}

pub fn setup_log() {
    use tracing_subscriber::util::SubscriberInitExt;

    let _ = tracing_subscriber::fmt::SubscriberBuilder::default()
        .without_time()
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::NONE)
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .finish()
        .try_init();
}
