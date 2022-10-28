// SPDX-FileCopyrightText: 2022 Kent Gibson <warthog618@gmail.com>
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

// ALL IT as can't construct a Chip without opening a GPIO file.
//
// Assumptions:
//  - kernel supports uAPI versions corresponding to selected build features

use gpiosim::Bank;

#[test]
fn find_named_line() {
    let sim = gpiosim::builder()
        .with_bank(
            Bank::new(8, "find_line 1")
                .name(3, "fl banana")
                .name(6, "fl apple"),
        )
        .with_bank(
            Bank::new(42, "find_line 2")
                .name(3, "fl piñata")
                .name(4, "fl piggly")
                .name(5, "fl apple"),
        )
        .live()
        .unwrap();

    let l = gpiocdev::find_named_line("fl banana").unwrap();
    assert_eq!(l.chip, sim.chips()[0].dev_path);
    assert_eq!(l.info.offset, 3);
    assert_eq!(l.offset, l.info.offset);

    let l = gpiocdev::find_named_line("fl piggly").unwrap();
    assert_eq!(l.chip, sim.chips()[1].dev_path);
    assert_eq!(l.info.offset, 4);
    assert_eq!(l.offset, l.info.offset);

    let l = gpiocdev::find_named_line("fl apple").unwrap();
    // depending on how other tests are running, the order of the sim chips is not 100% predictable.
    if sim.chips()[0].dev_path < sim.chips()[1].dev_path {
        assert_eq!(l.chip, sim.chips()[0].dev_path);
        assert_eq!(l.info.offset, 6);
    } else {
        assert_eq!(l.chip, sim.chips()[1].dev_path);
        assert_eq!(l.info.offset, 5);
    }
    assert_eq!(l.offset, l.info.offset);

    assert!(gpiocdev::find_named_line("fl nada").is_none())
}

#[test]
fn find_named_lines() {
    let sim = gpiosim::builder()
        .with_bank(
            Bank::new(8, "find_lines 1")
                .name(3, "fls banana")
                .name(6, "fls apple"),
        )
        .with_bank(
            Bank::new(42, "find_lines 2")
                .name(3, "fls piñata")
                .name(4, "fls piggly")
                .name(5, "fls apple"),
        )
        .live()
        .unwrap();

    let found = gpiocdev::find_named_lines(&["fls banana"], true).unwrap();
    assert_eq!(found.len(), 1);
    let l = found.get(&"fls banana").unwrap();
    assert_eq!(*l.chip, sim.chips()[0].dev_path);
    assert_eq!(l.info.offset, 3);
    assert_eq!(l.offset, l.info.offset);

    let found = gpiocdev::find_named_lines(&["fls piggly"], true).unwrap();
    assert_eq!(found.len(), 1);
    let l = found.get(&"fls piggly").unwrap();
    assert_eq!(*l.chip, sim.chips()[1].dev_path);
    assert_eq!(l.info.offset, 4);
    assert_eq!(l.offset, l.info.offset);

    let found = gpiocdev::find_named_lines(&["fls apple"], false).unwrap();
    assert_eq!(found.len(), 1);
    let l = found.get(&"fls apple").unwrap();
    if sim.chips()[0].dev_path < sim.chips()[1].dev_path {
        assert_eq!(*l.chip, sim.chips()[0].dev_path);
        assert_eq!(l.info.offset, 6);
    } else {
        assert_eq!(*l.chip, sim.chips()[1].dev_path);
        assert_eq!(l.info.offset, 5);
    }
    assert_eq!(l.offset, l.info.offset);

    let found = gpiocdev::find_named_lines(&["fls apple"], true);
    assert_eq!(
        found,
        Err(gpiocdev::Error::NonuniqueLineName("fls apple".to_string()))
    );

    let found = gpiocdev::find_named_lines(&["fls banana", "fls piggly"], true);
    assert_eq!(found, Err(gpiocdev::Error::DistributedLines()));

    let found = gpiocdev::find_named_lines(&["fl nada"], true).unwrap();
    assert_eq!(found.len(), 0);

    let found =
        gpiocdev::find_named_lines(&["fls apple", "fls banana", "fls nada"], false).unwrap();
    let l = found.get(&"fls banana").unwrap();
    assert_eq!(*l.chip, sim.chips()[0].dev_path);
    assert_eq!(l.info.offset, 3);
    let l = found.get(&"fls apple").unwrap();
    if sim.chips()[0].dev_path < sim.chips()[1].dev_path {
        assert_eq!(*l.chip, sim.chips()[0].dev_path);
        assert_eq!(l.info.offset, 6);
    } else {
        assert_eq!(*l.chip, sim.chips()[1].dev_path);
        assert_eq!(l.info.offset, 5);
    }
    assert!(found.get(&"fls nada").is_none());
}

#[test]
fn detect_abi_version() {
    // assumes a kernel with both v1 and v2 supported.

    // to ensure there is at least one chip
    let sim = gpiosim::simpleton(4);

    #[cfg(feature = "uapi_v2")]
    assert_eq!(gpiocdev::detect_abi_version(), Ok(gpiocdev::AbiVersion::V2));
    #[cfg(not(feature = "uapi_v2"))]
    assert_eq!(gpiocdev::detect_abi_version(), Ok(gpiocdev::AbiVersion::V1));

    drop(sim);
}

#[test]
fn supports_abi_version() {
    // assumes a kernel with both v1 and v2 supported.

    // to ensure there is at least one chip
    let sim = gpiosim::simpleton(4);

    #[cfg(feature = "uapi_v1")]
    assert_eq!(
        gpiocdev::supports_abi_version(gpiocdev::AbiVersion::V1),
        Ok(())
    );
    #[cfg(not(feature = "uapi_v1"))]
    assert_eq!(
        gpiocdev::supports_abi_version(gpiocdev::AbiVersion::V1),
        Err(gpiocdev::Error::UnsupportedAbi(
            gpiocdev::AbiVersion::V1,
            gpiocdev::AbiSupportKind::Library
        ))
    );
    #[cfg(feature = "uapi_v2")]
    assert_eq!(
        gpiocdev::supports_abi_version(gpiocdev::AbiVersion::V2),
        Ok(())
    );
    #[cfg(not(feature = "uapi_v2"))]
    assert_eq!(
        gpiocdev::supports_abi_version(gpiocdev::AbiVersion::V2),
        Err(gpiocdev::Error::UnsupportedAbi(
            gpiocdev::AbiVersion::V2,
            gpiocdev::AbiSupportKind::Library
        ))
    );

    drop(sim);
}
