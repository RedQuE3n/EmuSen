# Mars (Nintendo 64)

The core is not started. What exists is a plan: [`Mars_Gameplan.md`](Mars_Gameplan.md), pinned 2026-09-15, which is the "what should I work on next" doc for Mars in the same role `Mercury_Gameplan.md` plays for Mercury — with the difference that it was written before any code rather than during it, and says where it expects to be wrong.

Two decisions are already taken and carry their reasoning there: the whole RCP is emulated at low level (§2.1), and correctness leads with performance as its own later phase (§2.2). §3 is the oracle question — what a Nintendo 64 core can be graded against — and it carries the one unverified assumption the phase order rests on.

[`Mars_References.md`](Mars_References.md) is the second page here: four GPL emulator codebases checked out for study, the procedural rule that keeps them read-only (§2 — they are read for mechanism, and the implementation is written from hardware documentation), what each is actually good for, and §5, which is the record of what they got *wrong*. That last section is the one that fed back into the plan.

[`Mars_TestOracle.md`](Mars_TestOracle.md) is what the plan's §3 was waiting on: the hardware test corpus can report a text verdict headlessly, so the phase order stands. It carries the protocol, the four things Mars must do before the ROM will say a word — two of which fail silently if got wrong — and the finding that the RDP has no hardware-grounded oracle anywhere, only a very good software one.

[`Mars_Documentation.md`](Mars_Documentation.md) is the assessed source list — which documents are authoritative, where they contradict each other, where one of them is outright wrong, and what an implementer needs that no document anywhere answers. Two gaps there shape the work: the RDP's subpixel-mask rule, and system timing in its entirety. It also settles the plan's §2.1 condition by showing the condition was posed wrongly.

[`Mars_Rom.md`](Mars_Rom.md) is the first page describing code rather than plans: Phase 0's container handling and header parser, which emulate nothing. Read §1.1 before touching the loader — the container is decided by the magic word and never by the extension, and the reason is a documented disagreement between sources about what the extensions mean.

The remaining per-component pages (`Mars_CPU.md`, `Mars_RSP.md`, `Mars_RDP.md`, and the rest) get added here as the phases that build those subsystems land, the same way `Venus - SNES/`'s and `Mercury - GB-GBC/`'s were.

See `Man pages/EmuSen_Core_Naming_Scheme.md` for the full core naming scheme, and `Man pages/Hardware/README.md` for how this documentation set is organized.
