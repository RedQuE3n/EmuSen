# Beryl_SN76489 — the SN76489 crate's record

`beryl-sn76489`, in `EmuSen/Cores/Sega/Beryl-HW/`, created 2026-10-05 at Nephrite's stage 5, step 1
(`Nephrite_Native.md` §18). The first Beryl crate that is not a CPU; the contract it keeps is `README.md`'s
"A chip that is not a processor".

## 1. Sources

- **SMS Power's "SN76489" document** (Maxim, with Charles MacDonald's samples of the Genesis, Game Gear and Master
  System noise), saved in the corpus as `docs/smspower/SN76489.html`: the latch and data bytes and their examples, the
  counters, the noise register's rates, feedback and reset, the 2 dB attenuation steps, the half-period of 0 or 1
  holding a tone high, the Game Gear's stereo register. Its short code fragments are a document's illustrations of
  the shift register's rule and of a parity function, not an emulator's source; the crate states the rule in its own
  words and code.
- **Plutiedev's "PSG" page** and **the MegaDrive Wiki's "PSG" and "SN76489" articles**: the Genesis's addresses
  (`$C00011` for the 68000, `$7F11` for the Z80), which are Nephrite's, not the crate's. The wiki says each 2 dB step
  "halves" the volume; it does not (2 dB is a factor of 0.794), and SMS Power's figure is used.
- **Texas Instruments' datasheet** (`docs/ti/SN76489.pdf`) is in the corpus; nothing in this step needed more than the
  SMS Power document gives.

## 2. What the crate does

- **Registers**: three 10-bit half-periods, four attenuations and the noise register, written through one port. A
  latch byte names the register and gives its low four bits; a data byte gives a tone's high six bits or the low bits
  of an attenuation or of the noise register. The latched register is never cleared, and a tone changes at each
  byte.
- **Counters**: on each internal clock (the input clock over 16) a counter counts down if not zero and, on reaching
  zero, reloads and flips its channel's bit. A tone with a half-period of 0 or 1 is held high and its counter rests,
  which is what sample playback relies on; the document says this may be a property of Sega's copy, and the crate
  gives it to both variants, there being no measurement of the discrete chip in the corpus.
- **Noise**: the noise counter reloads from `$10`, `$20` or `$40`, or from tone 2's register; the shift register moves
  right when the channel's bit goes from 0 to 1, its new top bit the parity of the tapped bits (white noise) or bit 0
  ("periodic"); writing the noise register resets it to its top bit alone. `Variant::SEGA` is 16 bits tapped at bits
  0 and 3 (the Master System's, the Game Gear's and the Genesis's); `Variant::DISCRETE` is 15 bits at bits 0 and 1.
- **Output**: each channel's bit at its attenuation, 8,192 at full volume falling 2 dB a step to 326, and silence at
  15; the noise channel's is the shift register's bit 0. The Game Gear's stereo register puts each channel on its
  sides; every other console leaves it at `$FF`. The output is unipolar, as the chip's is; the console's analogue path
  removes its offset.
- **`advance`** passes over clocks at which no counter reaches zero, and reports the output only when it changes.

## 3. Tests

Eight: the document's latch and data examples; a tone taking each byte as it comes; the half-period timing and the
held tone; the noise register's reset, "periodic" noise at one in sixteen and the shift every 32 clocks at rate 0;
white noise's feedback in both variants; the 2 dB steps; the Game Gear's stereo; and `advance`'s skipping checked
against clocking one tick at a time over 50,000 ticks, outputs and state alike.

## 4. Open

- The discrete chip's behaviour at half-period 0, and its power-on registers (the document reports a discrete chip
  starting with a tone), are not modelled; the crate starts as Sega's copy does, silent.
- The chip's analogue imperfections (the decay of a held level that the document describes) are not modelled; the
  Genesis's are its board's, which Nephrite's later steps take up.
