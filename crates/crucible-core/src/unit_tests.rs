#[allow(unused_imports)]
use super::{IdDecodeError, IdKind, RunId};
use vstd::prelude::*;

verus! {

#[test]
fn run_id_envelope_unit_round_trip() {
    let source = String::from_str("unit-run-id");
    let envelope = RunId::new(source.clone()).to_envelope();
    assert(envelope.kind == IdKind::Run);

    match RunId::from_envelope(envelope) {
        Ok(id) => vstd::pervasive::runtime_assert(id.into_inner() == source),
        Err(IdDecodeError::UnsupportedSchemaVersion) | Err(IdDecodeError::WrongKind) => {
            vstd::pervasive::unreached()
        },
    }
}

} // verus!
