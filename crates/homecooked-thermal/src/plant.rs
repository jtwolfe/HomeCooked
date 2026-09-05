//! In-memory thermal plant: reservoirs, heat ports, offer/accept, tick step.

use std::collections::BTreeMap;

use crate::error::Error;
use crate::types::{
    delta_temp_c, energy_kwh, require_compatible, require_overlap, require_temp_in_band, HeatPort,
    Media, PortRef, Reservoir, ReservoirRole, TempBandC, TransferAccept, TransferDecline,
    TransferOffer, TransferReply, TransferResult, TransferTarget,
};

/// In-memory plant registry and best-effort negotiator.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ThermalPlant {
    reservoirs: BTreeMap<String, Reservoir>,
    ports: BTreeMap<(String, String), HeatPort>,
    pending: Vec<TransferAccept>,
}

impl ThermalPlant {
    pub fn new() -> Self {
        Self::default()
    }

    /// Fridge condenser (source) recovering into a DHW / water-heater preheat
    /// sink. Device ids are instances only; class names `fridge` and
    /// `water_heater` already exist in the catalog.
    pub fn fridge_condenser_dhw_demo() -> Result<Self, Error> {
        let mut plant = Self::new();
        plant.add_reservoir(Reservoir::new(
            "dhw-tank",
            ReservoirRole::Dhw,
            Media::Water,
            Some(35.0),
            TempBandC::new(20.0, 60.0)?,
            Some(4.0),
            Some(2.0),
        )?)?;
        plant.attach_port(HeatPort::new(
            "fridge-kitchen",
            "condenser",
            crate::types::PortDirection::Source,
            120,
            TempBandC::new(35.0, 55.0)?,
            1,
            Media::Water,
            None,
        )?)?;
        plant.attach_port(HeatPort::new(
            "water-heater-plant",
            "preheat",
            crate::types::PortDirection::Sink,
            2_000,
            TempBandC::new(20.0, 60.0)?,
            4,
            Media::Water,
            Some("dhw-tank".into()),
        )?)?;
        Ok(plant)
    }

    pub fn add_reservoir(&mut self, reservoir: Reservoir) -> Result<(), Error> {
        if self.reservoirs.contains_key(&reservoir.id) {
            return Err(Error::DuplicateReservoir(reservoir.id));
        }
        self.reservoirs.insert(reservoir.id.clone(), reservoir);
        Ok(())
    }

    /// Attach a heat port to its owning device (and optionally a reservoir).
    pub fn attach_port(&mut self, port: HeatPort) -> Result<(), Error> {
        if let Some(rid) = &port.attached_reservoir_id {
            if !self.reservoirs.contains_key(rid) {
                return Err(Error::UnknownReservoir(rid.clone()));
            }
            require_compatible(port.media, self.reservoirs[rid].media)?;
        }
        let key = (port.device_id.clone(), port.port_id.clone());
        if self.ports.contains_key(&key) {
            return Err(Error::DuplicatePort {
                device_id: port.device_id,
                port_id: port.port_id,
            });
        }
        self.ports.insert(key, port);
        Ok(())
    }

    pub fn attach_port_to_reservoir(
        &mut self,
        device_id: &str,
        port_id: &str,
        reservoir_id: Option<&str>,
    ) -> Result<(), Error> {
        if let Some(rid) = reservoir_id {
            if !self.reservoirs.contains_key(rid) {
                return Err(Error::UnknownReservoir(rid.to_string()));
            }
        }
        let port = self
            .ports
            .get_mut(&(device_id.to_string(), port_id.to_string()));
        let port = match port {
            Some(p) => p,
            None => {
                return Err(Error::UnknownPort {
                    device_id: device_id.to_string(),
                    port_id: port_id.to_string(),
                });
            }
        };
        if let Some(rid) = reservoir_id {
            require_compatible(port.media, self.reservoirs[rid].media)?;
            port.attached_reservoir_id = Some(rid.to_string());
        } else {
            port.attached_reservoir_id = None;
        }
        Ok(())
    }

    pub fn get_reservoir(&self, id: &str) -> Option<&Reservoir> {
        self.reservoirs.get(id)
    }

    pub fn get_port(&self, device_id: &str, port_id: &str) -> Option<&HeatPort> {
        self.ports
            .get(&(device_id.to_string(), port_id.to_string()))
    }

    pub fn list_reservoirs(&self) -> Vec<&Reservoir> {
        self.reservoirs.values().collect()
    }

    pub fn list_ports(&self) -> Vec<&HeatPort> {
        self.ports.values().collect()
    }

    pub fn list_ports_for_device(&self, device_id: &str) -> Vec<&HeatPort> {
        self.ports
            .values()
            .filter(|p| p.device_id == device_id)
            .collect()
    }

    /// Validate an offer without changing plant state.
    pub fn offer(&self, offer: &TransferOffer) -> Result<(), Error> {
        self.validate_offer(offer)?;
        Ok(())
    }

    /// Queue an accepted transfer (partial fill allowed). Energy is applied
    /// on [`Self::step`].
    pub fn accept(
        &mut self,
        offer: TransferOffer,
        accepted_power_w: u32,
    ) -> Result<TransferAccept, Error> {
        let max = self.max_power_w(&offer)?;
        if accepted_power_w == 0 {
            return Err(Error::ZeroPower);
        }
        if accepted_power_w > max {
            return Err(Error::PowerExceedsMax {
                requested: accepted_power_w,
                max,
            });
        }
        self.validate_offer(&offer)?;
        let accept = TransferAccept {
            from_port: offer.from_port,
            to: offer.to,
            accepted_power_w,
            duration_s: offer.duration_s,
            priority: offer.priority,
        };
        self.pending.push(accept.clone());
        Ok(accept)
    }

    /// Decline: no queued transfer, no temperature change.
    pub fn decline(&self, reason: impl Into<String>) -> TransferDecline {
        let _ = self;
        TransferDecline::new(reason)
    }

    /// Accept at `min(offered_max, port limits)` or decline with the error
    /// message. Declines when available max is below the offered minimum
    /// (partial fill below `power_w.min` is not automatic). Still requires
    /// [`Self::step`] to move energy.
    pub fn negotiate(&mut self, offer: TransferOffer) -> TransferReply {
        let max = match self.max_power_w(&offer) {
            Ok(m) => m,
            Err(e) => return TransferReply::Decline(TransferDecline::new(e.to_string())),
        };
        if max < offer.power_w.min {
            return TransferReply::Decline(TransferDecline::new(format!(
                "available max {max} W below offer min {} W",
                offer.power_w.min
            )));
        }
        match self.accept(offer, max) {
            Ok(a) => TransferReply::Accept(a),
            Err(e) => TransferReply::Decline(TransferDecline::new(e.to_string())),
        }
    }

    /// Apply queued accepts over `dt_s` seconds.
    ///
    /// Competing transfers that share a reservoir are served highest
    /// [`TransferAccept::priority`] first against remaining `headroom_kw`.
    pub fn step(&mut self, dt_s: f32) -> Result<Vec<TransferResult>, Error> {
        let mut pending = std::mem::take(&mut self.pending);
        pending.sort_by(|a, b| {
            b.priority
                .cmp(&a.priority)
                .then_with(|| a.from_port.device_id.cmp(&b.from_port.device_id))
                .then_with(|| a.from_port.port_id.cmp(&b.from_port.port_id))
        });

        let mut remaining_w: BTreeMap<String, f32> = BTreeMap::new();
        for (id, r) in &self.reservoirs {
            if let Some(kw) = r.headroom_kw {
                remaining_w.insert(id.clone(), kw * 1000.0);
            }
        }

        let mut results = Vec::new();
        for accept in pending {
            if let Some(result) = self.apply_accept(&accept, dt_s, &mut remaining_w)? {
                results.push(result);
            }
        }
        Ok(results)
    }

    /// Accept then step a single transfer (test / demo helper).
    pub fn apply(
        &mut self,
        offer: TransferOffer,
        accepted_power_w: u32,
        dt_s: f32,
    ) -> Result<TransferResult, Error> {
        self.accept(offer, accepted_power_w)?;
        let mut results = self.step(dt_s)?;
        results.pop().ok_or(Error::ZeroPower)
    }

    fn validate_offer(&self, offer: &TransferOffer) -> Result<(), Error> {
        let source = self.port_or_err(&offer.from_port)?;
        if !source.direction.can_source() {
            return Err(Error::WrongDirection {
                device_id: source.device_id.clone(),
                port_id: source.port_id.clone(),
                direction: source.direction,
                needed: "source",
            });
        }

        let sink = self.sink_context(&offer.to)?;
        require_compatible(source.media, sink.media)?;
        require_overlap(source.usable_temp_c, sink.usable_temp_c)?;

        if let Some(rid) = &source.attached_reservoir_id {
            let r = self.reservoir_or_err(rid)?;
            require_compatible(source.media, r.media)?;
            require_overlap(source.usable_temp_c, r.usable_band_c)?;
            if let Some(t) = r.temp_c {
                require_temp_in_band(t, source.usable_temp_c)?;
                require_temp_in_band(t, sink.usable_temp_c)?;
            }
        }

        if let Some(rid) = &sink.heated_reservoir_id {
            let r = self.reservoir_or_err(rid)?;
            require_compatible(sink.media, r.media)?;
            require_overlap(sink.usable_temp_c, r.usable_band_c)?;
            require_overlap(source.usable_temp_c, r.usable_band_c)?;
            if let Some(t) = r.temp_c {
                require_temp_in_band(t, source.usable_temp_c)?;
                require_temp_in_band(t, sink.usable_temp_c)?;
                require_temp_in_band(t, r.usable_band_c)?;
            }
        }

        if offer.power_w.max == 0 {
            return Err(Error::ZeroPower);
        }
        Ok(())
    }

    fn max_power_w(&self, offer: &TransferOffer) -> Result<u32, Error> {
        let source = self.port_or_err(&offer.from_port)?;
        let mut max = source.max_power_w.min(offer.power_w.max);
        if let TransferTarget::Port { device_id, port_id } = &offer.to {
            let sink = self.port_or_err(&PortRef {
                device_id: device_id.clone(),
                port_id: port_id.clone(),
            })?;
            max = max.min(sink.max_power_w);
        }
        Ok(max)
    }

    fn apply_accept(
        &mut self,
        accept: &TransferAccept,
        dt_s: f32,
        remaining_w: &mut BTreeMap<String, f32>,
    ) -> Result<Option<TransferResult>, Error> {
        // Re-check bands: an earlier higher-priority fill may have moved temp.
        if self.validate_offer_from_accept(accept).is_err() {
            return Ok(None);
        }

        let source = self.port_or_err(&accept.from_port)?.clone();
        let sink = self.sink_context(&accept.to)?;
        let heated_id = sink.heated_reservoir_id.clone();
        let cooled_id = source.attached_reservoir_id.clone();

        let mut power = accept.accepted_power_w;
        power = cap_headroom(power, heated_id.as_deref(), remaining_w);
        power = cap_headroom(power, cooled_id.as_deref(), remaining_w);
        if power == 0 {
            return Ok(None);
        }

        let duration_s = accept.duration_s.map(|d| d as f32).unwrap_or(dt_s);
        let applied_s = if duration_s < dt_s { duration_s } else { dt_s };
        let e_kwh = energy_kwh(power, applied_s);

        let mut delta = 0.0;
        if let Some(rid) = &heated_id {
            delta = self.add_temp(rid, e_kwh)?;
            debit_headroom(rid, power, remaining_w);
        }
        if let Some(rid) = &cooled_id {
            let _ = self.add_temp(rid, -e_kwh)?;
            if heated_id.as_deref() != Some(rid.as_str()) {
                debit_headroom(rid, power, remaining_w);
            }
        }

        Ok(Some(TransferResult {
            from_port: accept.from_port.clone(),
            to: accept.to.clone(),
            power_w: power,
            energy_kwh: e_kwh,
            heated_reservoir_id: heated_id,
            delta_temp_c: delta,
        }))
    }

    fn validate_offer_from_accept(&self, accept: &TransferAccept) -> Result<(), Error> {
        self.validate_offer(&TransferOffer {
            from_port: accept.from_port.clone(),
            to: accept.to.clone(),
            power_w: crate::types::PowerBandW {
                min: 0,
                max: accept.accepted_power_w.max(1),
            },
            duration_s: accept.duration_s,
            priority: accept.priority,
        })
    }

    fn add_temp(&mut self, reservoir_id: &str, energy_kwh: f32) -> Result<f32, Error> {
        let r = self
            .reservoirs
            .get_mut(reservoir_id)
            .ok_or_else(|| Error::UnknownReservoir(reservoir_id.to_string()))?;
        let delta = delta_temp_c(energy_kwh, r.capacity_kwh, r.usable_band_c);
        if let Some(t) = r.temp_c.as_mut() {
            *t = (*t + delta).clamp(r.usable_band_c.min, r.usable_band_c.max);
        }
        Ok(delta)
    }

    fn sink_context(&self, to: &TransferTarget) -> Result<SinkContext, Error> {
        match to {
            TransferTarget::Port { device_id, port_id } => {
                let sink = self.port_or_err(&PortRef {
                    device_id: device_id.clone(),
                    port_id: port_id.clone(),
                })?;
                if !sink.direction.can_sink() {
                    return Err(Error::WrongDirection {
                        device_id: sink.device_id.clone(),
                        port_id: sink.port_id.clone(),
                        direction: sink.direction,
                        needed: "sink",
                    });
                }
                Ok(SinkContext {
                    media: sink.media,
                    usable_temp_c: sink.usable_temp_c,
                    heated_reservoir_id: sink.attached_reservoir_id.clone(),
                })
            }
            TransferTarget::Reservoir { reservoir_id } => {
                let r = self.reservoir_or_err(reservoir_id)?;
                Ok(SinkContext {
                    media: r.media,
                    usable_temp_c: r.usable_band_c,
                    heated_reservoir_id: Some(r.id.clone()),
                })
            }
        }
    }

    fn port_or_err(&self, r: &PortRef) -> Result<&HeatPort, Error> {
        self.ports
            .get(&(r.device_id.clone(), r.port_id.clone()))
            .ok_or_else(|| Error::UnknownPort {
                device_id: r.device_id.clone(),
                port_id: r.port_id.clone(),
            })
    }

    fn reservoir_or_err(&self, id: &str) -> Result<&Reservoir, Error> {
        self.reservoirs
            .get(id)
            .ok_or_else(|| Error::UnknownReservoir(id.to_string()))
    }
}

struct SinkContext {
    media: Media,
    usable_temp_c: TempBandC,
    heated_reservoir_id: Option<String>,
}

fn cap_headroom(
    power_w: u32,
    reservoir_id: Option<&str>,
    remaining_w: &BTreeMap<String, f32>,
) -> u32 {
    let Some(id) = reservoir_id else {
        return power_w;
    };
    let Some(left) = remaining_w.get(id) else {
        return power_w;
    };
    if *left <= 0.0 {
        return 0;
    }
    power_w.min(*left as u32)
}

fn debit_headroom(reservoir_id: &str, power_w: u32, remaining_w: &mut BTreeMap<String, f32>) {
    if let Some(left) = remaining_w.get_mut(reservoir_id) {
        *left = (*left - power_w as f32).max(0.0);
    }
}
