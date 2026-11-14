//! A log and the engine folded over it (ADR-0009 §3): the one place records are sealed,
//! validated, appended and applied, used by the WebAssembly boundary and the demo generator.

use crate::engine::{Engine, Output};
use crate::log::{Entry, Log, Record, compute_hash, parse_line, repair};

#[derive(Clone, Debug, Default)]
pub struct Session {
    pub log: Log,
    pub engine: Engine,
}

/// One appended entry with the outputs it produced.
pub type Event = (Entry, Vec<Output>);

impl Session {
    pub fn new() -> Self {
        Self::default()
    }

    /// Rebuild from two stored copies: repair (ADR-0002 §5), then replay (I5).
    pub fn load(a: &[String], b: &[String]) -> Result<(Session, Vec<Event>), String> {
        let lines = repair(a, b).map_err(|e| format!("{e:?}"))?.lines;
        let log = Log::load(&lines).map_err(|(i, e)| format!("line {i}: {e}"))?;
        let mut engine = Engine::new();
        let events = log.entries().iter().map(|e| (e.clone(), engine.apply(e))).collect();
        Ok((Session { log, engine }, events))
    }

    /// Validate before append: a rejected record never enters the log (ADR-0006 §7).
    pub fn submit(&mut self, r: Record) -> Result<Event, String> {
        self.engine.validate(&r).map_err(str::to_string)?;
        let e = self.log.append(r).map_err(|e| e.to_string())?.clone();
        let out = self.engine.apply(&e);
        Ok((e, out))
    }

    /// Wire format `met_ms|kind|k=v;...` in the log's own field grammar (ADR-0009 §2): sealed
    /// here and read back through the one log parser, so no second parser faces untrusted input.
    pub fn submit_wire(&mut self, wire: &str) -> Result<Event, String> {
        if wire.split('|').count() != 3 {
            return Err("expected met_ms|kind|fields".into());
        }
        let prev = self.log.entries().last().map_or(crate::log::GENESIS.to_string(), |e| e.hash.clone());
        let body = format!("v1|{}|{wire}", self.log.next_seq());
        let hash = compute_hash(&body, &prev);
        let e = parse_line(&format!("{body}|{prev}|{hash}")).map_err(|e| e.to_string())?;
        self.submit(e.record)
    }

    pub fn record(&self, met_ms: i64, kind: &str) -> Record {
        Record::new(self.log.next_seq(), met_ms, kind).expect("kind is a literal")
    }

    /// Simulated transport (ADR-0008 §1): acknowledge everything transmittable now.
    pub fn transmit(&mut self, met_ms: i64) -> Result<Vec<Event>, String> {
        let ids: Vec<u64> = self.engine.transmittable().iter().map(|t| t.0).collect();
        ids.into_iter()
            .map(|id| self.submit(self.record(met_ms, "sent").int("share", id as i64).expect("valid key")))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_records_are_sealed_validated_and_replayable() {
        let mut s = Session::new();
        s.submit_wire("0|setup|budget_days=30").unwrap();
        // Rejected by the engine: nothing appended.
        assert_eq!(s.submit_wire("0|setup|budget_days=3").unwrap_err(), "budget below minimum");
        // Rejected by the grammar: unsorted keys, text not hex.
        assert!(s.submit_wire("0|indicator|dir=1;decimals=0").is_err());
        assert!(s.submit_wire("0|indicator|key_txt=mood").is_err());
        assert!(s.submit_wire("0|setup").is_err());
        assert_eq!(s.log.entries().len(), 1);
        let lines = s.log.lines();
        let (again, events) = Session::load(&lines, &lines).unwrap();
        assert_eq!(again.log.lines(), lines);
        assert_eq!(events.len(), 1);
    }
}
