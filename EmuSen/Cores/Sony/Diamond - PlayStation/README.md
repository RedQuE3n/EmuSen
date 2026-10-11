# Diamond (Sony PlayStation)

The PlayStation core in Rust, crate `diamond`, on the core ABI v1, marked in development (`EmuSen_CoreAPI.md` §27) so
that no player is offered it before its gate. Its processors are the BlackMoon crates in `../BlackMoon-HW/`.

At stage 1 (2026-10-11) it recognises a PS-EXE or a disc, reports the system `psx` with its market, memories,
processors, pads and firmware, and shows a black picture with silence at the SPU's 44,100 Hz; nothing is emulated
yet. The plan is `Man pages/Hardware/Sony/Diamond - PlayStation/Diamond_Plan.md`, the build record
`Diamond_Native.md` beside it.

Black Moon Clan codename (Prince Demande/Diamond); see `Man pages/EmuSen_Core_Naming_Scheme.md` §4. The crate is
named plainly, without the `RT` suffix of the Rust ports, because it has no C# predecessor to be told from
(`EmuSen_Core_Naming_Scheme.md` §3).
