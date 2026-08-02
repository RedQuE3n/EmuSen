# Venus (SNES) — OBC1

Covers `Cores/Nintendo/Venus - SNES/Coprocessors/Obc1/Obc1.cs`. The smallest enhancement chip this core implements, and the only one that isn't a processor.

> **Status: complete.** There is nothing left to build. One game uses this chip.

---

## 1. What the chip is

The Seta **OBC1** is an address generator. It sits between the S-CPU and the cartridge's SRAM and rewrites four addresses; that is the entire device. It has no registers of its own, no clock, no state, and nothing to step — every byte it reads or writes, *including its own control bytes*, lives in the cartridge's SRAM.

Only **Metal Combat: Falcon's Revenge** carries one. Detection is the header's cartridge type (`+$16`) with high nibble `$2` and a low nibble of 3 or more, i.e. `$25`.

Like the NEC DSPs and unlike the SA-1 or the GSU, it does not replace the memory map — it claims `$00-$3F/$80-$BF:6000-7FFF` and `CoprocessorOverlayMapper` leaves the rest of the cartridge on its ordinary LoROM decode. Because `Obc1` holds no state, it never reaches a save state at all; the SRAM it manipulates is already serialised by `Cartridge`.

If the header declares no SRAM the chip is not built, since it would have no memory to address.

---

## 2. What it actually does

The game keeps a mirror of OAM in SRAM and wants to update **one sprite at a time** through a fixed set of ports, without computing the sprite's address itself. The OBC1 does that computation.

Two control bytes select the target:

| Address | Meaning |
|---|---|
| `$1FF5` | bit 0 clear → table base `$1C00`; bit 0 set → base `$1800` |
| `$1FF6` | bits 0-6 → sprite index, 0-127 |

Note the **inversion** on `$1FF5`: base is `$1800 | ((~$1FF5 & 1) << 10)`, so the default (`$00`) selects `$1C00`.

Five ports then read and write relative to that:

| Port | Target |
|---|---|
| `$1FF0-$1FF3` | low table, `base + index × 4`, one byte each |
| `$1FF4` | high table, `base + (index >> 2) + $200` |

The low table is four bytes per sprite — X, Y, tile, attributes — so the index is scaled by 4. The high table is the SNES's usual packed OAM extension: **two bits per sprite, four sprites per byte**, so the index is scaled by ¼ and offset `$200` past the low table.

That packing is the one place the chip does more than arithmetic. A write to `$1FF4` must merge into the correct 2-bit field rather than overwrite the byte, and the field is chosen by the **low two bits of `$1FF6`**:

```
shift  = ($1FF6 & 3) * 2
result = (old & ~(3 << shift)) | ((value & 3) << shift)
```

A *read* of `$1FF4` returns the whole packed byte unmerged — the chip only splits on the way in.

Any other address in the window is plain SRAM, passed straight through. That includes `$1FF5` and `$1FF6` themselves: the game writes them as ordinary memory, and the chip reads them back out of SRAM every time it needs to compute an address.

---

## 3. Where to look when something is wrong

1. **Sprites appear at the wrong coordinates.** Check the `$1FF5` inversion first — getting it backwards puts the whole table 1KB away, which looks like garbage rather than like an off-by-one.
2. **Every fourth sprite's priority/high-X bit is wrong.** That is the `$1FF4` merge (§2) writing the wrong 2-bit field, or overwriting the byte instead of merging.
3. **Nothing happens at all.** The cartridge declared no SRAM, so no chip was built — see §1.
