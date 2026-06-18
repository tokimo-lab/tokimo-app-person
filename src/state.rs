use std::sync::{Arc, OnceLock};

use sea_orm::DatabaseConnection;
use tokimo_bus_client::BusClient;

pub struct AppState {
    pub db: DatabaseConnection,
    #[allow(dead_code)]
    pub bus_client: Arc<OnceLock<Arc<BusClient>>>,
}
