#[cfg(feature = "kafka")]
use rustchat::services::kafka_producer::KafkaProducer;
use rustchat::{api, config::Config, db, realtime::WsHub, storage::S3Client, telemetry};
use std::net::SocketAddr;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Install default rustls crypto provider (required for rustls 0.23+)
    // This must be done before any TLS connections are made
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls ring crypto provider");

    // Load environment from .env file if present
    dotenvy::dotenv().ok();

    // Load configuration
    let config = Config::load()?;

    // Initialize telemetry (logging/tracing)
    telemetry::init(&config.log_level);

    info!("Starting rustchat server v{}", env!("CARGO_PKG_VERSION"));

    // Connect to database and run migrations
    let db_pool = db::connect_with_config(&config.database_url, &config.db_pool).await?;
    info!("Database connected and migrations applied");

    // Seed admin user if configured
    if let (Some(admin_email), Some(admin_password)) = (&config.admin_user, &config.admin_password)
    {
        let user_exists = sqlx::query("SELECT 1 FROM users WHERE email = $1")
            .bind(admin_email)
            .fetch_optional(&db_pool)
            .await?;

        if user_exists.is_none() {
            info!("Creating initial admin user: {}", admin_email);
            let password_hash = rustchat::auth::hash_password(admin_password)?;
            let username = admin_email.split('@').next().unwrap_or("admin");

            sqlx::query(
                r#"
                INSERT INTO users (username, email, password_hash, role, is_active, display_name)
                VALUES ($1, $2, $3, 'system_admin', true, 'System Admin')
                "#,
            )
            .bind(username)
            .bind(admin_email)
            .bind(password_hash)
            .execute(&db_pool)
            .await?;

            info!("Admin user created successfully");
        } else {
            info!("Admin user already exists, skipping creation");
        }
    }

    // Create WebSocket hub
    let ws_hub = WsHub::new();
    info!("WebSocket hub initialized");

    // Initialize Redis Pool
    let redis_cfg = deadpool_redis::Config::from_url(&config.redis_url);
    let redis_pool = redis_cfg.create_pool(Some(deadpool_redis::Runtime::Tokio1))?;
    info!("Redis pool initialized");

    let cluster_broadcast = rustchat::realtime::ClusterBroadcast::new(
        redis_pool.clone(),
        config.redis_url.clone(),
        ws_hub.clone(),
    );
    match cluster_broadcast.start().await {
        Ok(()) => {
            ws_hub.set_cluster_broadcast(cluster_broadcast).await;
            info!("WebSocket cluster fan-out enabled");
        }
        Err(e) => {
            if config.require_cluster_fanout {
                anyhow::bail!(
                    "Failed to start websocket cluster fan-out and RUSTCHAT_REQUIRE_CLUSTER_FANOUT=true: {}",
                    e
                );
            }
            warn!(error = %e, "Failed to start websocket cluster fan-out; continuing in single-node mode");
        }
    }

    // Create S3 client
    let s3_client = S3Client::new(
        config.s3_endpoint.clone(),
        config.s3_public_endpoint.clone(),
        config.s3_bucket.clone(),
        config.s3_access_key.clone(),
        config.s3_secret_key.clone(),
        config.s3_region.clone(),
    );

    // Ensure S3 bucket exists - fail fast if storage is misconfigured
    match s3_client.ensure_bucket().await {
        Ok(()) => info!("S3 bucket verified/created successfully"),
        Err(e) => {
            // In production, storage is critical - fail startup
            if config.is_production() {
                anyhow::bail!(
                    "Failed to initialize S3 storage: {}. Check your S3 configuration.",
                    e
                );
            } else {
                // In dev, log warning but continue
                tracing::warn!("S3 bucket initialization failed (dev mode): {}", e);
            }
        }
    }

    // Spawn background jobs
    rustchat::jobs::spawn_retention_job(db_pool.clone());

    // Spawn email worker
    let email_worker_config = rustchat::jobs::EmailWorkerConfig::default();
    rustchat::jobs::spawn_email_worker(
        db_pool.clone(),
        email_worker_config,
        config.encryption_key.clone(),
    );

    // Generate node ID for this instance
    let node_id = format!(
        "{}-{}",
        hostname::get()
            .ok()
            .and_then(|h| h.into_string().ok())
            .unwrap_or_else(|| "unknown".to_string()),
        uuid::Uuid::new_v4()
    );

    // Initialize Kafka producer if enabled
    #[cfg(feature = "kafka")]
    let kafka_producer = if config.kafka.enabled {
        let producer = KafkaProducer::new(config.kafka.clone(), node_id.clone());
        if producer.is_available() {
            info!("Kafka producer initialized");
            Some(producer)
        } else {
            warn!("Kafka enabled but producer failed to initialize");
            None
        }
    } else {
        info!("Kafka integration disabled");
        None
    };

    #[cfg(not(feature = "kafka"))]
    let kafka_producer = None;

    // Start Kafka consumer for WebSocket fan-out if enabled
    #[cfg(feature = "kafka")]
    if config.kafka.enabled {
        let ws_hub_clone = ws_hub.clone();
        let kafka_config = config.kafka.clone();
        tokio::spawn(async move {
            if let Err(e) =
                rustchat::realtime::start_kafka_fanout_consumer(kafka_config, ws_hub_clone).await
            {
                tracing::error!(error = %e, "Failed to start Kafka fan-out consumer");
            }
        });
        info!("Kafka fan-out consumer started");
    }

    // Build application router (spawns reconciliation worker internally)
    let app = api::router(
        db_pool.clone(),
        redis_pool,
        config.jwt_secret.clone(),
        config.jwt_expiry_hours,
        ws_hub,
        s3_client,
        config.clone(),
        kafka_producer,
    );

    // Start server
    let addr: SocketAddr = format!("{}:{}", config.server_host, config.server_port)
        .parse()
        .expect("Invalid server address");

    info!("Listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}
