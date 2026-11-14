//! Transmission layer (ADR-0008): the engine authorises, the transport executes.
//! Every share is an outbox item whose `ShareQueued` output is the sharing-log entry (I6);
//! routing honours exclusions without exception (I10) and reachability for safety (F5).

use crate::engine::Output;
use crate::log::is_key;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ShareClass {
    Safety,
    Summary,
    Ppc,
    Crew,
}

impl ShareClass {
    pub const ALL: [ShareClass; 4] = [ShareClass::Safety, ShareClass::Summary, ShareClass::Ppc, ShareClass::Crew];

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|c| c.as_str() == s)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            ShareClass::Safety => "safety",
            ShareClass::Summary => "summary",
            ShareClass::Ppc => "ppc",
            ShareClass::Crew => "crew",
        }
    }

    /// Closed payload schema per class (ADR-0008 §2): nothing else can ever leave the device.
    pub fn allowed_keys(self) -> &'static [&'static str] {
        match self {
            ShareClass::Safety => &["share", "day", "lines", "value", "fault"],
            ShareClass::Summary => &["share", "ind", "day", "days", "mean_z_e3", "alarm"],
            ShareClass::Ppc => &["share", "day", "ppc"],
            ShareClass::Crew => &["share", "day", "crew"],
        }
    }

    /// Which recipients a class may go to (ADR-0008 §3).
    fn accepts(self, crew: bool) -> bool {
        match self {
            ShareClass::Ppc => !crew,
            ShareClass::Crew => crew,
            ShareClass::Safety | ShareClass::Summary => true,
        }
    }
}

#[derive(Clone, Debug)]
struct Item {
    class: ShareClass,
    to: Option<String>,
    payload: String,
    /// Indicator of a summary, so a consent withdrawal can cancel it.
    ind: Option<String>,
    sent: bool,
}

#[derive(Clone, Debug, Default)]
pub struct Outbox {
    /// key -> on board (true) or ground (false)
    recipients: BTreeMap<String, bool>,
    routes: BTreeMap<ShareClass, Vec<String>>,
    excluded: BTreeSet<String>,
    link_up: bool,
    items: BTreeMap<u64, Item>,
    next: u64,
}

impl Outbox {
    pub fn validate_recipient(&self, key: &str, crew: i64) -> Result<(), &'static str> {
        if !is_key(key) || self.recipients.contains_key(key) {
            return Err("invalid or duplicate recipient");
        }
        if matches!(crew, 0 | 1) { Ok(()) } else { Err("crew must be 0 or 1") }
    }

    pub fn add_recipient(&mut self, key: &str, crew: bool) {
        self.recipients.insert(key.to_string(), crew);
    }

    pub fn validate_route(&self, class: &str, to: &str) -> Result<(), &'static str> {
        let class = ShareClass::parse(class).ok_or("unknown share class")?;
        let crew = *self.recipients.get(to).ok_or("unknown recipient")?;
        if !class.accepts(crew) {
            return Err("recipient kind not allowed for this class");
        }
        if self.routes.get(&class).is_some_and(|r| r.iter().any(|t| t == to)) {
            return Err("recipient already in route");
        }
        Ok(())
    }

    pub fn add_route(&mut self, class: &str, to: &str) {
        let class = ShareClass::parse(class).expect("validated");
        self.routes.entry(class).or_default().push(to.to_string());
    }

    /// Checked at the first day_close: a single exclusion must never empty the safety route.
    pub fn ready(&self) -> Result<(), &'static str> {
        if self.routes.get(&ShareClass::Safety).map_or(0, Vec::len) < 2 {
            return Err("safety route needs at least 2 recipients");
        }
        Ok(())
    }

    pub fn is_recipient(&self, key: &str) -> bool {
        self.recipients.contains_key(key)
    }

    pub fn is_excluded(&self, key: &str) -> bool {
        self.excluded.contains(key)
    }

    pub fn link_up(&self) -> bool {
        self.link_up
    }

    /// Not excluded and reachable now (on board always; ground only while the link is up).
    fn deliverable(&self, r: &str) -> bool {
        !self.excluded.contains(r) && (self.recipients[r] || self.link_up)
    }

    /// First eligible recipient. Only safety skips unreachable ground recipients; other classes
    /// wait for the preferred one rather than override the astronaut's choice (ADR-0008 §4).
    fn resolve(&self, class: ShareClass) -> Option<String> {
        let route = self.routes.get(&class)?;
        if class == ShareClass::Safety {
            route.iter().find(|r| self.deliverable(r)).cloned()
        } else {
            route.iter().find(|r| !self.excluded.contains(*r)).cloned()
        }
    }

    pub fn queue(&mut self, class: ShareClass, fields: &[(&str, String)], ind: Option<&str>) -> Vec<Output> {
        let share = self.next;
        self.next += 1;
        let mut parts = vec![format!("share={share}")];
        parts.extend(fields.iter().map(|(k, v)| format!("{k}={v}")));
        debug_assert!(fields.iter().all(|(k, _)| class.allowed_keys().contains(k)));
        let payload = parts.join(";");
        let to = self.resolve(class);
        let mut out = vec![Output::ShareQueued { share, class: class.as_str(), to: to.clone(), payload: payload.clone() }];
        if to.is_none() {
            out.push(Output::ShareUndeliverable { share, class: class.as_str() });
        }
        self.items.insert(share, Item { class, to, payload, ind: ind.map(str::to_string), sent: false });
        out
    }

    fn reresolve(&mut self) -> Vec<Output> {
        let mut out = Vec::new();
        let ids: Vec<u64> = self.items.iter().filter(|(_, i)| !i.sent).map(|(id, _)| *id).collect();
        for id in ids {
            let class = self.items[&id].class;
            let new = self.resolve(class);
            let item = self.items.get_mut(&id).expect("listed");
            if item.to != new {
                item.to = new.clone();
                out.push(match new {
                    Some(to) => Output::ShareRerouted { share: id, to },
                    None => Output::ShareUndeliverable { share: id, class: class.as_str() },
                });
            }
        }
        out
    }

    pub fn set_excluded(&mut self, key: &str, on: bool) -> Vec<Output> {
        if on {
            self.excluded.insert(key.to_string());
        } else {
            self.excluded.remove(key);
        }
        let mut out = vec![Output::ExclusionChanged { recipient: key.to_string(), excluded: on }];
        if on {
            for (class, route) in &self.routes {
                if route.iter().any(|r| r == key) && route.iter().all(|r| self.excluded.contains(r)) {
                    out.push(Output::RouteEmpty { class: class.as_str() });
                }
            }
        }
        out.extend(self.reresolve());
        out
    }

    pub fn set_link(&mut self, up: bool) -> Vec<Output> {
        self.link_up = up;
        self.reresolve()
    }

    /// A transmission is acknowledged only for an authorised, unsent item whose recipient is
    /// still eligible: I6 and I10 hold for any log, not just for well-behaved transports.
    pub fn validate_sent(&self, share: u64) -> Result<(), &'static str> {
        let item = self.items.get(&share).ok_or("no such share")?;
        if item.sent {
            return Err("share already sent");
        }
        let to = item.to.as_deref().ok_or("share has no eligible recipient")?;
        if self.deliverable(to) { Ok(()) } else { Err("recipient excluded or unreachable") }
    }

    /// What the transport may transmit now: unsent items whose recipient is deliverable.
    pub fn transmittable(&self) -> Vec<(u64, &str, &str)> {
        self.items
            .iter()
            .filter_map(|(id, i)| match (&i.to, i.sent) {
                (Some(to), false) if self.deliverable(to) => Some((*id, to.as_str(), i.payload.as_str())),
                _ => None,
            })
            .collect()
    }

    pub fn mark_sent(&mut self, share: u64) -> Output {
        let item = self.items.get_mut(&share).expect("validated");
        item.sent = true;
        Output::ShareSent { share, to: item.to.clone().expect("validated") }
    }

    /// Tier-2 withdrawal cancels unsent summaries of that indicator (ADR-0008 §5).
    pub fn cancel_summaries(&mut self, ind: &str) -> Vec<Output> {
        let ids: Vec<u64> = self
            .items
            .iter()
            .filter(|(_, i)| !i.sent && i.class == ShareClass::Summary && i.ind.as_deref() == Some(ind))
            .map(|(id, _)| *id)
            .collect();
        ids.into_iter()
            .map(|share| {
                self.items.remove(&share);
                Output::ShareCancelled { share }
            })
            .collect()
    }
}
