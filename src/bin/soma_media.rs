// soma_media - BODY organ daemon
// Independent media preprocessing service accessible via Unix Domain Socket

use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};
use tracing::{debug, error, info, warn};
use tracing_subscriber::EnvFilter;

use soma_media::organ::{Organ, Stimulus, Response, MediaOrgan};

#[derive(Parser)]
#[command(name = "soma_media", version, about = "SOMA Media Daemon - FFmpeg Preprocessing Organ")]
struct Args {
    /// Unix socket path for UDS server
    #[arg(long, default_value = "/tmp/soma_media.sock")]
    socket_path: String,

    /// CardBus socket path for auto-registration
    #[arg(long, default_value = "/tmp/soma_card_bus")]
    cardbus_socket: String,

    /// Enable auto-registration with CardBus
    #[arg(long, default_value_t = true)]
    register_cardbus: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info"))
        )
        .init();

    let args = Args::parse();

    info!("🎬 Starting SOMA Media Daemon");
    info!("   Socket: {}", args.socket_path);

    // Track startup time for health checks
    let start_time = std::time::Instant::now();

    // Create media organ instance
    let organ = Arc::new(MediaOrgan::new());
    
    info!("   ✓ Media preprocessor initialized");

    // Remove old socket if exists
    let socket_path = PathBuf::from(&args.socket_path);
    if socket_path.exists() {
        std::fs::remove_file(&socket_path)
            .context("Failed to remove old socket")?;
    }

    // Create UDS listener
    let listener = UnixListener::bind(&socket_path)
        .context("Failed to bind Unix socket")?;
    
    info!("   ✓ Listening on {}", args.socket_path);

    // Register with CardBus if enabled
    if args.register_cardbus {
        let organ_clone = Arc::clone(&organ);
        let cardbus_socket = args.cardbus_socket.clone();
        tokio::spawn(async move {
            if let Err(e) = register_with_cardbus(&organ_clone, &cardbus_socket).await {
                warn!("CardBus registration failed: {}", e);
                warn!("Media will continue serving on UDS");
            }
        });
    }

    // Serve requests
    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                let organ = Arc::clone(&organ);
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(stream, organ, start_time).await {
                        error!("Connection error: {}", e);
                    }
                });
            }
            Err(e) => {
                error!("Accept error: {}", e);
            }
        }
    }
}

/// Handle a single UDS connection
async fn handle_connection(
    mut stream: UnixStream, 
    organ: Arc<MediaOrgan>,
    start_time: std::time::Instant,
) -> Result<()> {
    let mut buffer = vec![0u8; 65536]; // 64KB buffer

    loop {
        // Read request length (4 bytes)
        let mut len_buf = [0u8; 4];
        match stream.read_exact(&mut len_buf).await {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                debug!("Client disconnected");
                return Ok(());
            }
            Err(e) => return Err(e.into()),
        }

        let len = u32::from_be_bytes(len_buf) as usize;
        if len > buffer.len() {
            buffer.resize(len, 0);
        }

        // Read request body
        stream.read_exact(&mut buffer[..len]).await?;

        // Parse stimulus
        let stimulus: Stimulus = serde_json::from_slice(&buffer[..len])
            .context("Failed to parse stimulus")?;

        debug!("Received: op={}", stimulus.op);

        // Handle health check specially (no organ processing needed)
        let response = if stimulus.op == "health" || stimulus.op == "health.check" {
            Response {
                ok: true,
                output: serde_json::json!({
                    "status": "healthy",
                    "organ": "soma_media",
                    "version": env!("CARGO_PKG_VERSION"),
                    "uptime_ms": start_time.elapsed().as_millis() as u64,
                }),
                latency_ms: 0,
                cost: None,
            }
        } else {
            // Process via Organ trait
            match organ.stimulate(stimulus).await {
                Ok(resp) => resp,
                Err(e) => {
                    error!("Stimulate error: {:?}", e);
                    Response {
                        ok: false,
                        output: serde_json::json!({
                            "error": format!("{:?}", e)
                        }),
                        latency_ms: 0,
                        cost: None,
                    }
                }
            }
        };

        // Serialize response
        let response_bytes = serde_json::to_vec(&response)
            .context("Failed to serialize response")?;

        // Write response length + body
        let len_bytes = (response_bytes.len() as u32).to_be_bytes();
        stream.write_all(&len_bytes).await?;
        stream.write_all(&response_bytes).await?;
        stream.flush().await?;

        debug!("Sent: ok={}, latency={}ms", response.ok, response.latency_ms);
    }
}

/// CardBusMessage enum (MUST match nervous_system/src/ipc.rs EXACTLY)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
enum CardBusMessage {
    Register { organ_name: String, capabilities: serde_json::Value },
    Unregister { organ_name: String },
    Publish { topic: String, payload: Vec<u8> },
    Subscribe { topic: String },
    Request { target: String, operation: String, payload: Vec<u8> },
    Response { success: bool, payload: Vec<u8> },
    Ping,
    Pong,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Frame {
    msg_id: u64,
    message: CardBusMessage,
}

/// Register with CardBus nervous system
async fn register_with_cardbus(organ: &MediaOrgan, cardbus_socket: &str) -> Result<()> {
    // Wait for CardBus to be available
    for attempt in 1..=30 {
        if PathBuf::from(cardbus_socket).exists() {
            break;
        }
        if attempt == 30 {
            anyhow::bail!("CardBus socket not found after 30s: {}", cardbus_socket);
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    info!("🔌 Registering with CardBus: {}", cardbus_socket);

    // Connect to CardBus
    let mut stream = UnixStream::connect(cardbus_socket).await
        .context("Failed to connect to CardBus")?;

    // Get OrganCard as capabilities
    let card = organ.describe();
    let capabilities = serde_json::to_value(&card)
        .context("Failed to serialize OrganCard")?;

    // Create registration frame (matches nervous_system wire protocol)
    let frame = Frame {
        msg_id: 1,
        message: CardBusMessage::Register {
            organ_name: card.name.clone(),
            capabilities,
        },
    };

    let frame_bytes = serde_json::to_vec(&frame)?;
    let len_bytes = (frame_bytes.len() as u32).to_be_bytes();
    
    stream.write_all(&len_bytes).await?;
    stream.write_all(&frame_bytes).await?;
    stream.flush().await?;

    // Read response frame
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;
    
    let mut response_buf = vec![0u8; len];
    stream.read_exact(&mut response_buf).await?;

    let response_frame: Frame = serde_json::from_slice(&response_buf)
        .context("Failed to parse response frame")?;
    
    if let CardBusMessage::Response { success: true, .. } = response_frame.message {
        // Print beautiful storytelling logs
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        info!("🎬 {} v{}", card.name, card.version);
        info!("   Division: {} | Subsystem: {}", card.division, card.subsystem);
        info!("   {}", card.description);
        info!("   📦 Capabilities registered:");
        for func in card.functions.iter() {
            info!("      • {} - {}", func.name, func.description);
        }
        info!("   ✅ {} functions available on CardBus", card.functions.len());
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        // Keep connection alive for pub/sub messages
        maintain_cardbus_connection(stream).await;
        Ok(())
    } else {
        anyhow::bail!("Registration failed: {:?}", response_frame);
    }
}

/// Maintain persistent CardBus connection for pub/sub
async fn maintain_cardbus_connection(mut stream: UnixStream) {
    use tracing::debug;
    let mut buffer = vec![0u8; 65536];
    let mut ping_interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
    
    loop {
        tokio::select! {
            // Handle incoming messages
            result = async {
                let mut len_buf = [0u8; 4];
                stream.read_exact(&mut len_buf).await?;
                let len = u32::from_be_bytes(len_buf) as usize;
                
                if len > buffer.len() {
                    buffer.resize(len, 0);
                }
                
                stream.read_exact(&mut buffer[..len]).await?;
                Ok::<_, std::io::Error>(len)
            } => {
                match result {
                    Ok(len) => {
                        if let Ok(frame) = serde_json::from_slice::<Frame>(&buffer[..len]) {
                            debug!("Received CardBus message: {:?}", frame.message);
                        }
                    }
                    Err(e) => {
                        warn!("CardBus connection closed: {}", e);
                        break;
                    }
                }
            }
            
            _ = ping_interval.tick() => {
                let ping_frame = Frame {
                    msg_id: 0,
                    message: CardBusMessage::Ping,
                };
                
                if let Ok(ping_bytes) = serde_json::to_vec(&ping_frame) {
                    let len_bytes = (ping_bytes.len() as u32).to_be_bytes();
                    if stream.write_all(&len_bytes).await.is_err() ||
                       stream.write_all(&ping_bytes).await.is_err() ||
                       stream.flush().await.is_err() {
                        warn!("Failed to send ping, disconnecting");
                        break;
                    }
                    debug!("Sent CardBus ping");
                }
            }
        }
    }
    
    info!("CardBus connection terminated");
}
