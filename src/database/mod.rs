use sqlx::sqlite::SqlitePool;
use anyhow::{Context, Result};
use chrono::{Utc, Duration};
use std::path::PathBuf;

/// Database connection and management
pub struct Database {
    pool: SqlitePool,
}

/// Current database schema version
const SCHEMA_VERSION: i32 = 1;

/// Schema migration definition
struct Migration {
    version: i32,
    description: &'static str,
    sql: &'static str,
}

/// All database migrations in order
const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        description: "Initial schema with sessions, agents, interactions, and file_changes",
        sql: r#"
            -- Schema version tracking
            CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER PRIMARY KEY,
                applied_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            -- Sessions table
            CREATE TABLE sessions (
                id TEXT PRIMARY KEY,
                name TEXT,
                prompt TEXT NOT NULL,
                status TEXT NOT NULL CHECK (status IN ('active', 'completed', 'failed', 'paused')),
                agents_requested TEXT NOT NULL, -- JSON: {"claude": 2, "gpt": 1}
                started_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                completed_at DATETIME,
                created_by TEXT DEFAULT 'user'
            );

            -- Agents table
            CREATE TABLE agents (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                agent_type TEXT NOT NULL CHECK (agent_type IN ('claude', 'gpt', 'jules')),
                instance_number INTEGER NOT NULL, -- For claude-1, claude-2, etc.
                worktree_path TEXT,
                status TEXT NOT NULL CHECK (status IN ('initializing', 'running', 'waiting', 'completed', 'failed', 'paused')),
                progress INTEGER DEFAULT 0 CHECK (progress >= 0 AND progress <= 100),
                started_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                last_activity DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                process_id INTEGER,
                FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE CASCADE
            );

            -- Agent interactions (questions, responses, status updates, logs)
            CREATE TABLE interactions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                agent_id TEXT NOT NULL,
                session_id TEXT NOT NULL,
                type TEXT NOT NULL CHECK (type IN ('question', 'response', 'status', 'log', 'error', 'checkpoint')),
                content TEXT NOT NULL,
                metadata TEXT, -- JSON for additional structured data
                requires_response BOOLEAN DEFAULT FALSE,
                responded_at DATETIME,
                timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY(agent_id) REFERENCES agents(id) ON DELETE CASCADE,
                FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE CASCADE
            );

            -- File changes tracking
            CREATE TABLE file_changes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                agent_id TEXT NOT NULL,
                session_id TEXT NOT NULL,
                file_path TEXT NOT NULL,
                change_type TEXT NOT NULL CHECK (change_type IN ('created', 'modified', 'deleted', 'renamed')),
                lines_added INTEGER DEFAULT 0,
                lines_removed INTEGER DEFAULT 0,
                commit_hash TEXT,
                timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY(agent_id) REFERENCES agents(id) ON DELETE CASCADE,
                FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE CASCADE
            );

            -- Indexes for better query performance
            CREATE INDEX idx_agents_session_id ON agents(session_id);
            CREATE INDEX idx_agents_status ON agents(status);
            CREATE INDEX idx_interactions_agent_id ON interactions(agent_id);
            CREATE INDEX idx_interactions_session_id ON interactions(session_id);
            CREATE INDEX idx_interactions_type ON interactions(type);
            CREATE INDEX idx_interactions_requires_response ON interactions(requires_response);
            CREATE INDEX idx_file_changes_agent_id ON file_changes(agent_id);
            CREATE INDEX idx_file_changes_session_id ON file_changes(session_id);
            CREATE INDEX idx_sessions_status ON sessions(status);
            CREATE INDEX idx_sessions_started_at ON sessions(started_at);
        "#,
    },
];

impl Database {
    /// Create a new database connection
    pub async fn new(database_path: &PathBuf) -> Result<Self> {
        let database_url = format!("sqlite://{}?mode=rwc", database_path.display());
        
        let pool = SqlitePool::connect(&database_url)
            .await
            .with_context(|| format!("Failed to connect to database: {}", database_path.display()))?;

        let db = Self { pool };
        
        // Run migrations on startup
        db.migrate().await?;
        
        Ok(db)
    }

    /// Get database pool for direct access
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Run database migrations
    async fn migrate(&self) -> Result<()> {
        let current_version = self.get_schema_version().await?;
        
        if current_version >= SCHEMA_VERSION {
            return Ok(());
        }

        println!("🔄 Migrating database from version {} to {}", current_version, SCHEMA_VERSION);

        // Run migrations in order
        for migration in MIGRATIONS {
            if migration.version > current_version {
                println!("  📝 Applying migration {}: {}", migration.version, migration.description);
                
                let mut tx = self.pool.begin().await?;
                
                // Execute the migration SQL
                sqlx::query(migration.sql)
                    .execute(&mut *tx)
                    .await
                    .with_context(|| format!("Failed to apply migration {}", migration.version))?;
                
                // Update schema version
                sqlx::query("INSERT OR REPLACE INTO schema_version (version) VALUES (?)")
                    .bind(migration.version)
                    .execute(&mut *tx)
                    .await?;
                
                tx.commit().await?;
                
                println!("  ✅ Migration {} applied successfully", migration.version);
            }
        }

        Ok(())
    }

    /// Get current schema version
    async fn get_schema_version(&self) -> Result<i32> {
        // First, check if schema_version table exists
        let table_exists = sqlx::query_scalar::<_, i32>(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='schema_version'"
        )
        .fetch_one(&self.pool)
        .await?;

        if table_exists == 0 {
            return Ok(0); // No schema yet
        }

        // Get the latest version
        let version = sqlx::query_scalar::<_, Option<i32>>(
            "SELECT MAX(version) FROM schema_version"
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(version.unwrap_or(0))
    }

    /// Clean up old sessions and related data
    pub async fn cleanup_old_sessions(&self, days_to_keep: i64) -> Result<()> {
        let cutoff_date = Utc::now() - Duration::days(days_to_keep);
        
        println!("🧹 Cleaning up sessions older than {} days", days_to_keep);

        let mut tx = self.pool.begin().await?;

        // Delete old completed/failed sessions and cascade to related tables
        let deleted_sessions = sqlx::query(
            r#"
            DELETE FROM sessions 
            WHERE status IN ('completed', 'failed') 
            AND started_at < ?
            "#
        )
        .bind(cutoff_date)
        .execute(&mut *tx)
        .await?
        .rows_affected();

        // Clean up orphaned interactions (safety check)
        let deleted_interactions = sqlx::query(
            "DELETE FROM interactions WHERE timestamp < ? AND session_id NOT IN (SELECT id FROM sessions)"
        )
        .bind(cutoff_date)
        .execute(&mut *tx)
        .await?
        .rows_affected();

        tx.commit().await?;

        if deleted_sessions > 0 || deleted_interactions > 0 {
            println!("  ✅ Cleaned up {} sessions and {} orphaned interactions", 
                deleted_sessions, deleted_interactions);
        } else {
            println!("  ℹ️  No old data to clean up");
        }

        Ok(())
    }

    /// Get database statistics
    pub async fn get_stats(&self) -> Result<DatabaseStats> {
        let sessions_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM sessions")
            .fetch_one(&self.pool).await?;

        let active_agents_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM agents WHERE status IN ('initializing', 'running', 'waiting')"
        ).fetch_one(&self.pool).await?;

        let pending_questions_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM interactions WHERE type = 'question' AND requires_response = TRUE AND responded_at IS NULL"
        ).fetch_one(&self.pool).await?;

        let total_interactions_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM interactions")
            .fetch_one(&self.pool).await?;

        Ok(DatabaseStats {
            sessions_count,
            active_agents_count,
            pending_questions_count,
            total_interactions_count,
            schema_version: self.get_schema_version().await?,
        })
    }

    /// Close the database connection
    pub async fn close(&self) {
        self.pool.close().await;
    }
}

/// Database statistics
#[derive(Debug)]
pub struct DatabaseStats {
    pub sessions_count: i64,
    pub active_agents_count: i64,
    pub pending_questions_count: i64,
    pub total_interactions_count: i64,
    pub schema_version: i32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_database_creation_and_migration() {
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let db = Database::new(&db_path).await.expect("Should create database");
        
        // Check schema version
        let version = db.get_schema_version().await.expect("Should get version");
        assert_eq!(version, SCHEMA_VERSION);

        // Check that tables exist
        let table_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table'"
        )
        .fetch_one(db.pool())
        .await
        .expect("Should count tables");

        assert!(table_count >= 5); // At least our main tables + schema_version
    }

    #[tokio::test]
    async fn test_database_stats() {
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let db = Database::new(&db_path).await.expect("Should create database");
        let stats = db.get_stats().await.expect("Should get stats");

        assert_eq!(stats.sessions_count, 0);
        assert_eq!(stats.active_agents_count, 0);
        assert_eq!(stats.pending_questions_count, 0);
        assert_eq!(stats.schema_version, SCHEMA_VERSION);
    }

    #[tokio::test]
    async fn test_cleanup_old_sessions() {
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let db_path = temp_dir.path().join("test.db");

        let db = Database::new(&db_path).await.expect("Should create database");
        
        // Should not error even with no data
        db.cleanup_old_sessions(30).await.expect("Should cleanup without error");
    }
}