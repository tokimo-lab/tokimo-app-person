use std::sync::Arc;

use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

use crate::{bus_clients::app_events, state::AppState};

pub async fn emit_person_event(
    ctx: &Arc<AppState>,
    user_id: Uuid,
    operation: &str,
    person_id: Uuid,
    affected_person_ids: Vec<Uuid>,
    changed_fields: &[&str],
) {
    let Some(client) = ctx.bus_client.get() else {
        tracing::warn!("person event skipped: bus client not bound");
        return;
    };

    let affected: Vec<String> = affected_person_ids.into_iter().map(|id| id.to_string()).collect();
    let payload = json!({
        "eventId": Uuid::new_v4().to_string(),
        "occurredAt": Utc::now().to_rfc3339(),
        "entity": "person",
        "operation": operation,
        "personId": person_id.to_string(),
        "affectedPersonIds": affected,
        "changedFields": changed_fields,
    });

    if let Err(e) =
        app_events::emit_entity(client, user_id, "person", Some(format!("person:{person_id}")), payload).await
    {
        tracing::warn!(err = %e, operation, person_id = %person_id, "person event emit failed");
    }
}
