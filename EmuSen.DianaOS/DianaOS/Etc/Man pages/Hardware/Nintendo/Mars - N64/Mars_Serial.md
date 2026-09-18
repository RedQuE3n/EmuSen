# Mars — the serial interface, and the block the PIF runs for it

*Landed 2026-09-17. Phase E's fifth slice, and the first thing in Mars a player could touch. The code is
`Memory/SiInterface.cs`, `Memory/Joybus.cs` and `Memory/Controller.cs`, plus one predicate widened in
`Memory/MemoryBus.cs`; the grading is `MarsSerialTests`, the hardware corpus, and the second half of a
prediction two slices old.*

---

## 0. What grades this

**Three different things, and for once one of them is the corpus.**

- **The corpus grades PIF RAM's stores exactly.** `Mars_Corpus.md` §3's census attributed six of its failing
  groups to PIF RAM, and §1 is what they wanted: the count went from 152 failing assertions to **146**, which
  is six, which is all of them and nothing else. The number was predicted before the change and checked
  after.
- **The prediction of `Mars_Microcode.md` §3 is now complete.** It said that when Phase E built *both*
  devices its arbitrary interrupt pulses should be deleted and the test should still pass.
  `Mars_VideoTiming.md` §3 deleted the video one; this slice deletes the serial one. **The microcode test now
  runs with no stand-ins at all**, and Wave Race still hands its first display list to the display processor
  and still clears its depth buffer.
- **Eighteen unit tests grade the protocol**, because nothing else can: the corpus does not exercise the
  joybus, and angrylion has no PIF at all — it is a graphics plugin. §5 is where the protocol came from
  instead.

## 1. PIF RAM latches whole words

**A store from the processor to PIF RAM writes the whole thirty-two-bit word, whatever size the instruction
names** — the rule `Mars_Memory.md` §2.4 already established for the signal processor's memories, with the
register shifted so the named bytes land in their lanes and everything below them zeroed. `SB` of `0x12345678`
at offset 0 writes `0x78000000`; at offset 3 it writes `0x12345678` entire.

**One predicate now covers both windows**, rather than the rule being written twice. That is the whole of the
change, and it is worth noticing how little it was: the mechanism had been built for a different device two
slices earlier, the corpus had been naming the second device since before that, and nobody had connected
them. The census is what connected them, which is the argument for keeping one.

**Loads are unaffected**, and main memory keeps byte-precise stores. `Mars_Memory.md` §2.4 warns that the
easy way to break this rule is to apply it everywhere, and the test that pins main memory still passes.

## 2. The interface

**Sixty-four bytes each way between RDRAM and PIF RAM, and nothing else.** Writing the read register carries
PIF RAM out to the address in the DRAM register; writing the write register carries memory in, and then the
PIF runs whatever block arrived (§3). Either way the serial interrupt is raised when it is done.

**Only a write to the status register clears that interrupt**, as with the peripheral interface, and the
status register reports nothing else: no transfer is ever busy, because every transfer has already finished
by the time the instruction that started it retires.

**The block runs on the way in only.** A game writes its command block to memory, DMAs it to PIF RAM — which
is what makes the PIF read it — and then DMAs PIF RAM back to see the replies. Running the block on the way
out as well would answer questions nobody asked.

## 3. The joybus

**The PIF walks the block from its first byte, keeping count of which channel it is on.** Each entry is a
send length, a receive length, the command and its arguments, and room for the reply. Three bytes are not
entries at all:

| byte | meaning |
| --- | --- |
| `0xFE` | the block ends here |
| `0x00` | this channel has nothing to ask; move to the next one |
| `0xFD`, `0xFF` | padding; skip it and stay on this channel |

**Both lengths are six bits.** The top two bits of a length byte are not part of it — which matters, because
the PIF writes flags into them on the way out (§3.3).

**The channel advances after a command as well as on a zero.** So a block asking all four controllers for
their state is four entries with nothing between them, and a block that asks only the third is two zeros and
an entry.

**The PIF clears the byte that started it.** The last byte of PIF RAM carries the request; the PIF writes
zero over it when the block is done, so a block runs once however many times it is read back.

### 3.1 The controller

**Four ports, each either holding a controller or not.** A port holds its buttons as sixteen bits and its
stick as two signed bytes:

- **The high byte** is A, B, Z, Start, then up, down, left, right — in that order, from bit 7 down.
- **The low byte** is two zero bits, then L, R, and C-up, C-down, C-left, C-right.

**The first port holds a controller and the other three do not, until something says otherwise.** That is a
decision rather than a measurement: a console with nothing plugged in is the more neutral default, but every
game this slice is tested against needs a controller in the first port, and a test that has to plug one in
before it can ask anything is a test about the harness. `ICore.SetButton` will set these when Phase F wires
the frontend; nothing does yet.

### 3.2 What a controller answers

**Two commands, and a silence for everything else.**

- **Info** (`0x00`, or `0xFF` which means the same thing here) replies with three bytes: `05 00`, and then
  `02` for a controller with an empty slot or `01` for one with a Controller Pak. Mars models the pak's
  presence as a flag and nothing more.
- **State** (`0x01`) replies with the two button bytes and the two stick bytes.
- **The Controller Pak's own commands** (`0x02` read, `0x03` write) are **not answered**, which reads to a
  game as a controller that has no pak fitted rather than as an error. The pak is a save device and belongs
  with the others.

### 3.3 What the length byte says afterwards

**The PIF rewrites the receive length byte with two flags above it:**

- **bit 7** when nothing answered — an empty port, or a command the device does not know.
- **bit 6** when the reply was longer than the room left for it. The reply is cut rather than overflowing:
  an info command with room for two bytes gets `05 00` and the over-run flag.

**A shorter reply leaves the rest of the room alone**, which is how a game distinguishes "answered briefly"
from "not answered".

## 4. What the tests say

**Eighteen tests, no reference, and the first thirteen passed on their first run.** They cover the transfer in
both directions and its interrupt, the status register's clear, that a block runs only when its last byte asks
and only once, both commands, the pak flag, an empty port, an over-run, an unanswered command, the two ways
the channel advances and the one way it does not, both lengths being six bits, and the two positions the end
byte can stop the walk from.

**Twenty-six breakages, every one caught — but only after the round found that six of the first thirteen tests
were watching the wrong thing.** That is the highest survivor rate of any slice in Phase E, and all six were
the tests' fault rather than dead code:

- **Two counted channels the block could not distinguish.** The test for a zero length moving to the next
  channel put a controller in the third port and left one in the first, so the reply was the same whichever
  channel answered. The test for a command advancing the channel had only one command in its block. Both now
  arrange a port that answers differently from the one before it.
- **Two used lengths with nothing above six bits**, so masking them off decided nothing. One block now sets
  bit 6 in each length.
- **Two are the end byte**, and they are the interesting pair, because Mars has a guard the hardware does not
  and it very nearly hides them both. Mars refuses to walk past the end of the block; the referee's PIF has
  a 64-byte RAM and indices that wrap. The end byte's own low six bits are 62, so read as a length it asks
  for more room than the block can ever hold — and **Mars's bounds guard therefore stops the walk wherever
  the end byte's check would have**, in every position but one. The exceptions are the narrow cases the two
  new tests use: an end byte in the very first position, where a two-byte entry still fits, and an end byte
  in the receive position after a *zero-length* send. Without those two blocks the rules are untestable
  through Mars, and the round is what said so.

**The lesson is about the guard rather than the tests.** A defensive bound that a reference does not have can
make a real rule unobservable, and it does it quietly: the code is right, the test passes, and the rule is
held by the guard instead of by itself. This is the second time in Phase E that has happened — the other
was `Mars_VideoFilter.md` §4.2's byte mask, which the bracketing made dead — and the difference is that this
time the rule is real and it is the *evidence* that was missing.

**What no test here establishes is timing.**

Every transfer completes inside the instruction that starts it,
and a real controller read takes long enough that a game notices. `Mars_Memory.md` §7.1 made the same
decision for the peripheral interface and for the same reason: a transfer that takes no time is wrong in a way
that is easy to see and easy to change, and a transfer that takes the wrong amount of time is wrong in a way
that is neither.

## 5. The referee, which is the source here

**angrylion has no PIF, and parallel-rdp has no PIF; this slice has no software reference at all.** The
protocol above is read from the N64_MiSTer core's `rtl/PIF.vhd` and `rtl/Gamepad.vhd`, which implement it for
real hardware, and every rule in §3 is one of theirs:

- **The block walk** is `PIF.vhd` §774–801: stop at index 63 or `0xFE`; `0x00` advances the channel; `0xFD`
  and `0xFF` advance the index only; anything else is a send length, and its low six bits are taken
  (`EXT_send <= unsigned(ram_q_b(5 downto 0))`, §793). §1174 advances the channel after a command completes.
- **The length byte's flags** are §1151, in one line:
  `ram_data_b <= (not EXT_valid) & EXT_over & std_logic_vector(EXT_receive)` — not-valid above over-run above
  the six-bit length.
- **Clearing the byte that started it** is §777–781, guarded by not being in read mode, which is why §2 runs
  the block on the way in only.
- **The controller's replies** are `Gamepad.vhd` §348–398: `05`, `00`, and then `01` for a pak or `02`
  without one; and §478–485 with §529–535 for the two button bytes, bit for bit in the order §3.1 lists.
- **Which commands a controller knows** is `Gamepad.vhd` §321–338: info, state, the two pak commands, and a
  keyboard command; anything else times out, which is the silence of §3.2.

**Where the referee does more than Mars**, it is marked in §6 rather than guessed at: the pak commands, the
EEPROM and real-time clock on the channels above the fourth, and the CIC challenge.

**What none of this establishes is the console.** The FPGA is one implementation, and being the only one
available makes its agreement unfalsifiable rather than strong. Every rule here should be read as *"this is
what a synthesisable implementation of the PIF does"*, not as a measurement — which is the same caveat
`Mars_RdpReferee.md` §0 opens with, arrived at from the opposite direction.

## 6. What is not here

- **The Controller Pak**, the EEPROM and the real-time clock — the channels above the fourth and the two pak
  commands. These are save devices and belong with the peripheral interface's.
- **The CIC challenge**, the boot handshake the PIF answers with its own six-byte reply. Mars boots through
  `Mars_Boot.md`'s handoff rather than through the PIF, so nothing asks.
- **Timing** (§4).
- **Anything a player can reach.** `ICore.SetButton` is unwired, so the buttons in §3.1 are only ever the
  zeroes a test leaves there.
