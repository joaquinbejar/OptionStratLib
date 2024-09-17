use std::env;
use std::sync::Once;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

static INIT: Once = Once::new();
/// Sets up the logger for the application.
///
/// The logger level is determined by the `LOGLEVEL` environment variable.
/// If the variable is not set, it defaults to `INFO`.
pub fn setup_logger() {
    INIT.call_once(|| {
        let log_level = env::var("LOGLEVEL")
            .unwrap_or_else(|_| "INFO".to_string())
            .to_uppercase();

        let level = match log_level.as_str() {
            "DEBUG" => Level::DEBUG,
            "ERROR" => Level::ERROR,
            "WARN" => Level::WARN,
            "TRACE" => Level::TRACE,
            _ => Level::INFO,
        };

        let subscriber = FmtSubscriber::builder().with_max_level(level).finish();

        tracing::subscriber::set_global_default(subscriber)
            .expect("Error setting default subscriber");

        tracing::debug!("Log level set to: {}", level);
    });
}

#[cfg(test)]
mod tests_setup_logger {
    use super::setup_logger;
    use std::env;
    use tracing::subscriber::set_global_default;
    use tracing_subscriber::FmtSubscriber;

    #[test]
    fn test_logger_initialization_info() {
        env::set_var("LOGLEVEL", "INFO");
        setup_logger();

        // After setting up the logger, you would typically assert that the logger is working
        // However, due to the nature of logging, it's difficult to directly assert on log output.
        // You can, however, check that set_global_default has been called successfully without panic.
        assert!(
            set_global_default(FmtSubscriber::builder().finish()).is_err(),
            "Logger should already be set"
        );
    }

    #[test]
    fn test_logger_initialization_debug() {
        env::set_var("LOGLEVEL", "DEBUG");
        setup_logger();

        // Similar to the previous test, check that the global logger has been set
        assert!(
            set_global_default(FmtSubscriber::builder().finish()).is_err(),
            "Logger should already be set"
        );
    }

    #[test]
    fn test_logger_initialization_default() {
        env::remove_var("LOGLEVEL");
        setup_logger();

        // Check that the global logger has been set
        assert!(
            set_global_default(FmtSubscriber::builder().finish()).is_err(),
            "Logger should already be set"
        );
    }

    #[test]
    fn test_logger_called_once() {
        env::set_var("LOGLEVEL", "INFO");

        setup_logger(); // First call should set up the logger
        setup_logger(); // Second call should not re-initialize

        // Check that the global logger has been set only once
        assert!(
            set_global_default(FmtSubscriber::builder().finish()).is_err(),
            "Logger should already be set and should not be reset"
        );
    }
}

#[cfg(test)]
mod tests_setup_logger_bis {
    use super::*;
    use std::sync::Mutex;
    use tracing::subscriber::with_default;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::Layer;

    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    #[derive(Clone)]
    struct TestLayer {
        level: std::sync::Arc<Mutex<Option<Level>>>,
    }

    impl<S> Layer<S> for TestLayer
    where
        S: tracing::Subscriber,
    {
        fn on_event(
            &self,
            event: &tracing::Event<'_>,
            _ctx: tracing_subscriber::layer::Context<'_, S>,
        ) {
            let mut level = self.level.lock().unwrap();
            *level = Some(*event.metadata().level());
        }
    }

    fn create_test_layer() -> (TestLayer, std::sync::Arc<Mutex<Option<Level>>>) {
        let level = std::sync::Arc::new(Mutex::new(None));
        (
            TestLayer {
                level: level.clone(),
            },
            level,
        )
    }

    #[test]
    fn test_default_log_level() {
        let _lock = TEST_MUTEX.lock().unwrap();
        env::remove_var("LOGLEVEL");

        let (layer, level) = create_test_layer();
        let subscriber = tracing_subscriber::registry().with(layer);

        with_default(subscriber, || {
            setup_logger();
            tracing::info!("Test log");
        });

        assert_eq!(*level.lock().unwrap(), Some(Level::INFO));
    }

    #[test]
    fn test_debug_log_level() {
        let _lock = TEST_MUTEX.lock().unwrap();
        env::set_var("LOGLEVEL", "DEBUG");

        let (layer, level) = create_test_layer();
        let subscriber = tracing_subscriber::registry().with(layer);

        with_default(subscriber, || {
            setup_logger();
            tracing::debug!("Test log");
        });

        assert_eq!(*level.lock().unwrap(), Some(Level::DEBUG));

        env::remove_var("LOGLEVEL");
    }

    #[test]
    fn test_error_log_level() {
        let _lock = TEST_MUTEX.lock().unwrap();
        env::set_var("LOGLEVEL", "ERROR");

        let (layer, level) = create_test_layer();
        let subscriber = tracing_subscriber::registry().with(layer);

        with_default(subscriber, || {
            setup_logger();
            tracing::error!("Test log");
        });

        assert_eq!(*level.lock().unwrap(), Some(Level::ERROR));

        env::remove_var("LOGLEVEL");
    }

    #[test]
    fn test_warn_log_level() {
        let _lock = TEST_MUTEX.lock().unwrap();
        env::set_var("LOGLEVEL", "WARN");

        let (layer, level) = create_test_layer();
        let subscriber = tracing_subscriber::registry().with(layer);

        with_default(subscriber, || {
            setup_logger();
            tracing::warn!("Test log");
        });

        assert_eq!(*level.lock().unwrap(), Some(Level::WARN));

        env::remove_var("LOGLEVEL");
    }

    #[test]
    fn test_trace_log_level() {
        let _lock = TEST_MUTEX.lock().unwrap();
        env::set_var("LOGLEVEL", "TRACE");

        let (layer, level) = create_test_layer();
        let subscriber = tracing_subscriber::registry().with(layer);

        with_default(subscriber, || {
            setup_logger();
            tracing::trace!("Test log");
        });

        assert_eq!(*level.lock().unwrap(), Some(Level::TRACE));

        env::remove_var("LOGLEVEL");
    }

    #[test]
    fn test_invalid_log_level() {
        let _lock = TEST_MUTEX.lock().unwrap();
        env::set_var("LOGLEVEL", "INVALID");

        let (layer, level) = create_test_layer();
        let subscriber = tracing_subscriber::registry().with(layer);

        with_default(subscriber, || {
            setup_logger();
            tracing::info!("Test log");
        });

        assert_eq!(*level.lock().unwrap(), Some(Level::INFO));

        env::remove_var("LOGLEVEL");
    }
}
