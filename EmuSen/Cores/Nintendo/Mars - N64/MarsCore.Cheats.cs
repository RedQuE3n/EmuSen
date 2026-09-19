using System;
using EmuSen.Cores.Nintendo.Mars.Debug;
using EmuSen.DianaOS.DianaOS.Var;

namespace EmuSen.Cores.Nintendo.Mars
{
    // The registry a frontend hands over, applied at each frame's end and consulted by the cartridge - see Mars_Cheats.md §5.
    public sealed partial class MarsCore : global::EmuSen.Cores.ICheatRegistryHost
    {
        // Main memory's name here, in the codec, and in the debug target - see Mars_Cheats.md §3.
        public const string SpaceRdram = MarsDebugSpaces.Rdram;

        private CheatRegistry _cheats = new();

        // Settable so the registry a frontend already fills becomes this core's own - see EmuSen_Cheats.md §6.
        public CheatRegistry Cheats
        {
            get => _cheats;
            set
            {
                _cheats = value;
                if (Bus is not null) Bus.RomPatcher = new global::EmuSen.Cores.CheatRomPatcher(value);
            }
        }

        // Public so a paused frontend need not wait for a frame boundary - see EmuSen_Cheats.md §6.
        public void ApplyCheats()
        {
            if (Bus is not null) Cheats.ApplyAll(ReadForCheat, WriteForCheat);
        }

        // Held while interrupts are off, as Project64 holds it, or a cheat lands in the boot code's checksum - see Mars_Cheats.md §5.1.
        private void ApplyCheatsAtFrameEnd()
        {
            if (Cpu is { } cpu && (cpu.Cop0[global::EmuSen.Cores.Nintendo.Mars.Cpu.Core.Cpu.StatusRegister] & 1) != 0) ApplyCheats();
        }

        // Past the end reads zero, as the bus reads RDRAM that is not installed - see Mars_Cheats.md §3.1.
        private byte ReadForCheat(string spaceName, int address)
        {
            if (CheatSpace(spaceName) is not { } bytes || (uint)address >= (uint)bytes.Length) return 0;

            if (bytes == Bus!.Rdram) Bus.Dp.WaitFor((uint)address, 9);
            return bytes[address];
        }

        // Past the end is dropped, as a store to RDRAM that is not installed is - see Mars_Cheats.md §3.1.
        private void WriteForCheat(string spaceName, int address, byte value)
        {
            if (CheatSpace(spaceName) is { } bytes && (uint)address < (uint)bytes.Length)
            {
                if (bytes == Bus!.Rdram) Bus.Dp.WaitFor((uint)address, 9);
                bytes[address] = value;
                Bus!.Written++;
            }
        }

        // The writable spaces the debug target names, byte for byte as the bus stores them.
        private byte[]? CheatSpace(string spaceName)
        {
            if (string.Equals(spaceName, SpaceRdram, StringComparison.OrdinalIgnoreCase)) return Bus?.Rdram;
            if (string.Equals(spaceName, MarsDebugSpaces.Dmem, StringComparison.OrdinalIgnoreCase)) return Bus?.SpDmem;
            if (string.Equals(spaceName, MarsDebugSpaces.Imem, StringComparison.OrdinalIgnoreCase)) return Bus?.SpImem;
            if (string.Equals(spaceName, MarsDebugSpaces.PifRam, StringComparison.OrdinalIgnoreCase)) return Bus?.PifRam;
            return null;
        }
    }
}
