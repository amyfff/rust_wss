use tower_governor::{
    // PERBAIKAN: Path import yang benar
    governor::GovernorConfigBuilder,
    key_extractor::SmartIpKeyExtractor,
    GovernorLayer, NoOpMiddleware,
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
    GovernorLayer { config: Box::leak(config) }
}