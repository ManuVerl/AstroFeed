use crate::model::{event::Event, position::Position};
use chrono::Utc;
use egui::Context;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

/// Status of a single external source synchronisation.
#[derive(Debug, Clone)]
pub struct SyncReport {
    pub source_name: String,
    pub status: SyncStatus,
    pub last_sync: Option<chrono::DateTime<Utc>>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SyncStatus {
    Ok,
    Error,
    Pending,
    Disabled,
}

impl SyncReport {
    pub fn pending(name: &str) -> Self {
        Self {
            source_name: name.to_string(),
            status: SyncStatus::Pending,
            last_sync: None,
            error_message: None,
        }
    }

    pub fn disabled(name: &str) -> Self {
        Self {
            source_name: name.to_string(),
            status: SyncStatus::Disabled,
            last_sync: None,
            error_message: None,
        }
    }
}

/// Canonical names for all external data sources.
/// These strings are used as keys in `Settings::disabled_sources`.
pub const SOURCE_NAMES: &[&str] = &[
    "ISS Passes (Celestrak TLE)",
    "Planets (JPL Horizons)",
    "Meteor Showers (IMO)",
    "Comets (MPC)",
    "Solar Transit (calc. local)",
    "ISS Radio (ARISS)",
    "Milky Way Transit (calc. local)",
];

/// Orchestrates all external data sources.
pub struct SourceManager {
    events: Arc<Mutex<Vec<Event>>>,
    sync_report: Arc<Mutex<Vec<SyncReport>>>,
    ctx: Context,
}

impl SourceManager {
    pub fn new(
        events: Arc<Mutex<Vec<Event>>>,
        sync_report: Arc<Mutex<Vec<SyncReport>>>,
        ctx: Context,
    ) -> Self {
        Self { events, sync_report, ctx }
    }

    /// Spawn background tasks to refresh all sources for the given position.
    /// Sources whose name is in `disabled` are skipped and reported as Disabled.
    pub fn refresh(&self, position: Position, disabled: HashSet<String>) {
        let events = Arc::clone(&self.events);
        let sync_report = Arc::clone(&self.sync_report);
        let ctx = self.ctx.clone();

        // Use the global runtime handle so the task runs even when called from
        // inside eframe's blocking event loop (which is not a tokio context).
        let handle = match crate::TOKIO_HANDLE.get() {
            Some(h) => h.clone(),
            None => {
                log::error!("Tokio runtime not initialized");
                return;
            }
        };

        handle.spawn(async move {
            // Mark active sources as pending, disabled ones immediately as Disabled.
            {
                let mut report = sync_report.lock().unwrap();
                *report = SOURCE_NAMES
                    .iter()
                    .map(|name| {
                        if disabled.contains(*name) {
                            SyncReport::disabled(name)
                        } else {
                            SyncReport::pending(name)
                        }
                    })
                    .collect();
            }
            ctx.request_repaint();

            // Fetch from each active source concurrently
            let (iss_result, planets_result, meteors_result, comets_result, solar_result, iss_radio_result, milky_way_result) = tokio::join!(
                async {
                    if disabled.contains("ISS Passes (Celestrak TLE)") { return None; }
                    Some(crate::sources::iss_passes::fetch(&position).await)
                },
                async {
                    if disabled.contains("Planets (JPL Horizons)") { return None; }
                    Some(crate::sources::planets::fetch(&position).await)
                },
                async {
                    if disabled.contains("Meteor Showers (IMO)") { return None; }
                    Some(crate::sources::meteors::fetch(&position).await)
                },
                async {
                    if disabled.contains("Comets (MPC)") { return None; }
                    Some(crate::sources::comets::fetch(&position).await)
                },
                async {
                    if disabled.contains("Solar Transit (calc. local)") { return None; }
                    Some(crate::sources::solar_transit::fetch(&position).await)
                },
                async {
                    if disabled.contains("ISS Radio (ARISS)") { return None; }
                    Some(crate::sources::iss_radio::fetch(&position).await)
                },
                async {
                    if disabled.contains("Milky Way Transit (calc. local)") { return None; }
                    Some(crate::sources::milky_way::fetch(&position).await)
                },
            );

            let results: Vec<(&str, Option<Result<Vec<Event>, _>>)> = vec![
                ("ISS Passes (Celestrak TLE)",       iss_result),
                ("Planets (JPL Horizons)",            planets_result),
                ("Meteor Showers (IMO)",              meteors_result),
                ("Comets (MPC)",                      comets_result),
                ("Solar Transit (calc. local)",       solar_result),
                ("ISS Radio (ARISS)",                 iss_radio_result),
                ("Milky Way Transit (calc. local)",   milky_way_result),
            ];

            let mut all_events: Vec<Event> = Vec::new();
            let mut reports: Vec<SyncReport> = Vec::new();

            for (name, opt_result) in results {
                match opt_result {
                    None => {
                        // Source is disabled — keep as-is (already in the report).
                        reports.push(SyncReport::disabled(name));
                    }
                    Some(Ok(mut evts)) => {
                        all_events.append(&mut evts);
                        reports.push(SyncReport {
                            source_name: name.to_string(),
                            status: SyncStatus::Ok,
                            last_sync: Some(Utc::now()),
                            error_message: None,
                        });
                    }
                    Some(Err(e)) => {
                        log::warn!("Source '{}' failed: {}", name, e);
                        reports.push(SyncReport {
                            source_name: name.to_string(),
                            status: SyncStatus::Error,
                            last_sync: None,
                            error_message: Some(e.to_string()),
                        });
                    }
                }
            }

            // Sort chronologically
            all_events.sort_by_key(|e| e.start_time);

            {
                let mut ev = events.lock().unwrap();
                *ev = all_events;
            }
            {
                let mut rp = sync_report.lock().unwrap();
                *rp = reports;
            }

            ctx.request_repaint();
        });
    }
}
