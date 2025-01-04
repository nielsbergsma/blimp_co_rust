use worker::{Result, Bucket};
use scheduling::projection::Dashboard;

pub struct R2DashboardRepository {
    bucket: Bucket
}

impl R2DashboardRepository {
    pub fn build(bucket: Bucket) -> Self {
        Self {
            bucket
        }
    }

    pub async fn get(&self) -> Result<Dashboard> {
        let result = self.bucket.get(self.object_key()).execute().await?;
        if let Some(object) = result {
            if let Some(body) = object.body() {
                let data = body.bytes().await?;
                let dashboard = serde_json::from_slice(&data)?;
                return Ok(dashboard);
            }
        }
        Ok(Dashboard::default())
    }

    pub async fn set(&self, dashboard: Dashboard) -> Result<()> {
        let key = self.object_key();
        let value = serde_json::to_vec(&dashboard)?;

        self.bucket.put(key, value).execute().await
            .map(|_| ())
    }

    fn object_key(&self) -> String {
        "dashboard".to_owned()
    }
}