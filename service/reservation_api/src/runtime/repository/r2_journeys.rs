use worker::Bucket;
use reservation::projection::Journeys;

pub struct R2JourneysRepository {
    bucket: Bucket
}

impl R2JourneysRepository {
    pub fn build(bucket: Bucket) -> Self {
        Self {
            bucket
        }
    }

    pub async fn get(&self) -> worker::Result<Journeys> {
        let key = self.object_key();
        let result = self.bucket.get(key).execute().await?;

        if let Some(value) = result {
            if let Some(body) = value.body() {
                let data = body.bytes().await?;
                let journeys = serde_json::from_slice(&data)?;
                return Ok(journeys);
            }
        }
        Ok(Journeys::default())
    }

    pub async fn set(&self, journeys: &Journeys) -> worker::Result<()> {
        let key = self.object_key();
        let value = serde_json::to_vec(journeys)?;

        self.bucket.put(key, value).execute().await
            .map(|_| ())
        }

    fn object_key(&self) -> String {
        "journeys".to_owned()
    }
}