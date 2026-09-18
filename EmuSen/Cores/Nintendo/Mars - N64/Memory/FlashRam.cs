using System;

namespace EmuSen.Cores.Nintendo.Mars.Memory
{
    // 128KB of flash behind a command register, which does its work when told to execute - see Mars_Save.md §4.
    public sealed class FlashRam
    {
        public const int Size = 0x2_0000;
        public const int PageSize = 128;

        // What the status word reads in each mode: 0x1111_8001 and a chip ID - see §4.
        public const ulong StatusIdentify = 0x1111_8001_00C2_001DUL;
        public const ulong StatusErase = 0x1111_8008_00C2_001DUL;
        public const ulong StatusProgram = 0x1111_8004_00C2_001DUL;
        public const ulong StatusRead = 0x1111_8004_F000_001DUL;

        private const byte SetErasePage = 0x4B, Erase = 0x78, SetProgramPage = 0xA5, Program = 0xB4,
            Execute = 0xD2, Identify = 0xE1, ReadArray = 0xF0;

        private enum Mode { Idle, Read, Status, Erase, Program }

        public readonly byte[] Data = new byte[Size];

        public bool Dirty;

        private readonly byte[] _page = new byte[PageSize];

        private Mode _mode;
        private ulong _status;
        private uint _pageNumber;

        public FlashRam(byte[]? saved)
        {
            Data.AsSpan().Fill(0xFF);
            if (saved != null) saved.AsSpan(0, Math.Min(saved.Length, Size)).CopyTo(Data);
        }

        // The processor reads the status word's halves, whatever the mode - see §4.
        public uint Read32(uint offset) => (offset & 4) == 0 ? (uint)(_status >> 32) : (uint)_status;

        // A write anywhere but the domain's first word is a command - see §4.
        public void Write32(uint offset, uint value)
        {
            if (offset == 0) return;

            switch ((byte)(value >> 24))
            {
                case SetErasePage:
                    _pageNumber = value & 0x3FF;
                    break;

                case Erase:
                    _mode = Mode.Erase;
                    _status = StatusErase;
                    break;

                case SetProgramPage:
                    _pageNumber = value & 0x3FF;
                    _status = StatusProgram;
                    break;

                case Program:
                    _mode = Mode.Program;
                    break;

                case Execute:
                    Run();
                    break;

                case Identify:
                    _mode = Mode.Status;
                    _status = StatusIdentify;
                    break;

                case ReadArray:
                    _mode = Mode.Read;
                    _status = StatusRead;
                    break;
            }
        }

        // A transfer out reads the status word in status mode, the array in read mode, and zeros otherwise - see §4.
        public byte DmaRead8(uint offset) => _mode switch
        {
            Mode.Status => (byte)(_status >> (8 * (7 - (int)(offset & 7)))),
            Mode.Read => Data[offset & (Size - 1)],
            _ => 0,
        };

        // A transfer in fills the page buffer, whatever the mode, wrapping every page - see §4.
        public void DmaWrite8(uint offset, byte value) => _page[offset & (PageSize - 1)] = value;

        // One page, erased or programmed from the buffer; the mode stays as it was - see §4.
        private void Run()
        {
            var target = Data.AsSpan((int)_pageNumber * PageSize, PageSize);

            if (_mode == Mode.Erase) target.Fill(0xFF);
            else if (_mode == Mode.Program) _page.CopyTo(target);
            else return;

            Dirty = true;
        }
    }
}
