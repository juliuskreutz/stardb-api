//! Per-UID stat recalculation adapters and their shared chronological scan engine.
//! Callers supply a connection so imports can persist pulls and stats in one transaction.

pub(crate) mod gi;
pub(crate) mod hsr;
pub(crate) mod zzz;

mod scan;
