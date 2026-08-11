use crucible_xtask::{
    validate_tool_probes, validate_toolchain_lock, ToolName, ToolProbe, ToolchainError,
};

fn canonical_lock() -> Vec<u8> {
    concat!(
        "crucible-toolchain\t1\n",
        "tool\tverus\t0.2026.08.09.92f466f\td97501a883931d1d173b1bf4b6cf4d973f16d105dbcb468e177b52b2331612d2\t2f5a41c553f424aacdd732339e9d125563716a0b003c27730f75d6f81a282cef\n",
        "tool\tcargo-verus\t0.2026.08.09.92f466f\t9e637927c66c48aa186217a3690d5bda11c8ffb71239c086ec8d6074f62625a9\t2f5a41c553f424aacdd732339e9d125563716a0b003c27730f75d6f81a282cef\n",
        "tool\tz3\t4.16.0\te583c4186a45e72411fa2cb2048401eed03f0f8e5f24694676a8f6271a50b765\t2f5a41c553f424aacdd732339e9d125563716a0b003c27730f75d6f81a282cef\n",
        "tool\tverusfmt\t0.7.2\t9f7566434ce5e9ccf16422b0c17ebe7d0af3a993fd40e7b7ceef1d6d217b1b47\tc5e0a8e07337055b2469d2878ddcc589da1f4be91348bb1917ce122ea46d4015\n",
        "tool\trustc\t1.97.1\td3a664c970a9fd8361b64194861bebc1ae37b9054e5ee3400dc1c9e691797eea\trustup:1.97.1-x86_64-unknown-linux-gnu\n",
        "tool\tcargo\t1.97.1\t828980723df339d62434390e9fb8ef8831036583343ae2316b7ab5646b5c1953\trustup:1.97.1-x86_64-unknown-linux-gnu\n",
    )
    .as_bytes()
    .to_vec()
}

#[test]
fn canonical_toolchain_lock_is_consumed_exactly() {
    validate_toolchain_lock(&canonical_lock()).expect("canonical lock");
}

#[test]
fn altered_toolchain_digest_is_rejected() {
    let mut lock = canonical_lock();
    let digest_byte = lock
        .iter()
        .position(|byte| *byte == b'd')
        .expect("digest byte");
    lock[digest_byte] = b'0';
    assert_eq!(
        validate_toolchain_lock(&lock).unwrap_err(),
        ToolchainError::PinMismatch
    );
}

#[test]
fn probes_require_absolute_paths_exact_versions_and_binary_digests() {
    let probes = vec![
        ToolProbe::new(
            ToolName::Verus,
            b"/opt/verus".to_vec(),
            b"0.2026.08.09.92f466f".to_vec(),
            b"d97501a883931d1d173b1bf4b6cf4d973f16d105dbcb468e177b52b2331612d2".to_vec(),
        ),
        ToolProbe::new(
            ToolName::CargoVerus,
            b"/opt/cargo-verus".to_vec(),
            b"0.2026.08.09.92f466f".to_vec(),
            b"9e637927c66c48aa186217a3690d5bda11c8ffb71239c086ec8d6074f62625a9".to_vec(),
        ),
        ToolProbe::new(
            ToolName::Z3,
            b"/opt/z3".to_vec(),
            b"4.16.0".to_vec(),
            b"e583c4186a45e72411fa2cb2048401eed03f0f8e5f24694676a8f6271a50b765".to_vec(),
        ),
        ToolProbe::new(
            ToolName::Verusfmt,
            b"/opt/verusfmt".to_vec(),
            b"0.7.2".to_vec(),
            b"9f7566434ce5e9ccf16422b0c17ebe7d0af3a993fd40e7b7ceef1d6d217b1b47".to_vec(),
        ),
        ToolProbe::new(
            ToolName::Rustc,
            b"/opt/rustc".to_vec(),
            b"1.97.1".to_vec(),
            b"d3a664c970a9fd8361b64194861bebc1ae37b9054e5ee3400dc1c9e691797eea".to_vec(),
        ),
        ToolProbe::new(
            ToolName::Cargo,
            b"/opt/cargo".to_vec(),
            b"1.97.1".to_vec(),
            b"828980723df339d62434390e9fb8ef8831036583343ae2316b7ab5646b5c1953".to_vec(),
        ),
    ];

    validate_tool_probes(&probes).expect("exact probes");

    let mut wrong = probes;
    wrong[0] = ToolProbe::new(
        ToolName::Verus,
        b"relative/verus".to_vec(),
        b"0.2026.08.09.92f466f".to_vec(),
        b"d97501a883931d1d173b1bf4b6cf4d973f16d105dbcb468e177b52b2331612d2".to_vec(),
    );
    assert_eq!(
        validate_tool_probes(&wrong).unwrap_err(),
        ToolchainError::NonAbsoluteToolPath
    );
}
