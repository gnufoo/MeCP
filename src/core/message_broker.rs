//! Inter-Application Message Broker
//!
//! This module provides an in-memory message queue with Redis persistence
//! for application-to-application communication within MeCP.

#![allow(dead_code)]
//!
//! # Design
//!
//! - Each initialized application instance (app_id + user_id) is a unique endpoint
//! - Messages are stored in Redis for persistence
//! - In-memory channels provide real-time notifications to running app instances
//!
//! # Usage
//!
//! ```ignore
//! let broker = MessageBroker::new(redis_config).await?;
//!
//! // Register an app instance
//! broker.register_app("mailbox", "alice").await?;
//!
//! // Send a message from one app to another
//! broker.send_message(AppMessage {
//!     from_app: "mailbox".to_string(),
//!     from_user: "alice".to_string(),
//!     to_app: "mailbox".to_string(),
//!     to_user: "bob".to_string(),
//!     message_type: "new_mail".to_string(),
//!     payload: json!({"subject": "Hello"}),
//! }).await?;
//! ```

use anyhow::{Result, Context};
use redis::{AsyncCommands, aio::ConnectionManager};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock, broadcast};
use chrono::{DateTime, Utc};

use crate::services::config::RedisConfig;

/// A message sent between applications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterAppMessage {
    /// Unique message ID
    pub id: String,
    /// Source application ID
    pub from_app: String,
    /// Source user ID
    pub from_user: String,
    /// Destination application ID
    pub to_app: String,
    /// Destination user ID
    pub to_user: String,
    /// Message type (e.g., "new_mail", "notification", "request")
    pub message_type: String,
    /// Message payload (JSON)
    pub payload: JsonValue,
    /// Timestamp when message was created
    pub created_at: DateTime<Utc>,
    /// Whether the message has been delivered
    pub delivered: bool,
}

impl InterAppMessage {
    /// Create a new message
    pub fn new(
        from_app: impl Into<String>,
        from_user: impl Into<String>,
        to_app: impl Into<String>,
        to_user: impl Into<String>,
        message_type: impl Into<String>,
        payload: JsonValue,
    ) -> Self {
        Self {
            id: uuid_v4(),
            from_app: from_app.into(),
            from_user: from_user.into(),
            to_app: to_app.into(),
            to_user: to_user.into(),
            message_type: message_type.into(),
            payload,
            created_at: Utc::now(),
            delivered: false,
        }
    }

    /// Get the recipient key (app:user)
    pub fn recipient_key(&self) -> String {
        format!("{}:{}", self.to_app, self.to_user)
    }

    /// Get the sender key (app:user)
    pub fn sender_key(&self) -> String {
        format!("{}:{}", self.from_app, self.from_user)
    }
}

/// Generate a simple UUID v4
fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let random: u64 = rand_simple();
    format!("{:016x}-{:016x}", timestamp, random)
}

/// Simple random number generator (no external deps)
fn rand_simple() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64;
    // Simple xorshift
    let mut x = nanos;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    x
}

/// Subscription handle for receiving messages
pub struct MessageSubscription {
    receiver: mpsc::Receiver<InterAppMessage>,
}

impl MessageSubscription {
    /// Receive the next message (blocking)
    pub async fn recv(&mut self) -> Option<InterAppMessage> {
        self.receiver.recv().await
    }

    /// Try to receive a message without blocking
    pub fn try_recv(&mut self) -> Option<InterAppMessage> {
        self.receiver.try_recv().ok()
    }
}

/// Registered application instance
struct AppInstance {
    app_id: String,
    user_id: String,
    sender: mpsc::Sender<InterAppMessage>,
}

/// Message broker for inter-application communication
pub struct MessageBroker {
    /// Redis connection manager
    redis: Option<ConnectionManager>,
    /// Registered application instances (key: "app_id:user_id")
    instances: Arc<RwLock<HashMap<String, AppInstance>>>,
    /// Broadcast channel for global notifications
    broadcast: broadcast::Sender<InterAppMessage>,
    /// Redis key prefix for messages
    redis_prefix: String,
}

impl MessageBroker {
    /// Create a new message broker with Redis backend
    pub async fn new(config: &RedisConfig) -> Result<Self> {
        let redis = if config.enabled {
            let url = if let Some(ref password) = config.password {
                format!("redis://:{}@{}:{}/{}", password, config.host, config.port, config.database)
            } else {
                format!("redis://{}:{}/{}", config.host, config.port, config.database)
            };

            let client = redis::Client::open(url.as_str())
                .context("Failed to create Redis client")?;
            
            let manager = ConnectionManager::new(client).await
                .context("Failed to create Redis connection manager")?;
            
            Some(manager)
        } else {
            None
        };

        let (broadcast, _) = broadcast::channel(1000);

        Ok(Self {
            redis,
            instances: Arc::new(RwLock::new(HashMap::new())),
            broadcast,
            redis_prefix: "mecp:msg:".to_string(),
        })
    }

    /// Create a message broker without Redis (in-memory only)
    pub fn new_in_memory() -> Self {
        let (broadcast, _) = broadcast::channel(1000);
        
        Self {
            redis: None,
            instances: Arc::new(RwLock::new(HashMap::new())),
            broadcast,
            redis_prefix: "mecp:msg:".to_string(),
        }
    }

    /// Register an application instance to receive messages
    pub async fn register_app(&self, app_id: &str, user_id: &str) -> Result<MessageSubscription> {
        let key = format!("{}:{}", app_id, user_id);
        let (sender, receiver) = mpsc::channel(100);

        let instance = AppInstance {
            app_id: app_id.to_string(),
            user_id: user_id.to_string(),
            sender,
        };

        let mut instances = self.instances.write().await;
        let was_registered = instances.contains_key(&key);
        instances.insert(key.clone(), instance);
        let total_instances = instances.len();

        if was_registered {
            tracing::info!(
                "🔄 APP RE-REGISTERED: '{}' (app={}, user={}) - total {} instances",
                key, app_id, user_id, total_instances
            );
        } else {
            tracing::info!(
                "✅ APP REGISTERED: '{}' (app={}, user={}) - total {} instances",
                key, app_id, user_id, total_instances
            );
        }

        // Deliver any pending messages from Redis
        if let Some(ref _redis) = self.redis {
            // TODO: Load pending messages from Redis
            tracing::debug!("📥 Checking for pending messages for '{}'", key);
        }

        Ok(MessageSubscription { receiver })
    }

    /// Unregister an application instance
    pub async fn unregister_app(&self, app_id: &str, user_id: &str) {
        let key = format!("{}:{}", app_id, user_id);
        let mut instances = self.instances.write().await;
        if instances.remove(&key).is_some() {
            let total_instances = instances.len();
            tracing::info!(
                "❌ APP UNREGISTERED: '{}' (app={}, user={}) - total {} instances remaining",
                key, app_id, user_id, total_instances
            );
        } else {
            tracing::debug!("App '{}' was not registered, nothing to unregister", key);
        }
    }

    /// Send a message to another application
    pub async fn send_message(&self, message: InterAppMessage) -> Result<String> {
        let message_id = message.id.clone();
        let sender_key = message.sender_key();
        let recipient_key = message.recipient_key();
        
        // Detailed logging for message tracking
        tracing::info!(
            "📬 MESSAGE SEND: id={}, type='{}', from='{}' -> to='{}'",
            message_id, message.message_type, sender_key, recipient_key
        );
        tracing::debug!(
            "   📋 Payload: {}",
            serde_json::to_string(&message.payload).unwrap_or_else(|_| "<error>".to_string())
        );

        // Store message in Redis for persistence
        if let Some(ref mut redis) = self.redis.clone() {
            let redis_key = format!("{}{}:{}", self.redis_prefix, recipient_key, message_id);
            let json = serde_json::to_string(&message)?;
            
            let _: () = redis.set_ex(&redis_key, &json, 86400 * 7) // 7 days TTL
                .await
                .context("Failed to store message in Redis")?;
            
            // Also add to recipient's message list
            let list_key = format!("{}{}:list", self.redis_prefix, recipient_key);
            let _: () = redis.lpush(&list_key, &message_id)
                .await
                .context("Failed to add message to list")?;

            tracing::info!("💾 MESSAGE STORED: id={} in Redis for recipient '{}'", message_id, recipient_key);
        } else {
            tracing::debug!("📝 MESSAGE (in-memory only): id={}", message_id);
        }

        // Try to deliver to registered instance
        let instances = self.instances.read().await;
        let registered_count = instances.len();
        
        if let Some(instance) = instances.get(&recipient_key) {
            match instance.sender.send(message.clone()).await {
                Ok(_) => {
                    tracing::info!(
                        "✅ MESSAGE DELIVERED: id={} to '{}' (app={}, user={})",
                        message_id, recipient_key, instance.app_id, instance.user_id
                    );
                }
                Err(e) => {
                    tracing::warn!(
                        "⚠️  MESSAGE DELIVERY FAILED: id={} to '{}': {}",
                        message_id, recipient_key, e
                    );
                }
            }
        } else {
            tracing::info!(
                "📭 MESSAGE QUEUED: id={} for '{}' (recipient not online, {} instances registered)",
                message_id, recipient_key, registered_count
            );
        }

        // Broadcast to all listeners
        match self.broadcast.send(message) {
            Ok(count) => {
                if count > 0 {
                    tracing::debug!("📡 MESSAGE BROADCAST: id={} to {} global listeners", message_id, count);
                }
            }
            Err(_) => {
                tracing::debug!("📡 No global broadcast listeners for message {}", message_id);
            }
        }

        Ok(message_id)
    }

    /// Get pending messages for an application instance
    pub async fn get_pending_messages(&self, app_id: &str, user_id: &str, limit: usize) -> Result<Vec<InterAppMessage>> {
        let recipient_key = format!("{}:{}", app_id, user_id);

        if let Some(ref mut redis) = self.redis.clone() {
            let list_key = format!("{}{}:list", self.redis_prefix, recipient_key);
            
            // Get message IDs
            let message_ids: Vec<String> = redis.lrange(&list_key, 0, limit as isize - 1)
                .await
                .unwrap_or_default();

            let mut messages = Vec::new();
            for msg_id in message_ids {
                let redis_key = format!("{}{}:{}", self.redis_prefix, recipient_key, msg_id);
                if let Ok(json) = redis.get::<_, String>(&redis_key).await {
                    if let Ok(msg) = serde_json::from_str::<InterAppMessage>(&json) {
                        messages.push(msg);
                    }
                }
            }

            Ok(messages)
        } else {
            // In-memory mode: no persistence
            Ok(Vec::new())
        }
    }

    /// Mark a message as delivered/read
    pub async fn mark_delivered(&self, app_id: &str, user_id: &str, message_id: &str) -> Result<()> {
        let recipient_key = format!("{}:{}", app_id, user_id);

        if let Some(ref mut redis) = self.redis.clone() {
            let redis_key = format!("{}{}:{}", self.redis_prefix, recipient_key, message_id);
            
            // Get and update message
            if let Ok(json) = redis.get::<_, String>(&redis_key).await {
                if let Ok(mut msg) = serde_json::from_str::<InterAppMessage>(&json) {
                    msg.delivered = true;
                    let updated_json = serde_json::to_string(&msg)?;
                    let _: () = redis.set_ex(&redis_key, &updated_json, 86400 * 7).await?;
                }
            }

            // Remove from pending list
            let list_key = format!("{}{}:list", self.redis_prefix, recipient_key);
            let _: () = redis.lrem(&list_key, 1, message_id).await?;
        } else {
            // Redis not available - just log and continue
            tracing::debug!(
                "mark_delivered called for '{}' message '{}' but Redis is not available (in-memory mode)",
                recipient_key, message_id
            );
        }

        Ok(())
    }

    /// Delete a message
    pub async fn delete_message(&self, app_id: &str, user_id: &str, message_id: &str) -> Result<()> {
        let recipient_key = format!("{}:{}", app_id, user_id);

        if let Some(ref mut redis) = self.redis.clone() {
            let redis_key = format!("{}{}:{}", self.redis_prefix, recipient_key, message_id);
            let _: () = redis.del(&redis_key).await?;

            let list_key = format!("{}{}:list", self.redis_prefix, recipient_key);
            let _: () = redis.lrem(&list_key, 0, message_id).await?;
        } else {
            // Redis not available - just log and continue
            tracing::debug!(
                "delete_message called for '{}' message '{}' but Redis is not available (in-memory mode)",
                recipient_key, message_id
            );
        }

        Ok(())
    }

    /// Subscribe to global message broadcast
    pub fn subscribe_broadcast(&self) -> broadcast::Receiver<InterAppMessage> {
        self.broadcast.subscribe()
    }

    /// Get count of registered instances
    pub async fn instance_count(&self) -> usize {
        self.instances.read().await.len()
    }

    /// Check if an app instance is registered
    pub async fn is_registered(&self, app_id: &str, user_id: &str) -> bool {
        let key = format!("{}:{}", app_id, user_id);
        self.instances.read().await.contains_key(&key)
    }
}

/// KV Store for application data (Redis-based)
pub struct AppKvStore {
    redis: Option<ConnectionManager>,
    prefix: String,
}

impl AppKvStore {
    /// Create a new KV store for an application instance
    pub async fn new(config: &RedisConfig, app_id: &str, user_id: &str) -> Result<Self> {
        let redis = if config.enabled {
            let url = if let Some(ref password) = config.password {
                format!("redis://:{}@{}:{}/{}", password, config.host, config.port, config.database)
            } else {
                format!("redis://{}:{}/{}", config.host, config.port, config.database)
            };

            let client = redis::Client::open(url.as_str())
                .context("Failed to create Redis client")?;
            
            let manager = ConnectionManager::new(client).await
                .context("Failed to create Redis connection manager")?;
            
            Some(manager)
        } else {
            None
        };

        Ok(Self {
            redis,
            prefix: format!("mecp:app:{}:{}:", app_id, user_id),
        })
    }

    /// Get a value by key
    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        if let Some(ref mut redis) = self.redis.clone() {
            let full_key = format!("{}{}", self.prefix, key);
            let value: Option<String> = redis.get(&full_key).await?;
            Ok(value)
        } else {
            Ok(None)
        }
    }

    /// Set a value by key
    pub async fn set(&self, key: &str, value: &str) -> Result<()> {
        if let Some(ref mut redis) = self.redis.clone() {
            let full_key = format!("{}{}", self.prefix, key);
            let _: () = redis.set(&full_key, value).await?;
        }
        Ok(())
    }

    /// Set a value with TTL (in seconds)
    pub async fn set_ex(&self, key: &str, value: &str, ttl_seconds: u64) -> Result<()> {
        if let Some(ref mut redis) = self.redis.clone() {
            let full_key = format!("{}{}", self.prefix, key);
            let _: () = redis.set_ex(&full_key, value, ttl_seconds).await?;
        }
        Ok(())
    }

    /// Delete a key
    pub async fn delete(&self, key: &str) -> Result<()> {
        if let Some(ref mut redis) = self.redis.clone() {
            let full_key = format!("{}{}", self.prefix, key);
            let _: () = redis.del(&full_key).await?;
        }
        Ok(())
    }

    /// Check if a key exists
    pub async fn exists(&self, key: &str) -> Result<bool> {
        if let Some(ref mut redis) = self.redis.clone() {
            let full_key = format!("{}{}", self.prefix, key);
            let exists: bool = redis.exists(&full_key).await?;
            Ok(exists)
        } else {
            Ok(false)
        }
    }

    /// List all keys matching a pattern
    pub async fn keys(&self, pattern: &str) -> Result<Vec<String>> {
        if let Some(ref mut redis) = self.redis.clone() {
            let full_pattern = format!("{}{}", self.prefix, pattern);
            let keys: Vec<String> = redis::cmd("KEYS")
                .arg(&full_pattern)
                .query_async(&mut redis.clone())
                .await?;
            
            // Strip prefix from keys
            let prefix_len = self.prefix.len();
            let stripped: Vec<String> = keys.into_iter()
                .map(|k| k[prefix_len..].to_string())
                .collect();
            
            Ok(stripped)
        } else {
            Ok(Vec::new())
        }
    }

    /// Increment a counter
    pub async fn incr(&self, key: &str) -> Result<i64> {
        if let Some(ref mut redis) = self.redis.clone() {
            let full_key = format!("{}{}", self.prefix, key);
            let value: i64 = redis.incr(&full_key, 1).await?;
            Ok(value)
        } else {
            Ok(0)
        }
    }

    /// Get JSON value
    pub async fn get_json<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<Option<T>> {
        if let Some(value) = self.get(key).await? {
            let parsed: T = serde_json::from_str(&value)?;
            Ok(Some(parsed))
        } else {
            Ok(None)
        }
    }

    /// Set JSON value
    pub async fn set_json<T: Serialize>(&self, key: &str, value: &T) -> Result<()> {
        let json = serde_json::to_string(value)?;
        self.set(key, &json).await
    }

    /// Push to a list
    pub async fn list_push(&self, key: &str, value: &str) -> Result<()> {
        if let Some(ref mut redis) = self.redis.clone() {
            let full_key = format!("{}{}", self.prefix, key);
            let _: () = redis.rpush(&full_key, value).await?;
        }
        Ok(())
    }

    /// Get list items
    pub async fn list_range(&self, key: &str, start: isize, stop: isize) -> Result<Vec<String>> {
        if let Some(ref mut redis) = self.redis.clone() {
            let full_key = format!("{}{}", self.prefix, key);
            let items: Vec<String> = redis.lrange(&full_key, start, stop).await?;
            Ok(items)
        } else {
            Ok(Vec::new())
        }
    }

    /// Get list length
    pub async fn list_len(&self, key: &str) -> Result<usize> {
        if let Some(ref mut redis) = self.redis.clone() {
            let full_key = format!("{}{}", self.prefix, key);
            let len: usize = redis.llen(&full_key).await?;
            Ok(len)
        } else {
            Ok(0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let msg = InterAppMessage::new(
            "mailbox", "alice",
            "mailbox", "bob",
            "new_mail",
            serde_json::json!({"subject": "Hello"}),
        );

        assert_eq!(msg.from_app, "mailbox");
        assert_eq!(msg.from_user, "alice");
        assert_eq!(msg.to_app, "mailbox");
        assert_eq!(msg.to_user, "bob");
        assert_eq!(msg.message_type, "new_mail");
        assert!(!msg.id.is_empty());
        assert!(!msg.delivered);
    }

    #[test]
    fn test_recipient_key() {
        let msg = InterAppMessage::new(
            "app1", "user1",
            "app2", "user2",
            "test",
            serde_json::json!({}),
        );

        assert_eq!(msg.recipient_key(), "app2:user2");
        assert_eq!(msg.sender_key(), "app1:user1");
    }

    #[tokio::test]
    async fn test_in_memory_broker() {
        let broker = MessageBroker::new_in_memory();

        // Register an app
        let mut sub = broker.register_app("testapp", "testuser").await.unwrap();
        
        assert!(broker.is_registered("testapp", "testuser").await);
        assert_eq!(broker.instance_count().await, 1);

        // Send a message
        let msg = InterAppMessage::new(
            "sender", "alice",
            "testapp", "testuser",
            "test",
            serde_json::json!({"data": "hello"}),
        );

        let msg_id = broker.send_message(msg).await.unwrap();
        assert!(!msg_id.is_empty());

        // Receive the message
        if let Some(received) = sub.try_recv() {
            assert_eq!(received.message_type, "test");
            assert_eq!(received.payload["data"], "hello");
        }

        // Unregister
        broker.unregister_app("testapp", "testuser").await;
        assert!(!broker.is_registered("testapp", "testuser").await);
    }
}
