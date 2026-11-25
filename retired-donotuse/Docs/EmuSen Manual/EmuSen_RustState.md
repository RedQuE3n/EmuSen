# EmuSen — the Rust cores' save state, in `emusen-native`

*Written 2026-09-28.* This page covers the save state in `EmuSen/Cores/Shared/emusen-native/`, the crate every Rust
core builds from. The crate is named for what it will hold, not for what it holds today: `EmuSen_NativeCores.md` §9 Q1
decided that the common native interface goes into it later (§3.17 there), and that is not built. It is the Rust counterpart of `EmuSen/Common/StateSerializer.cs` (`EmuSen_Save_States.md`), and it exists
for the same reason that file does: each core describes its own fields, and one shared implementation turns them into
bytes. A Rust core's state is its C# core's state, byte for byte (`Mars_Native.md` §5.1, `Mercury_Native.md` §3.1,
`Moon_Native.md` §3.1). This crate is how that stays true for a new core without copying a codec into it.

---

## 1. Why it exists, and what it replaced

**Decided 2026-09-28: every Rust core takes the state format, its naming rule, `Skip<T>` and the state half of its C
ABI from one crate**, just as every C# core takes them from `StateSerializer`. Before this, each of MarsRT, MercuryRT
and MoonRT carried its own copy:

- `state.rs`: 410, 493 and 324 lines;
- `naming.rs`: 101, 85 and 90;
- `Skip<T>`: 26 lines in each `lib.rs`;
- a local `status()`, `input()` and layout copy in each `ffi/mod.rs`.

The copies had diverged only by additions: MercuryRT added strings and format versions, MoonRT added `group_class`,
`present` and raw identifiers in the naming rule. No copy disagreed with another on a byte. That made the extraction a
union, not a reconciliation. It is also why `Moon_Native.md` §5 Q2's threshold ("when a third core makes the shape a
rule instead of a coincidence") was met by MoonRT.

The three cores' sources lost 1,624 lines and gained 260, 51 of them MarsRT's new digest test. The crate is 966 lines,
300 of them its tests.

## 2. What is in it

| Item | What it is |
|---|---|
| `State` | One C# class or struct. `write_state` and `read_state` walk its fields in the C# ordinal order. `type Error` is the core's own error, which must convert from `Truncated`. |
| `StateWriter` | Writes into a caller's buffer (`new`), only counts (`counter`), or also lists the layout (`layout`), one `offset length type path` line per field. Its primitives are `u8`, `i8`, `bool`, `i16`, `u16`, `i32`, `u32`, `i64`, `u64`, `bytes`, `bools`, `u16s`, `i32s`, `u32s`, `u64s` and `string`, plus `class`, `group_class`, `structure`, `structures` and `group`. |
| `StateReader` | Reads what either writer made, with `BinaryReader`'s readings of bytes no writer makes: any nonzero bool is true, and a class flag of 0 leaves the object as it was. Also has `skip`, `position`, `present`, and `set_version`/`before` for formats with retired fields. |
| `Truncated`, `StringError` | The reader's own errors. Each core converts them into its own. |
| `Skip<T>` | A field C# marks `[SkipInState]`. It dereferences to its value and never makes two machines unequal. |
| `naming::snake`, `naming::check` | The rule that each Rust field is the snake_case of the C# name written beside it, and the scan of a core's sources that enforces it. |
| `ffi::status` | The status codes the crate owns (§3). |
| `ffi::{Status, result, input, copy_text}` | The helpers every export uses. |
| `ffi::StateMachine`, `state_exports!` | A machine whose whole state is one walk, and the four exports over it (load, size, save, layout) under names the core chooses. |
| `SampleQueue` | C#'s `EmuSen.Common.SampleQueue`: a core's undrained stereo samples. `push_pair` queues a pair, then drops the oldest pairs until the queue is within its limit. `drain` takes whole pairs only. `set_limit` takes hold at the next pair. Added 2026-09-30; see §2.1. It is not part of any state. |

**What stays in a core:**

- its magic and version;
- its `StateError`, with the refusals only its format has;
- its structs and their walks;
- its exports when they do not fit the macro's shape. MarsRT's take a snapshot flag and settle the clocks before a
  save, so MarsRT uses the helpers and keeps its own exports;
- anything one console alone has. MarsRT's `boxed` is an example.

**Adding a core:**

- `emusen-native = { path = "../../Shared/emusen-native" }` in its `Cargo.toml`;
- `impl State` for each serialized class, with `type Error = StateError`;
- `impl From<Truncated> for StateError`, and `ffi::Status` for it;
- `StateMachine` for the machine, and one `state_exports!` line;
- a `naming.rs` of one test calling `naming::check` over its modules.

### 2.1 The audio queue (added 2026-09-30)

MoonRT and MercuryRT each carried the same queue in their mixers:

- a `VecDeque<i16>` and a limit;
- the push of a pair, with the trim loop that follows it;
- the whole-pair drain.

The two copies differed only in the queue's initial capacity. They are now one `SampleQueue`, and each core's
`*_machine_set_audio_limit` and `*_machine_audio_buffered` exports call it. The C# twin, which the host drives
through `AudioSettings.AudioBufferMaxSamples`, is `EmuSen/Common/SampleQueue.cs`
(`EmuSen_Settings_Reference.md` §4.85.2).

**What proves nothing changed (measured 2026-09-30):**

- MercuryRT: its platform cases hold its sound to the SHA-256 digests recorded on linux-x64 before the change, byte
  for byte, on all four programs. Its machine cases still match the C# Mercury's samples every frame.
- MoonRT: `MoonRtSoundAndPictureTests` compares its queue with the C# one under limits of 0, 3, 500, 1,001, 2,000 and
  128,000, with the queue left undrained for up to seven frames (`Moon_Native.md` §8.2).

**MarsRT is not moved.** Its audio interface (`memory/ai.rs`) is a different shape, and its Rust stays as it is until
the PGO profile is retrained. It joins at its migration step. The crate's new module is not reached from MarsRT's code.
This was measured on 2026-09-30. MarsRT was built in release twice from the same path, once with the crate as it was
and once with the queue added. The two libraries' disassemblies are the same instruction for instruction, once
addresses and the crate hash in mangled names are set aside. Their `.text`, `.rodata` and `.data` sections are the
same size. The only other difference is `samples.rs` in the line tables' file list. The files are not byte-identical,
because a change to a crate's source changes the hash inside every mangled name. So byte identity is not the test
here.

## 3. The status codes

The codes were per-core, and two meanings had collided: −5 was "a string length the C# reader refuses" in MercuryRT
and "an RDRAM size that is neither 4 MB nor 8 MB" in MarsRT. The crate now owns −1 to −8, and a core's own codes run
from −9 to −255 (`ffi::status::FIRST_CORE` and `LAST_CORE`), the band `EmuSen_NativeCores.md` §3.3 gives the cores.

| Code | Meaning | Owner |
|---|---|---|
| −1 | A null handle or buffer | shared |
| −2 | The input ended before a field | shared |
| −3 | Not this core's state: the magic is another's | shared |
| −4 | A version this core does not read | shared |
| −5 | A string length `BinaryReader` refuses | shared |
| −6 | Reserved | — |
| −7 | The caller's buffer is shorter than the state | shared |
| −8 | Reserved | — |
| −9 to −255 | Each core's own: bad images, spaces, boards, faults | the core |
| −256 and below | The common interface's and reproduced C# exceptions', planned (`EmuSen_NativeCores.md` §3.3) | not yet built |

**The renumbering touched MarsRT alone.** MercuryRT's and MoonRT's codes were already inside the rule. MarsRT's three
state refusals of its own moved:

- the RDRAM size from −5 to −12;
- the pending-word overflow from −6 to −13;
- pending words outside a snapshot from −8 to −14.

`MarsMachine.Describe` maps the new numbers, and the interface version went from 9 to 10 in both `lib.rs` and
`MarsNative.InterfaceVersion`, so a library and a shim of different numberings refuse each other. No state file
changes: the codes are return values, never data.

**Not covered.** MarsRT's VI test ABI (`ffi/vi.rs`) returns −2 for a buffer of the wrong size. It is not a state
function, so the rule does not reach it, and it was left alone.

## 4. What was checked, and how (measured 2026-09-28)

The claim is that the extraction changed no behaviour.

- **Every existing oracle passes unchanged**, with the counts below. The crate tests that moved into `emusen-native`
  are counted there.
- **Every library exports what it exported**: `nm -D` lists 85 (MarsRT), 35 (MercuryRT) and 22 (MoonRT) functions,
  identical to the base, `cd6bc71e`.
- **MarsRT's frame and state are unchanged**: interleaved timing and the state hashes of §4.2.

### 4.1 The oracles

| Suite | Base (`cd6bc71e`) | With the crate |
|---|---|---|
| MarsRT's crate tests | 491 | 490: two codec tests and the naming rule's examples moved out, two digest tests added |
| MercuryRT's crate tests | 22 + 16 | 17 + 16: four codec tests and the rule's examples moved out |
| MoonRT's crate tests | 10 | 6: three codec tests and the rule's examples moved out |
| `emusen-native`'s own tests | — | 15 |
| WiseMan, Release, with every game's state present | — | 966 of 966 |

- The WiseMan run took 1 h 39 m, and every case passed. It covered:
  - MarsRT's C#-state round trips and game comparisons, with SM64, OoT, GoldenEye and DK64 states from
    `EMUSEN_MARSRT_STATES`;
  - MercuryRT's state, machine, debug and defect tests, with four scratch games;
  - MoonRT's 71 state cases;
  - the Moon debug target's 28;
  - AccuracyCoin's two.
- Thirteen of the 966 printed that they did not run. They are RDP command streams, benches and corpora whose inputs
  were not given, and none of them is a state oracle.
- A first run of the same filter on the Debug build was stopped after two hours, before it reported, and replaced by
  this Release one. It is not counted.

### 4.2 MarsRT's frame and state

**The state hashes are the base's.** Each build ran `marsrt-pgo <rom> <state> 600 mistress=1 warmup=120` once per game,
which is Mistress's frame loop with a rewind snapshot four times a second. The three builds were the base, the change,
and the base with PGO turned off.

| Game | Base | Change | Base without PGO |
|---|---|---|---|
| Super Mario 64 | `8553FCCA132C724A` | `8553FCCA132C724A` | `8553FCCA132C724A` |
| Ocarina of Time | `D1382AB72E567556` | `D1382AB72E567556` | `D1382AB72E567556` |

**A save's bytes are the base's.** A bench (`savebench`, in the scratch directory) built MarsRT as a library from each
tree, loaded each game's state, and saved it both as a state and as a snapshot. All four outputs are byte-identical
between the trees (FNV-1a over the bytes): 12,724,733 and 12,755,456 bytes for the states, and 12,986,881 and
13,017,604 for the snapshots.

**The timing is not measured yet.** The interleaved rounds were started on 2026-09-28 and stopped after one run: a game
was running on the desktop at about five cores, and the quiet gate had waited its full five minutes. The hashes above
were taken at `nice 19`, which a load does not affect. The rounds are owed, with the script and all three builds ready
in the scratch directory (`bench.sh`, `bin-base`, `bin-change`, `bin-basenopgo`). The prediction to be retired by them
is **P1** in §4.3.

### 4.3 The PGO profile

MarsRT is built profile-guided (`Mars_Native.md` §6.17). Moving a function into another crate renames it, so the moved
functions lose their guidance. They are cold: a state is saved four times a second in Mistress's rewind, never per
frame.

**The digest.** `build.rs`'s source digest covered MarsRT's own `src/`, `Cargo.toml` and `Cargo.lock`. After the move,
an edit in `emusen-native` would have left the verdict at `matched` while changing the code the profile was trained on.
The digest now also covers every `path = "..."` dependency's `Cargo.toml` and `src/`, tests left out. Its files are
named relative to the crate, so the value does not depend on where the checkout is.

- The function lives in `pgo/digest.rs`, which `build.rs` includes.
- `src/tests/pgo_digest.rs` includes the same file and pins three behaviours:
  - an edit in a dependency moves the digest;
  - an edit in a test file does not;
  - MarsRT names the state crate as a path dependency, at a path that exists.

**What moved, measured by building both trees with the profile and LLVM's warnings.** Both builds used
`-Cprofile-use` and `-pgo-warn-missing-function`, and neither was given `-no-pgo-warn-mismatch`.

| | Base | Change |
|---|---|---|
| MarsRT's functions with no profile data | 22 | 1,193 |
| MarsRT's functions whose record was discarded (a hash mismatch) | 23 | 4 |
| `emusen-native`'s functions with no profile data | — | 58 |

**The profile guides nothing in MarsRT any more. The cause is not the moved functions but the new dependency.**

- Every function's name carries its crate's disambiguator, which hashes what cargo passes as `-C metadata`.
  `Mars_Native.md` §6.17.1 records that cargo's metadata hashes the package's dependencies.
- The profile names MarsRT's functions under three disambiguators, one per trained flavour, 1,195 functions each. The
  base's `dist` build is `CsjrJNaGb5TN3_6marsrt`, one of the three.
- With the dependency, the same build named them `CslNauWps3kSY_6marsrt`. That was measured while the crate was still
  called `emusen-state`; after its rename, the published library's symbols read `Cs2b562MR2kMn_6marsrt`. The profile
  holds neither, so no MarsRT function finds its record. The counts above are from the first name.

The prediction written into this step, that "the moved functions are cold, so moving them costs nothing", named the
wrong mechanism. It holds for the functions moved, and it misses that adding the dependency renames all the others.

- **The verdict reads `stale`, and understates this.** `build.rs` compares source digests; it cannot see a
  disambiguator change. The change's source digest is `dbc1f949edc3d0f6`, the base's `dbcf0e9632091207`, and the
  profile was trained on `36a5237755f1c652`. So the base was already `stale`, and the change is `stale` too. A build
  that finds no record for any function would be better called `untrained`, and the file cannot say so.
- **P1, to be retired by §4.2's owed rounds:** the change runs about as fast as the base without PGO, **7–9% slower
  than the base** on SM64 and OoT, which is §6.17.5's measured gain. The unguided base is in the rounds so that the
  loss can be attributed.
- **The remedy is a retrain** (`pgo/train.sh`), not a change to the crate. Any new dependency, or a new version of one,
  renames MarsRT's functions again, so adding one belongs in §6.17.6's list of reasons to retrain. A retrain is a
  separate decision and was not done here.

## 5. Mutants

**Thirteen mutants, all caught** (measured 2026-09-28). The runner (`mutants.py`, in
`~/.cache/emusen/probe/moonrt/state-crate/`) applies one edit to the crate. It then runs the crate's tests and
MoonRT's and MercuryRT's crate tests, builds those two libraries from the edit, and runs WiseMan's `MoonRtState` and
`MercuryRtState` filters against them with the real games. Then it restores the file and the libraries. MarsRT's
tests are not in the round: a MarsRT build takes minutes per mutant. Its only share of these primitives is the bool
and `u16`/`u32`/`u64` arrays, the structures and the reader's `skip`.

| Mutant | Caught by |
|---|---|
| M1 a bool read as `== 1` | crate (class flags); WiseMan's odd-bytes cases of both cores and MoonRT's class-flag case |
| M2 a counter adds nothing for a bool array | **survived the round**; now the crate's layout test, which writes one |
| M3 an `i32` array read big-endian | crate; MoonRT's crate round trip; WiseMan's running, noise and games cases (MoonRT, whose boards keep `int` arrays) |
| M4 a string of 127 bytes given a two-byte prefix | crate only; the cores' strings are all shorter |
| M5 a fifth length byte up to `0x1F` accepted | crate only; no C# writer makes such a length |
| M6 a class read whatever its flag | crate; WiseMan's class-flag case |
| M7 a class written with a flag of 0 | **the cores only in the round** (MoonRT's and MercuryRT's crates, eight WiseMan cases); now also the crate's layout test, which checks the bytes |
| M8 array elements listed without their index | crate only; MarsRT alone lists `structures` |
| M9 a state of the named version read as older | crate only; the version-5 cases that would see it need `EMUSEN_MERCURY_V5_STATES` |
| M10 the backing field's name kept whole | crate; MercuryRT's naming scan |
| M11 a read's field sought after the first owner only | crate only; MarsRT alone names a second owner (`machine.`) |
| M12 two skipped fields compare unequal | crate; both cores' crate tests that compare a machine after a refused load with the one before it |
| M13 the layout export returns the bytes copied, not the whole length | crate's export test only; the C# shims ask for the length first with a null buffer, which the mutant does not touch |

**Where only the crate stands.** Six mutants (M4, M5, M8, M9, M11, M13) are caught by the crate's tests alone. Each is
a path no core's current state reaches: a long string, an invalid string, a listed array of structures, an older
version, a second owner, a short layout buffer. So the crate's tests are the oracle of the paths a future core may
use. The cores' oracles cover only the paths their own formats take.

## 6. What this crate does not do, and what else was seen

- It does not know a console. A magic, a version, a field list or a refusal of one core's format is the core's.
- It does not decide the in-frame shape of anything. It serializes; the machine's layout is the core's.
- **Candidates seen during the extraction, not extracted.** Each is a copy in two or three cores, and each is left for
  a separate decision:
  - the crash-log hook, `*_set_crash_log`, the same panic hook in all three `lib.rs` files;
  - the `*_interface_version` export and the C# loaders around it (`MoonNative.cs` and `MercuryNative.cs` differ only
    in their names and numbers; `MarsNative.cs` is a superset);
  - the ROM-patch tables (`*_machine_set_rom_patches`) in MercuryRT and MoonRT, the same shape;
  - the audio queue and its drain export (`*_machine_drain_audio`). *The queue was extracted on 2026-09-30 (§2.1);
    the exports are still each core's own.*
  - MoonRT's fault helper (`Fault`, `at`, `put`, `rem`). It is used by one core so far, but the C#-exception-as-status
    pattern applies to any core ported from a C# core that throws.
