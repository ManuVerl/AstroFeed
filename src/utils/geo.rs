/// Convert decimal degrees to radians.
#[allow(dead_code)]
pub fn deg_to_rad(deg: f64) -> f64 {
    deg * std::f64::consts::PI / 180.0
}

/// Convert radians to decimal degrees.
#[allow(dead_code)]
pub fn rad_to_deg(rad: f64) -> f64 {
    rad * 180.0 / std::f64::consts::PI
}

/// Compute the great-circle distance in km between two lat/lon points (Haversine).
#[allow(dead_code)]
pub fn haversine_km(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const R: f64 = 6371.0; // Earth radius in km
    let dlat = deg_to_rad(lat2 - lat1);
    let dlon = deg_to_rad(lon2 - lon1);
    let a = (dlat / 2.0).sin().powi(2)
        + deg_to_rad(lat1).cos() * deg_to_rad(lat2).cos() * (dlon / 2.0).sin().powi(2);
    2.0 * R * a.sqrt().asin()
}
