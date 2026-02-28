//! Background reconciliation worker for membership policies
//! 
//! This module provides async background processing for:
//! - Applying policies to users when policies change
//! - Periodic reconciliation to ensure consistency
//! - Processing audit trail and cleanup

use crate::api::AppState;
use crate::error::ApiResult;

use std::sync::Arc;
use std::time::Duration;
use tokio::time::{interval, MissedTickBehavior};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::membership_policies::{
    apply_auto_membership_for_team_join, PolicyRepository, PolicyScopeType,
};

/// Reconciliation task types
#[derive(Debug, Clone)]
pub enum ReconciliationTask {
    /// Apply a policy to all applicable users
    ApplyPolicy { policy_id: Uuid },
    /// Re-sync a specific user across all their teams
    ResyncUser { user_id: Uuid },
    /// Re-sync all users in a team
    ResyncTeam { team_id: Uuid },
    /// Full reconciliation check (periodic)
    FullReconciliation,
}

/// Worker for background membership reconciliation
pub struct ReconciliationWorker {
    state: Arc<AppState>,
    rx: async_channel::Receiver<ReconciliationTask>,
}

impl ReconciliationWorker {
    /// Create a new worker with the given app state
    pub fn new(state: Arc<AppState>) -> (Self, async_channel::Sender<ReconciliationTask>) {
        let (tx, rx) = async_channel::bounded(1000);
        (Self { state, rx }, tx)
    }

    /// Run the worker loop
    pub async fn run(self) {
        info!("Starting membership reconciliation worker");

        while let Ok(task) = self.rx.recv().await {
            debug!("Processing reconciliation task: {:?}", task);
            
            if let Err(e) = self.process_task(task).await {
                error!("Reconciliation task failed: {}", e);
            }
        }

        info!("Membership reconciliation worker stopped");
    }

    /// Process a single reconciliation task
    async fn process_task(&self, task: ReconciliationTask) -> ApiResult<()> {
        match task {
            ReconciliationTask::ApplyPolicy { policy_id } => {
                self.apply_policy(policy_id).await?;
            }
            ReconciliationTask::ResyncUser { user_id } => {
                self.resync_user(user_id).await?;
            }
            ReconciliationTask::ResyncTeam { team_id } => {
                self.resync_team(team_id).await?;
            }
            ReconciliationTask::FullReconciliation => {
                self.full_reconciliation().await?;
            }
        }
        Ok(())
    }

    /// Apply a policy to all applicable users
    async fn apply_policy(&self, policy_id: Uuid) -> ApiResult<()> {
        info!("Applying policy {} to all applicable users", policy_id);

        let repo = PolicyRepository::new(&self.state.db);
        let policy = match repo.get_policy(policy_id).await? {
            Some(p) if p.policy.enabled => p,
            _ => {
                debug!("Policy {} not found or disabled, skipping", policy_id);
                return Ok(());
            }
        };

        // Get all users this policy applies to based on source type
        let user_ids: Vec<(Uuid,)> = match policy.policy.source_type {
            super::membership_policies::PolicySourceType::AllUsers => {
                sqlx::query_as("SELECT id FROM users WHERE deleted_at IS NULL")
                    .fetch_all(&self.state.db)
                    .await?
            }
            super::membership_policies::PolicySourceType::AuthService => {
                let auth_service = policy
                    .policy
                    .source_config
                    .get("auth_service")
                    .and_then(|v| v.as_str());
                
                if let Some(service) = auth_service {
                    sqlx::query_as(
                        "SELECT id FROM users WHERE deleted_at IS NULL AND auth_service = $1"
                    )
                    .bind(service)
                    .fetch_all(&self.state.db)
                    .await?
                } else {
                    sqlx::query_as("SELECT id FROM users WHERE deleted_at IS NULL")
                        .fetch_all(&self.state.db)
                        .await?
                }
            }
            // Other source types would need more complex queries
            _ => {
                warn!("Source type {:?} not yet fully implemented for batch processing", 
                    policy.policy.source_type);
                return Ok(());
            }
        };

        info!("Applying policy {} to {} users", policy_id, user_ids.len());

        // Apply policy to each user
        for (user_id,) in user_ids {
            // Get user's teams
            let team_ids: Vec<(Uuid,)> = sqlx::query_as(
                "SELECT team_id FROM team_members WHERE user_id = $1"
            )
            .bind(user_id)
            .fetch_all(&self.state.db)
            .await?;

            for (team_id,) in &team_ids {
                // Skip if policy is team-scoped and doesn't match
                if policy.policy.scope_type == PolicyScopeType::Team {
                    if let Some(policy_team_id) = policy.policy.team_id {
                        if policy_team_id != *team_id {
                            continue;
                        }
                    }
                }

                // Apply the policy
                if let Err(e) = apply_auto_membership_for_team_join(
                    &self.state,
                    user_id,
                    *team_id,
                    "background_reconciliation",
                )
                .await
                {
                    error!("Failed to apply policy {} for user {} in team {}: {}",
                        policy_id, user_id, team_id, e);
                }
            }
        }

        info!("Finished applying policy {}", policy_id);
        Ok(())
    }

    /// Re-sync a specific user across all their teams
    async fn resync_user(&self, user_id: Uuid) -> ApiResult<()> {
        info!("Re-syncing memberships for user {}", user_id);

        // Get user's teams
        let team_ids: Vec<(Uuid,)> = sqlx::query_as(
            "SELECT team_id FROM team_members WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_all(&self.state.db)
        .await?;

        let mut total_applied = 0;
        let mut total_failed = 0;

        for (team_id,) in &team_ids {
            match apply_auto_membership_for_team_join(
                &self.state,
                user_id,
                *team_id,
                "background_resync",
            )
            .await
            {
                Ok(entries) => {
                    total_applied += entries.iter().filter(|e| e.status == "success").count();
                    total_failed += entries.iter().filter(|e| e.status == "failed").count();
                }
                Err(e) => {
                    error!("Failed to resync user {} in team {}: {}", user_id, team_id, e);
                    total_failed += 1;
                }
            }
        }

        info!(
            "Finished re-syncing user {}: {} applied, {} failed across {} teams",
            user_id, total_applied, total_failed, team_ids.len()
        );

        Ok(())
    }

    /// Re-sync all users in a team
    async fn resync_team(&self, team_id: Uuid) -> ApiResult<()> {
        info!("Re-syncing all memberships for team {}", team_id);

        let user_ids: Vec<(Uuid,)> = sqlx::query_as(
            "SELECT user_id FROM team_members WHERE team_id = $1"
        )
        .bind(team_id)
        .fetch_all(&self.state.db)
        .await?;

        info!("Re-syncing {} users in team {}", user_ids.len(), team_id);

        for (user_id,) in user_ids {
            if let Err(e) = apply_auto_membership_for_team_join(
                &self.state,
                user_id,
                team_id,
                "background_team_resync",
            )
            .await
            {
                error!("Failed to resync user {} in team {}: {}", user_id, team_id, e);
            }
        }

        info!("Finished re-syncing team {}", team_id);
        Ok(())
    }

    /// Full reconciliation - check all policies and memberships
    async fn full_reconciliation(&self) -> ApiResult<()> {
        info!("Starting full membership reconciliation");

        // Get all enabled policies
        let repo = PolicyRepository::new(&self.state.db);
        let policies = repo.list_policies(None, None, Some(true)).await?;

        info!("Processing {} enabled policies", policies.len());

        // Apply each policy
        for policy in policies {
            if let Err(e) = self.apply_policy(policy.policy.id).await {
                error!("Failed to apply policy {} during reconciliation: {}",
                    policy.policy.id, e);
            }
        }

        info!("Full reconciliation complete");
        Ok(())
    }
}

/// Spawn the reconciliation worker and return the task handle and sender
pub fn spawn_reconciliation_worker(
    state: Arc<AppState>,
) -> (tokio::task::JoinHandle<()>, async_channel::Sender<ReconciliationTask>) {
    let (worker, tx) = ReconciliationWorker::new(state);
    let handle = tokio::spawn(worker.run());
    (handle, tx)
}

/// Spawn periodic reconciliation tasks
pub fn spawn_periodic_reconciliation(
    _state: Arc<AppState>,
    tx: async_channel::Sender<ReconciliationTask>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(3600)); // Hourly
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            interval.tick().await;
            
            debug!("Triggering periodic full reconciliation");
            if let Err(e) = tx.send(ReconciliationTask::FullReconciliation).await {
                error!("Failed to queue periodic reconciliation: {}", e);
            }
        }
    })
}

/// Trigger reconciliation for a policy change
pub async fn trigger_policy_reconciliation(
    tx: &async_channel::Sender<ReconciliationTask>,
    policy_id: Uuid,
) -> ApiResult<()> {
    tx.send(ReconciliationTask::ApplyPolicy { policy_id })
        .await
        .map_err(|e| crate::error::AppError::Internal(format!("Failed to queue task: {}", e)))?;
    Ok(())
}

/// Trigger reconciliation for a user
pub async fn trigger_user_resync(
    tx: &async_channel::Sender<ReconciliationTask>,
    user_id: Uuid,
) -> ApiResult<()> {
    tx.send(ReconciliationTask::ResyncUser { user_id })
        .await
        .map_err(|e| crate::error::AppError::Internal(format!("Failed to queue task: {}", e)))?;
    Ok(())
}

/// Trigger reconciliation for a team
pub async fn trigger_team_resync(
    tx: &async_channel::Sender<ReconciliationTask>,
    team_id: Uuid,
) -> ApiResult<()> {
    tx.send(ReconciliationTask::ResyncTeam { team_id })
        .await
        .map_err(|e| crate::error::AppError::Internal(format!("Failed to queue task: {}", e)))?;
    Ok(())
}
