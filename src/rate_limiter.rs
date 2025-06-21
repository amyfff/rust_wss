use tower_governor::{
    governor::{middleware::NoOpMiddleware, GovernorConfigBuilder}, // MODIFIED IMPORT
    key_extractor::SmartIpKeyExtractor,
    GovernorLayer,
};

pub fn create_governor_layer() -> GovernorLayer<SmartIpKeyExtractor, NoOpMiddleware> {
    let config = Box::new(
        GovernorConfigBuilder::default()
            .per_second(30)
            .burst_size(15)
            .key_extractor(SmartIpKeyExtractor)
            .finish()
            .unwrap(),
    );
    GovernorLayer {
        config: Box::leak(config),
    }
}