using System;
using System.IO;
using EmuSen.Common;
using EmuSen.Cores.Nintendo.Mars.Rom;

namespace EmuSen.Cores.Nintendo.Mars.Memory
{
    // The one save chip a cartridge carries, named before boot or by the first thing the game does with it - see Mars_Save.md §1.
    public sealed class SaveChip
    {
        public Eeprom? Eeprom { get; private set; }
        public Sram? Sram { get; private set; }
        public FlashRam? Flash { get; private set; }

        public N64SaveType Type { get; private set; }

        // Held until the chip is known, so a save from an earlier run reaches whichever device it turns out to be - see §1.
        private readonly byte[]? _saved;

        public SaveChip(N64SaveType type, byte[]? saved = null)
        {
            _saved = saved;
            Type = N64SaveType.Unknown;
            if (type != N64SaveType.Unknown) Become(type);
        }

        public bool Dirty => Eeprom?.Dirty ?? Sram?.Dirty ?? Flash?.Dirty ?? false;

        public byte[]? Contents => Eeprom?.Data ?? Sram?.Data ?? Flash?.Data;

        public void Saved()
        {
            if (Eeprom != null) Eeprom.Dirty = false;
            if (Sram != null) Sram.Dirty = false;
            if (Flash != null) Flash.Dirty = false;
        }

        // The chip's type and then its device, rebuilt first because reflection only fills what exists - see Mars_SaveStates.md §3.
        public void WriteState(BinaryWriter w)
        {
            w.Write((int)Type);
            if (Device is { } device) StateSerializer.Write(w, device);
        }

        // A loaded chip is marked changed, so the file on disk catches up with it at the next write - see Mars_SaveStates.md §3.
        public void ReadState(BinaryReader r)
        {
            var type = (N64SaveType)r.ReadInt32();

            Eeprom = null;
            Sram = null;
            Flash = null;
            Type = N64SaveType.Unknown;
            if (type != N64SaveType.Unknown) Become(type);

            if (Device is not { } device) return;
            StateSerializer.Read(r, device);

            if (Eeprom != null) Eeprom.Dirty = true;
            if (Sram != null) Sram.Dirty = true;
            if (Flash != null) Flash.Dirty = true;
        }

        private object? Device => (object?)Eeprom ?? (object?)Sram ?? Flash;

        // What an earlier run's save says the chip was, by its length alone - see §1.
        public static N64SaveType FromSaveLength(int length) => length switch
        {
            0x200 or Eeprom.Size => N64SaveType.Eeprom4k,
            Sram.BankSize => N64SaveType.Sram256k,
            3 * Sram.BankSize => N64SaveType.SramBanked768k,
            FlashRam.Size => N64SaveType.FlashRam,
            _ => N64SaveType.Unknown,
        };

        // The fifth channel: an EEPROM answers, an undecided chip answers as the smaller one, anything else is silent - see §2.
        public bool AnswerJoybus(byte[] ram, int command, int send, int receive, out int wrote)
        {
            wrote = 0;
            if (send == 0) return false;

            if (Type == N64SaveType.Unknown && ram[command] is Eeprom.Read or Eeprom.Write) Become(N64SaveType.Eeprom4k);
            if (Eeprom != null) return Eeprom.Answer(ram, command, send, receive, out wrote);

            // Asking what is there does not decide it, so an undecided chip describes the smaller EEPROM - see §2.
            if (Type != N64SaveType.Unknown || ram[command] is not (Eeprom.Info or Eeprom.Reset)) return false;

            wrote = 3;
            Joybus.Reply(ram, command + send, receive, 0x00, Eeprom.Kind4Kbit, 0x00);
            return true;
        }

        // The processor on the second domain: a first touch here names FlashRAM - see §1.
        public uint Read32(uint offset)
        {
            if (Type == N64SaveType.Unknown) Become(N64SaveType.FlashRam);

            if (Flash != null) return Flash.Read32(offset);
            if (Sram != null) return Word(offset);

            return OpenBus(offset);
        }

        public void Write32(uint offset, uint value)
        {
            if (Type == N64SaveType.Unknown) Become(N64SaveType.FlashRam);

            if (Flash != null) Flash.Write32(offset, value);
            else if (Sram != null) for (int i = 0; i < 4; i++) Sram.Write8(offset + (uint)i, (byte)(value >> (24 - 8 * i)));
        }

        // A transfer on the second domain: a first touch here names SRAM - see §1.
        public byte DmaRead8(uint offset)
        {
            if (Type == N64SaveType.Unknown) Become(N64SaveType.Sram256k);

            if (Sram != null) return Sram.Read8(offset);
            return Flash?.DmaRead8(offset) ?? 0;
        }

        public void DmaWrite8(uint offset, byte value)
        {
            if (Type == N64SaveType.Unknown) Become(N64SaveType.Sram256k);

            if (Sram != null) Sram.Write8(offset, value);
            else Flash?.DmaWrite8(offset, value);
        }

        // Nothing answers on an empty second domain, so the bus keeps the low half of the address twice - see §3.
        public static uint OpenBus(uint offset) => (offset & 0xFFFF) * 0x0001_0001u;

        private uint Word(uint offset) =>
            (uint)((Sram!.Read8(offset) << 24) | (Sram.Read8(offset + 1) << 16) | (Sram.Read8(offset + 2) << 8) | Sram.Read8(offset + 3));

        private void Become(N64SaveType type)
        {
            Type = type;

            switch (type)
            {
                case N64SaveType.Eeprom4k:
                case N64SaveType.Eeprom16k:
                    Eeprom = new Eeprom(type == N64SaveType.Eeprom16k, _saved);
                    break;

                case N64SaveType.Sram256k:
                    Sram = new Sram(1, _saved);
                    break;

                case N64SaveType.SramBanked768k:
                    Sram = new Sram(3, _saved);
                    break;

                case N64SaveType.Sram1M:
                    Sram = new Sram(4, _saved);
                    break;

                case N64SaveType.FlashRam:
                    Flash = new FlashRam(_saved);
                    break;
            }
        }
    }
}
