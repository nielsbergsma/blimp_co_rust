use worker::{Bucket};
use reservation::projection::{Availability, YearMonth};

pub struct R2AvailabilityRepository {
    bucket: Bucket
}

impl R2AvailabilityRepository {
    pub fn build(bucket: Bucket) -> Self {
        Self {
            bucket
        }
    }

    pub async fn get(&self, period: YearMonth) -> worker::Result<Availability> {
        let key = self.object_key(period);
        let result = self.bucket.get(key).execute().await?;

        if let Some(object) = result {
            if let Some(value) = object.body() {
                let data = value.bytes().await?;
                let availability = serde_json::from_slice(&data)?;
                return Ok(availability);
            }
        }
        Ok(Availability::from_period(period))
    }

    pub async fn set(&self, availability: Availability) -> worker::Result<()> {
        let key = self.object_key(availability.period());
        let value = serde_json::to_vec(&availability)?;

        self.bucket.put(key, value).execute().await
            .map(|_| ())
    }

    fn object_key(&self, period: YearMonth) -> String {
        ["availability/", &period.year().to_string(), "/", &period.month().name().to_lowercase()].concat()
    }
}