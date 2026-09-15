# Mars (Nintendo 64)

The core is not started. What exists is a plan: [`Mars_Gameplan.md`](Mars_Gameplan.md), pinned 2026-09-15, which is the "what should I work on next" doc for Mars in the same role `Mercury_Gameplan.md` plays for Mercury — with the difference that it was written before any code rather than during it, and says where it expects to be wrong.

Two decisions are already taken and carry their reasoning there: the whole RCP is emulated at low level (§2.1), and correctness leads with performance as its own later phase (§2.2). §3 is the oracle question — what a Nintendo 64 core can be graded against — and it carries the one unverified assumption the phase order rests on.

[`Mars_References.md`](Mars_References.md) is the second page here: four GPL emulator codebases checked out for study, the procedural rule that keeps them read-only (§2 — they are read for mechanism, and the implementation is written from hardware documentation), what each is actually good for, and §5, which is the record of what they got *wrong*. That last section is the one that fed back into the plan.

Per-component pages (`Mars_CPU.md`, `Mars_RSP.md`, `Mars_RDP.md`, and the rest) get added here as the phases that build those subsystems land, the same way `Venus - SNES/`'s and `Mercury - GB-GBC/`'s were.

See `Man pages/EmuSen_Core_Naming_Scheme.md` for the full core naming scheme, and `Man pages/Hardware/README.md` for how this documentation set is organized.
