// use tracing_subscriber::prelude::*;

pub fn init_sentry_guard() -> Option<sentry::ClientInitGuard> {
    let Some(dsn) = std::env::var("SENTRY_DSN").ok() else {
        return None;
    };

    let guard = sentry::init((
        dsn,
        sentry::ClientOptions {
            release: sentry::release_name!(),
            // Capture user IPs and potentially sensitive headers when using HTTP server integrations
            // see https://docs.sentry.io/platforms/rust/data-management/data-collected for more info
            send_default_pii: true,
            // Enable capturing of traces; set this a to lower value in production:
            // traces_sample_rate: 0.01,
            ..Default::default()
        },
    ));

    // tracing_subscriber::registry()
    //     .with(tracing_subscriber::fmt::layer())
    //     .with(sentry::integrations::tracing::layer())
    //     .init();

    Some(guard)
}
