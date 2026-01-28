//! Data Access Object (DAO) layer
//!
//! This layer encapsulates data access logic and provides a cleaner abstraction
//! over repositories. It handles complex queries that span multiple entities.

pub mod client_dao;
pub mod rack_dao;

pub use client_dao::ClientDao;
pub use rack_dao::RackDao;
