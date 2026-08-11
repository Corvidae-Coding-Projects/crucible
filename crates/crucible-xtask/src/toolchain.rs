use vstd::prelude::*;

verus! {

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolName {
    Verus,
    CargoVerus,
    Z3,
    Verusfmt,
    Rustc,
    Cargo,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolchainError {
    PinMismatch,
    WrongProbeCount,
    DuplicateToolProbe,
    NonAbsoluteToolPath,
    ToolIdentityMismatch,
}

#[derive(Debug)]
pub struct ToolProbe {
    pub name: ToolName,
    pub absolute_path: Vec<u8>,
    pub version: Vec<u8>,
    pub binary_sha256: Vec<u8>,
}

impl ToolProbe {
    pub fn new(
        name: ToolName,
        absolute_path: Vec<u8>,
        version: Vec<u8>,
        binary_sha256: Vec<u8>,
    ) -> (probe: Self)
        ensures
            probe.name == name,
            probe.absolute_path@ == absolute_path@,
            probe.version@ == version@,
            probe.binary_sha256@ == binary_sha256@,
    {
        Self { name, absolute_path, version, binary_sha256 }
    }
}

fn bytes_equal(left: &[u8], right: &[u8]) -> (equal: bool)
    ensures
        equal == (left@ == right@),
{
    if left.len() != right.len() {
        return false;
    }
    let mut index = 0;
    while index < left.len()
        invariant
            index <= left@.len(),
            left@.len() == right@.len(),
            forall|prior: int| 0 <= prior < index ==> left@[prior] == right@[prior],
        decreases left.len() - index,
    {
        if left[index] != right[index] {
            assert(left@ != right@);
            return false;
        }
        index += 1;
    }
    assert(left@ =~= right@);
    true
}

pub fn validate_toolchain_lock(input: &[u8]) -> (result: Result<(), ToolchainError>) {
    let expected =
        b"crucible-toolchain\t1\ntool\tverus\t0.2026.08.09.92f466f\td97501a883931d1d173b1bf4b6cf4d973f16d105dbcb468e177b52b2331612d2\t2f5a41c553f424aacdd732339e9d125563716a0b003c27730f75d6f81a282cef\ntool\tcargo-verus\t0.2026.08.09.92f466f\t9e637927c66c48aa186217a3690d5bda11c8ffb71239c086ec8d6074f62625a9\t2f5a41c553f424aacdd732339e9d125563716a0b003c27730f75d6f81a282cef\ntool\tz3\t4.16.0\te583c4186a45e72411fa2cb2048401eed03f0f8e5f24694676a8f6271a50b765\t2f5a41c553f424aacdd732339e9d125563716a0b003c27730f75d6f81a282cef\ntool\tverusfmt\t0.7.2\t9f7566434ce5e9ccf16422b0c17ebe7d0af3a993fd40e7b7ceef1d6d217b1b47\tc5e0a8e07337055b2469d2878ddcc589da1f4be91348bb1917ce122ea46d4015\ntool\trustc\t1.97.1\td3a664c970a9fd8361b64194861bebc1ae37b9054e5ee3400dc1c9e691797eea\trustup:1.97.1-x86_64-unknown-linux-gnu\ntool\tcargo\t1.97.1\t828980723df339d62434390e9fb8ef8831036583343ae2316b7ab5646b5c1953\trustup:1.97.1-x86_64-unknown-linux-gnu\n";
    if bytes_equal(input, expected) {
        Ok(())
    } else {
        Err(ToolchainError::PinMismatch)
    }
}

fn probe_identity_matches(probe: &ToolProbe) -> bool {
    match probe.name {
        ToolName::Verus => {
            probe.version == b"0.2026.08.09.92f466f" && probe.binary_sha256
                == b"d97501a883931d1d173b1bf4b6cf4d973f16d105dbcb468e177b52b2331612d2"
        },
        ToolName::CargoVerus => {
            probe.version == b"0.2026.08.09.92f466f" && probe.binary_sha256
                == b"9e637927c66c48aa186217a3690d5bda11c8ffb71239c086ec8d6074f62625a9"
        },
        ToolName::Z3 => {
            probe.version == b"4.16.0" && probe.binary_sha256
                == b"e583c4186a45e72411fa2cb2048401eed03f0f8e5f24694676a8f6271a50b765"
        },
        ToolName::Verusfmt => {
            probe.version == b"0.7.2" && probe.binary_sha256
                == b"9f7566434ce5e9ccf16422b0c17ebe7d0af3a993fd40e7b7ceef1d6d217b1b47"
        },
        ToolName::Rustc => {
            probe.version == b"1.97.1" && probe.binary_sha256
                == b"d3a664c970a9fd8361b64194861bebc1ae37b9054e5ee3400dc1c9e691797eea"
        },
        ToolName::Cargo => {
            probe.version == b"1.97.1" && probe.binary_sha256
                == b"828980723df339d62434390e9fb8ef8831036583343ae2316b7ab5646b5c1953"
        },
    }
}

pub fn validate_tool_probes(probes: &[ToolProbe]) -> (result: Result<(), ToolchainError>) {
    if probes.len() != 6 {
        return Err(ToolchainError::WrongProbeCount);
    }
    let mut left = 0;
    while left < probes.len()
        invariant
            left <= probes@.len(),
        decreases probes.len() - left,
    {
        if probes[left].absolute_path.len() < 2 || probes[left].absolute_path[0] != b'/' {
            return Err(ToolchainError::NonAbsoluteToolPath);
        }
        if !probe_identity_matches(&probes[left]) {
            return Err(ToolchainError::ToolIdentityMismatch);
        }
        let mut right = left + 1;
        while right < probes.len()
            invariant
                left < probes@.len(),
                left < right <= probes@.len(),
            decreases probes.len() - right,
        {
            if probes[left].name == probes[right].name {
                return Err(ToolchainError::DuplicateToolProbe);
            }
            right += 1;
        }
        left += 1;
    }
    Ok(())
}

} // verus!
