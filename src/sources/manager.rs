use crate::model::{event::Event, position::Position};
use chrono::Utc;
use egui::Context;
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
}

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
    pub fn refresh(&self, position: Position) {
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
            // Mark all sources as pending
            {
                let mut report = sync_report.lock().unwrap();
                *report = vec![
                    SyncReport::pending("ISS Passes (Celestrak TLE)"),
                    SyncReport::pending("Planets (JPL Horizons)"),
                    SyncReport::pending("Meteor Showers (IMO)"),
                    SyncReport::pending("Comets (MPC)"),
                    SyncReport::pending("Solar Transit (calc. local)"),
                    SyncReport::pending("ISS Radio (ARISS)"),
                ];
            }
            ctx.request_repaint();

            // Fetch from each source concurrently
            let (iss_result, planets_result, meteors_result, comets_result, solar_result, iss_radio_result) = tokio::join!(
                crate::sources::iss_passes::fetch(&position),
                crate::sources::planets::fetch(&position),
                crate::sources::meteors::fetch(&position),
                crate::sources::comets::fetch(&position),
                crate::sources::solar_transit::fetch(&position),
                crate::sources::iss_radio::fetch(&position),
            );

            let results = vec![
                ("ISS Passes (Celestrak TLE)", iss_result),
                ("Planets (JPL Horizons)", planets_result),
                ("Meteor Showers (IMO)", meteors_result),
                ("Comets (MPC)", comets_result),
                ("Solar Transit (calc. local)", solar_result),
                ("ISS Radio (ARISS)", iss_radio_result),
            ];

            let mut all_events: Vec<Event> = Vec::new();
            let mut reports: Vec<SyncReport> = Vec::new();

            for (name, result) in results {
                match result {
                    Ok(mut evts) => {
                        all_events.append(&mut evts);
                        reports.push(SyncReport {
                            source_name: name.to_string(),
                            status: SyncStatus::Ok,
                            last_sync: Some(Utc::now()),
                            error_message: None,
                        });
                    }
                    Err(e) => {
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
