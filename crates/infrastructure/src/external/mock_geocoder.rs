use async_trait::async_trait;

use delivery_domain::errors::DomainResult;
use delivery_domain::ports::Geocoder;
use delivery_domain::value_objects::Location;

/// Mock geocoder that returns a fixed location in central India.
/// Replace with NominatimGeocoder or Google Maps for production.
pub struct MockGeocoder;

#[async_trait]
impl Geocoder for MockGeocoder {
    async fn geocode(&self, _address: &str) -> DomainResult<Location> {
        // Default: Bhopal, MP, India
        Location::new(23.2599, 77.4126)
    }

    async fn reverse_geocode(&self, location: Location) -> DomainResult<String> {
        Ok(format!("{:.4}, {:.4}", location.latitude, location.longitude))
    }
}
