//! Room grouping for batch operations.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::Error;
use crate::light::Light;
use crate::response::LightingResponse;

type Result<T> = std::result::Result<T, Error>;

/// A logical grouping of lights for batch operations.
///
/// Rooms allow you to organize lights and perform operations on multiple bulbs
/// at once. These groupings are independent of the Wiz app's room configuration.
///
/// # Example
///
/// ```
/// use std::net::Ipv4Addr;
/// use std::str::FromStr;
/// use wiz_lights_rs::{Room, Light};
///
/// let mut room = Room::new("Living Room");
/// let light = Light::new(Ipv4Addr::from_str("192.168.1.100").unwrap(), Some("Corner Lamp"));
/// let light_id = room.new_light(light).unwrap();
/// ```
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
    /// Create a new room with the given name.
    pub fn new(name: &str) -> Self {
        Room {
            name: String::from(name),
            lights: None,
            id: Uuid::new_v4(),
            linked: false,
        }
    }

    /// Link an external ID to this room.
    ///
    /// Can only be called once.
    ///
    /// # Panics
    ///
    /// Panics if called more than once.
    pub fn link(&mut self, id: &Uuid) {
        assert!(!self.linked, "refusing to overwrite id!");
        self.id = *id;
        self.linked = true;
    }

    /// Get the name of this room.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Query all bulbs in this room for their current status.
    ///
    /// This queries all lights concurrently using tokio's join_all.
    pub async fn get_status(&mut self) -> Result<Vec<LightingResponse>> {
        let Some(lights) = &mut self.lights else {
            return Ok(Vec::new());
        };

        let mut responses = Vec::new();
        for light in lights.values_mut() {
            let status = light.get_status().await?;
            responses.push(LightingResponse::status(light.ip(), status));
        }
        Ok(responses)
    }

    /// Add a new light to this room.
    ///
    /// Returns the UUID assigned to the light.
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

    /// Remove a light from this room.
    pub fn delete_light(&mut self, light_id: &Uuid) -> Result<()> {
        let Some(lights) = &mut self.lights else {
            return Err(Error::RoomNotFound(self.id));
        };

        lights
            .remove(light_id)
            .map(|_| ())
            .ok_or_else(|| Error::light_not_found(&self.id, light_id))
    }

    /// Update a light's configuration (IP address, name).
    ///
    /// # Example
    ///
    /// ```
    /// use std::str::FromStr;
    /// use std::net::Ipv4Addr;
    /// use wiz_lights_rs::{Room, Light};
    ///
    /// let ip1 = Ipv4Addr::from_str("10.1.2.3").unwrap();
    /// let ip2 = Ipv4Addr::from_str("10.1.2.4").unwrap();
    ///
    /// let mut room = Room::new("test");
    ///
    /// let light = Light::new(ip1, Some("foo"));
    /// let light_id = room.new_light(light).unwrap();
    ///
    /// let read = room.read(&light_id).unwrap();
    /// assert_eq!(read.name(), Some("foo"));
    /// assert_eq!(read.ip(), ip1);
    ///
    /// room.update_light(&light_id, &Light::new(ip2, Some("bar"))).unwrap();
    ///
    /// let read = room.read(&light_id).unwrap();
    /// assert_eq!(read.name(), Some("bar"));
    /// assert_eq!(read.ip(), ip2);
    /// ```
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

    /// List all light IDs in this room.
    ///
    /// # Example
    ///
    /// ```
    /// use std::str::FromStr;
    /// use std::net::Ipv4Addr;
    /// use wiz_lights_rs::{Room, Light};
    ///
    /// let mut room = Room::new("test");
    /// assert!(room.list().is_none());
    ///
    /// let light = Light::new(Ipv4Addr::from_str("10.1.2.3").unwrap(), None);
    /// let light_id = room.new_light(light).unwrap();
    ///
    /// let ids = room.list().unwrap();
    /// assert_eq!(*ids.iter().next().unwrap(), &light_id);
    /// ```
    pub fn list(&self) -> Option<Vec<&Uuid>> {
        self.lights.as_ref().map(|lights| lights.keys().collect())
    }

    /// Get a reference to a light by ID.
    pub fn read(&self, light_id: &Uuid) -> Option<&Light> {
        self.lights.as_ref().and_then(|lights| lights.get(light_id))
    }

    /// Get a mutable reference to a light by ID.
    pub fn read_mut(&mut self, light_id: &Uuid) -> Option<&mut Light> {
        self.lights
            .as_mut()
            .and_then(|lights| lights.get_mut(light_id))
    }

    /// Update lights in this room from a lighting response.
    ///
    /// Returns `true` if any light was updated.
    pub fn process_reply(&mut self, resp: &LightingResponse) -> bool {
        let Some(lights) = &mut self.lights else {
            return false;
        };

        lights.values_mut().any(|light| light.process_reply(resp))
    }

    /// Update this room's attributes from another room.
    ///
    /// # Example
    ///
    /// ```
    /// use wiz_lights_rs::Room;
    ///
    /// let mut room = Room::new("foo");
    /// let other = Room::new("bar");
    /// assert!(room.update(&other));
    /// assert_eq!(room.name(), "bar");
    /// ```
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
