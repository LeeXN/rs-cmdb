//! Rack Data Access Object
//!
//! Encapsulates data access operations related to racks and their clients,
//! including position validation and overlap detection.

use crate::cache::CachedClientRepository;
use crate::repository::rack_repository::RackRepository;
use common::error::{CmdbError, CmdbResult};
use common::models::Client;
use std::sync::Arc;
use tracing::instrument;

/// Data Access Object for Rack operations
pub struct RackDao {
    rack_repo: Arc<RackRepository>,
    client_repo: Arc<CachedClientRepository>,
}

impl RackDao {
    /// Create a new RackDao
    pub fn new(rack_repo: Arc<RackRepository>, client_repo: Arc<CachedClientRepository>) -> Self {
        Self {
            rack_repo,
            client_repo,
        }
    }

    /// Get a rack by ID
    #[instrument(skip(self))]
    pub async fn get(&self, id: &str) -> CmdbResult<Option<common::models::Rack>> {
        self.rack_repo.get(id).await
    }

    /// Get all racks
    #[instrument(skip(self))]
    #[allow(dead_code)]
    pub async fn list_all(&self) -> CmdbResult<Vec<common::models::Rack>> {
        self.rack_repo.list_all().await
    }

    /// Save a rack
    #[instrument(skip(self, rack))]
    #[allow(dead_code)]
    pub async fn save(&self, rack: &common::models::Rack) -> CmdbResult<()> {
        self.rack_repo.save(rack).await
    }

    /// Delete a rack
    #[instrument(skip(self))]
    #[allow(dead_code)]
    pub async fn delete(&self, id: &str) -> CmdbResult<()> {
        self.rack_repo.delete(id).await
    }

    /// Get all clients in a rack
    #[instrument(skip(self))]
    pub async fn get_clients(&self, rack_id: &str) -> CmdbResult<Vec<Client>> {
        let all_clients: Vec<Client> = self.client_repo.list_all().await?;
        Ok(all_clients
            .into_iter()
            .filter(|c| c.rack.as_deref() == Some(rack_id))
            .collect())
    }

    /// Validate rack position (check for overlaps)
    #[instrument(skip(self))]
    pub async fn validate_position(
        &self,
        rack_id: &str,
        unit_position: u32,
        u_height: u32,
        exclude_client_id: Option<&str>,
    ) -> CmdbResult<()> {
        // Check if rack exists
        let Some(rack) = self.get(rack_id).await? else {
            return Err(CmdbError::Validation(format!("Rack {} not found", rack_id)));
        };

        // Check if position is within rack height
        if unit_position + u_height - 1 > rack.height_u {
            return Err(CmdbError::Validation(format!(
                "Position {} with height {} exceeds rack height {}",
                unit_position, u_height, rack.height_u
            )));
        }

        // Check for overlaps with other clients
        let rack_clients = self.get_clients(rack_id).await?;
        for client in rack_clients {
            // Skip the client we're validating (for updates)
            if exclude_client_id.is_some_and(|id| id == client.id) {
                continue;
            }

            if let Some(pos_str) = &client.unit_position
                && let Ok(other_pos) = pos_str.parse::<u32>()
            {
                let other_height = client.u_height.unwrap_or(1);
                let start1 = unit_position;
                let end1 = unit_position + u_height - 1;
                let start2 = other_pos;
                let end2 = other_pos + other_height - 1;

                // Check for overlap
                if start1 <= end2 && end1 >= start2 {
                    return Err(CmdbError::Validation(format!(
                        "Position overlap with client {} at position {} (height {})",
                        client.hostname, other_pos, other_height
                    )));
                }
            }
        }

        Ok(())
    }

    /// Get available positions in a rack
    #[instrument(skip(self))]
    #[allow(dead_code)]
    pub async fn get_available_positions(&self, rack_id: &str) -> CmdbResult<Vec<(u32, u32)>> {
        let Some(rack) = self.get(rack_id).await? else {
            return Err(CmdbError::NotFound(format!("Rack {} not found", rack_id)));
        };

        let rack_clients = self.get_clients(rack_id).await?;
        let mut occupied = vec![false; rack.height_u as usize];

        // Mark occupied positions
        for client in rack_clients {
            if let Some(pos_str) = &client.unit_position
                && let Ok(pos) = pos_str.parse::<usize>()
            {
                let height = client.u_height.unwrap_or(1) as usize;
                for i in pos.saturating_sub(1)..(pos + height - 1).min(rack.height_u as usize) {
                    if i < occupied.len() {
                        occupied[i] = true;
                    }
                }
            }
        }

        // Find available slots
        let mut available = Vec::new();
        let mut slot_start = None;

        for (i, &occ) in occupied.iter().enumerate() {
            if !occ {
                if slot_start.is_none() {
                    slot_start = Some(i);
                }
            } else {
                if let Some(start) = slot_start {
                    available.push((start as u32 + 1, (i - start) as u32));
                    slot_start = None;
                }
            }
        }

        // Handle trailing available space
        if let Some(start) = slot_start {
            available.push((start as u32 + 1, (occupied.len() - start) as u32));
        }

        Ok(available)
    }

    /// Get rack utilization statistics
    #[instrument(skip(self))]
    #[allow(dead_code)]
    pub async fn get_utilization(&self, rack_id: &str) -> CmdbResult<RackUtilization> {
        let Some(rack) = self.get(rack_id).await? else {
            return Err(CmdbError::NotFound(format!("Rack {} not found", rack_id)));
        };

        let rack_clients = self.get_clients(rack_id).await?;
        let client_count = rack_clients.len();
        let mut used_units = 0;

        for client in rack_clients {
            if let Some(_pos_str) = &client.unit_position {
                used_units += client.u_height.unwrap_or(1);
            }
        }

        let utilization_percent = if rack.height_u > 0 {
            (used_units as f64 / rack.height_u as f64) * 100.0
        } else {
            0.0
        };

        Ok(RackUtilization {
            rack_id: rack_id.to_string(),
            rack_name: rack.name,
            total_units: rack.height_u,
            used_units,
            available_units: rack.height_u - used_units,
            client_count,
            utilization_percent,
        })
    }
}

/// Rack utilization statistics
#[derive(Debug, serde::Serialize)]
#[allow(dead_code)]
pub struct RackUtilization {
    pub rack_id: String,
    pub rack_name: String,
    pub total_units: u32,
    pub used_units: u32,
    pub available_units: u32,
    pub client_count: usize,
    pub utilization_percent: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::CacheConfigs;
    use crate::repository::client_repository::ClientRepository;
    use crate::tests::fixtures::setup_test_db;
    use common::models::Rack;
    use crate::db::Database;

    #[tokio::test]
    async fn test_get_and_save_rack() {
        let db = setup_test_db().unwrap();
        let db: Arc<dyn Database> = Arc::new(db);

        let client_inner = Arc::new(ClientRepository::new(db.clone()));
        let configs = CacheConfigs::default();
        let client_repo = Arc::new(CachedClientRepository::new(client_inner, &configs));
        let rack_repo = Arc::new(RackRepository::new(db));
        let dao = RackDao::new(rack_repo, client_repo);

        let rack = Rack { name: "Rack-A".into(), height_u: 42, ..Default::default() };
        dao.save(&rack).await.unwrap();

        let retrieved = dao.get(&rack.id).await.unwrap().unwrap();
        assert_eq!(retrieved.name, "Rack-A");
    }

    #[tokio::test]
    async fn test_validate_position_valid() {
        let db = setup_test_db().unwrap();
        let db: Arc<dyn Database> = Arc::new(db);

        let client_inner = Arc::new(ClientRepository::new(db.clone()));
        let configs = CacheConfigs::default();
        let client_repo = Arc::new(CachedClientRepository::new(client_inner, &configs));
        let rack_repo = Arc::new(RackRepository::new(db));
        let dao = RackDao::new(rack_repo, client_repo);

        let rack = Rack { name: "Rack-B".into(), height_u: 42, ..Default::default() };
        dao.save(&rack).await.unwrap();

        let result = dao.validate_position(&rack.id, 1, 1, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_position_out_of_bounds() {
        let db = setup_test_db().unwrap();
        let db: Arc<dyn Database> = Arc::new(db);

        let client_inner = Arc::new(ClientRepository::new(db.clone()));
        let configs = CacheConfigs::default();
        let client_repo = Arc::new(CachedClientRepository::new(client_inner, &configs));
        let rack_repo = Arc::new(RackRepository::new(db));
        let dao = RackDao::new(rack_repo, client_repo);

        let rack = Rack { name: "Rack-C".into(), height_u: 42, ..Default::default() };
        dao.save(&rack).await.unwrap();

        let result = dao.validate_position(&rack.id, 50, 1, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_position_overlap() {
        let db = setup_test_db().unwrap();
        let db: Arc<dyn Database> = Arc::new(db);

        let client_inner = Arc::new(ClientRepository::new(db.clone()));
        let configs = CacheConfigs::default();
        let client_repo = Arc::new(CachedClientRepository::new(client_inner, &configs));
        let rack_repo = Arc::new(RackRepository::new(db));
        let dao = RackDao::new(rack_repo, client_repo.clone());

        let rack = Rack { name: "Rack-D".into(), height_u: 42, ..Default::default() };
        dao.save(&rack).await.unwrap();

        let mut client = Client::new("overlap-host".into(), "10.0.0.1".into());
        client.rack = Some(rack.id.clone());
        client.unit_position = Some("1".into());
        client.u_height = Some(2);
        client_repo.save(&client).await.unwrap();

        let result = dao.validate_position(&rack.id, 1, 1, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_utilization() {
        let db = setup_test_db().unwrap();
        let db: Arc<dyn Database> = Arc::new(db);

        let client_inner = Arc::new(ClientRepository::new(db.clone()));
        let configs = CacheConfigs::default();
        let client_repo = Arc::new(CachedClientRepository::new(client_inner, &configs));
        let rack_repo = Arc::new(RackRepository::new(db));
        let dao = RackDao::new(rack_repo, client_repo.clone());

        let rack = Rack { name: "Rack-E".into(), height_u: 42, ..Default::default() };
        dao.save(&rack).await.unwrap();

        let mut client = Client::new("util-host".into(), "10.0.0.1".into());
        client.rack = Some(rack.id.clone());
        client.unit_position = Some("1".into());
        client.u_height = Some(2);
        client_repo.save(&client).await.unwrap();

        let utilization = dao.get_utilization(&rack.id).await.unwrap();
        assert_eq!(utilization.total_units, 42);
        assert_eq!(utilization.used_units, 2);
        assert_eq!(utilization.available_units, 40);
        assert_eq!(utilization.client_count, 1);
    }

    #[tokio::test]
    async fn test_get_clients() {
        let db = setup_test_db().unwrap();
        let db: Arc<dyn Database> = Arc::new(db);

        let client_inner = Arc::new(ClientRepository::new(db.clone()));
        let configs = CacheConfigs::default();
        let client_repo = Arc::new(CachedClientRepository::new(client_inner, &configs));
        let rack_repo = Arc::new(RackRepository::new(db));
        let dao = RackDao::new(rack_repo, client_repo.clone());

        let rack = Rack { name: "Rack-F".into(), height_u: 42, ..Default::default() };
        dao.save(&rack).await.unwrap();

        let mut c1 = Client::new("rack-f-host1".into(), "10.0.0.1".into());
        c1.rack = Some(rack.id.clone());
        client_repo.save(&c1).await.unwrap();

        let mut c2 = Client::new("rack-f-host2".into(), "10.0.0.2".into());
        c2.rack = Some(rack.id.clone());
        client_repo.save(&c2).await.unwrap();

        let clients = dao.get_clients(&rack.id).await.unwrap();
        assert_eq!(clients.len(), 2);
    }
}
