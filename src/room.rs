//! Room grouping for batch operations.

use std::collections::HashMap;

use futures::future;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::Error;
use crate::light::Light;
use crate::response::LightingResponse;

type Result<T> = std::result::Result<T, Error>;

/// A grouping of lights for batch operations.
#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Room {
    name: String,
    lights: Option<HashMap<Uuid, Light>>,
    #[serde(skip)]
    id: Uuid,
    #[serde(skip)]
    linked: bool,
}

impl Room {
    pub fn new(name: &str) -> Self {
        Room {
            name: String::from(name),
            lights: None,
            id: Uuid::new_v4(),
            linked: false,
        }
    }

    pub fn link(&mut self, id: &Uuid) {
        assert!(!self.linked, "refusing to overwrite id!");
        self.id = *id;
        self.linked = true;
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub async fn get_status(&self) -> Result<Vec<LightingResponse>> {
        let Some(lights) = &self.lights else {
            return Ok(Vec::new());
        };

        // Create futures for concurrent execution
        let futures: Vec<_> = lights
            .values()
            .map(|light| async move {
                let ip = light.ip();
                light
                    .get_status()
                    .await
                    .map(|status| LightingResponse::status(ip, status))
            })
            .collect();

        // Execute all queries concurrently using join_all
        let results = future::join_all(futures).await;

        // Collect successful responses and return first error if any
        let mut responses = Vec::new();
        for result in results {
            responses.push(result?);
        }
        Ok(responses)
    }

    pub fn new_light(&mut self, light: Light) -> Result<Uuid> {
        self.validate_light(&light, None)?;

        let id = Uuid::new_v4();
        match &mut self.lights {
            Some(lights) => {
                lights.insert(id, light);
            }
            None => {
                self.lights = Some(HashMap::from([(id, light)]));
            }
        }
        Ok(id)
    }

    pub fn delete_light(&mut self, light_id: &Uuid) -> Result<()> {
        let Some(lights) = &mut self.lights else {
            return Err(Error::RoomNotFound(self.id));
        };

        lights
            .remove(light_id)
            .map(|_| ())
            .ok_or_else(|| Error::light_not_found(&self.id, light_id))
    }

    pub fn update_light(&mut self, id: &Uuid, light: &Light) -> Result<()> {
        let Some(lights) = &mut self.lights else {
            return Err(Error::NoLights(self.id));
        };

        let Some(existing) = lights.get_mut(id) else {
            return Err(Error::light_not_found(&self.id, id));
        };

        if existing.update(light) {
            Ok(())
        } else {
            Err(Error::no_change_light(&self.id, id))
        }
    }

    pub fn list(&self) -> Option<Vec<&Uuid>> {
        self.lights.as_ref().map(|lights| lights.keys().collect())
    }

    pub fn read(&self, light_id: &Uuid) -> Option<&Light> {
        self.lights.as_ref().and_then(|lights| lights.get(light_id))
    }

    pub fn read_mut(&mut self, light_id: &Uuid) -> Option<&mut Light> {
        self.lights
            .as_mut()
            .and_then(|lights| lights.get_mut(light_id))
    }

    pub fn process_reply(&mut self, resp: &LightingResponse) -> bool {
        let Some(lights) = &mut self.lights else {
            return false;
        };

        lights.values_mut().any(|light| light.process_reply(resp))
    }

    pub fn update(&mut self, other: &Self) -> bool {
        if self.name == other.name {
            return false;
        }
        self.name.clone_from(&other.name);
        true
    }

    fn validate_light(&self, light: &Light, exclude_id: Option<&Uuid>) -> Result<()> {
        let Some(lights) = &self.lights else {
            return Ok(());
        };

        let ip = light.ip();
        for (id, known) in lights {
            if Some(id) == exclude_id {
                continue;
            }
            if known.ip() == ip {
                return Err(Error::invalid_ip(&ip, "already known"));
            }
        }
        Ok(())
    }
}
