# Mercury — cheat codes

*Written 2026-08-08 with Phase B, because registering the core in `CoreCatalog`
means claiming two libretro cheat folders and a code format to read them with.*

---

## 1. Two formats, two mechanisms

The Game Boy had both of the devices the NES and SNES had, and they work the two
different ways this project already models:

| | Game Genie | GameShark |
|---|---|---|
| Mechanism | ROM read intercept | RAM poke, once per frame |
| Length | 6 or 9 hex digits | 8 hex digits |
| Compare byte | on the 9-digit form | never |
| Can cheat | anything in ROM, including code | anything in the CPU's address space |

The distinction is not a detail of implementation. A Game Genie code changes what
the *cartridge says*, so it can patch an instruction and take effect before the
game ever runs the affected code. A GameShark code changes what memory *holds*,
re-applied every frame, so it fights the game rather than rewriting it — which is
why GameShark codes for a value the game recomputes every frame appear to do
nothing.

Length alone separates them, which is why Mercury deliberately does **not** offer
a plain `address:value` form the way Moon does: it would be six hex digits and
would collide with the short Game Genie code. GameShark already is the raw form,
with a bank byte in front.

## 2. The Game Genie cipher

Nine characters, conventionally written `ABC-DEF-GHI`. Taking the nine hex digits
as `d0`…`d8`:

```
value   = (d0 << 4) | d1
address = ((d5 ^ $F) << 12) | (d2 << 8) | (d3 << 4) | d4
compare = ror8((d6 << 4) | d8, 2) ^ $BA
```

Three things in that are worth naming, because each is a place a plausible-looking
implementation goes wrong:

- **The address's top nibble is last and inverted.** `d5` is not part of the low
  three nibbles it sits next to in the code; it is the high nibble, XOR'd with
  `$F`. This is the whole of the obfuscation — the rest is a straight read.
- **`d7` is unused.** It is a check digit the device itself validated. Mercury
  ignores it rather than rejecting codes that fail it, because a code that
  decodes to the right patch with a mistyped check digit is still the right patch.
- **The compare is rotated before it is XOR'd,** right by two, as a byte. Getting
  the direction or the width wrong produces a compare that fails on every read, so
  the cheat silently does nothing rather than visibly misbehaving — which makes it
  the hardest of the three to notice.

A six-digit code is the same decode with no compare. `DecodeCompare` returns null
for it, which the shared `CheatRegistry` already understands as "patch every bank".

### 2.1 Why the compare matters

Same reason as on the NES (`Moon_Cheats.md`): a Game Boy CPU address in the
`$4000-$7FFF` window is whatever bank the mapper currently selects, so a code
without a compare patches that address in *every* bank. The compare byte is what
pins the patch to the one bank whose byte matches. Dropping it does not produce a
weaker cheat, it produces a wrong one.

## 3. The GameShark form

Eight hex digits, `TTVVAAAA`:

- `TT` — the external RAM bank the code applies to. Mercury reads it (`BankOf`)
  but does not act on it: the poke goes through `CPUBUS`, so it lands in whatever
  bank is mapped when the frame boundary comes round. For the overwhelming
  majority of codes, which target work RAM rather than cartridge RAM, the bank
  byte is `$01` and means nothing.
- `VV` — the value to write.
- `AAAA` — **the address, little-endian.** `011A56C1` is address `$C156`, not
  `$56C1`. This is the single most common transcription error with these codes.

## 4. What is not supported

- **No multi-line or conditional GameShark codes.** The Game Boy device had no
  such thing; the format is one poke per code.
- **The bank byte is not enforced**, as above. A code that only makes sense for
  one cartridge-RAM bank will be applied whichever bank is live. No code in
  circulation is known to depend on this, and enforcing it would require the
  registry to carry a per-code bank the interface has no field for.
- **No Game Genie code validation beyond length.** A code with the wrong check
  digit decodes anyway (§2).
