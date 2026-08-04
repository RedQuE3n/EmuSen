using EmuSen.Common;

namespace EmuSen.Cores.Nintendo.Mercury.Cpu.Core
{
    // The SM83: an 8080 descendant that is neither an 8080 nor a Z80 - see Mercury_Cpu.md §1.
    public sealed partial class Cpu
    {
        public const byte FlagZ = 0x80;
        public const byte FlagN = 0x40;
        public const byte FlagH = 0x20;
        public const byte FlagC = 0x10;

        // Where each interrupt hands control, indexed by the bit it occupies in IE/IF.
        private static readonly ushort[] InterruptVectors = { 0x0040, 0x0048, 0x0050, 0x0058, 0x0060 };

        [SkipInState] private readonly ICpuBus _bus;

        public byte A, B, C, D, E, H, L;

        // The low nibble is not wired up; it reads back as zero however it is written.
        public byte F;

        public ushort SP, PC;

        public bool Ime;
        public bool Halted;

        // EI takes effect after the instruction that follows it, not immediately.
        private bool _imeScheduled;

        // HALT with interrupts disabled and one already pending fails to advance PC once - see Mercury_Cpu.md §4.1.
        private bool _haltBug;

        // What HALT needs to know at execute time, captured before the fetch.
        private bool _interruptPending;

        public ushort LastInstructionPC { get; private set; }

        public long Cycles { get; private set; }

        public Cpu(ICpuBus bus) => _bus = bus;

        public ushort AF
        {
            get => (ushort)((A << 8) | F);
            set { A = (byte)(value >> 8); F = (byte)(value & 0xF0); }
        }

        public ushort BC
        {
            get => (ushort)((B << 8) | C);
            set { B = (byte)(value >> 8); C = (byte)value; }
        }

        public ushort DE
        {
            get => (ushort)((D << 8) | E);
            set { D = (byte)(value >> 8); E = (byte)value; }
        }

        public ushort HL
        {
            get => (ushort)((H << 8) | L);
            set { H = (byte)(value >> 8); L = (byte)value; }
        }

        // Post-boot-ROM state on a DMG, since Mercury starts with the cartridge - see Mercury_Cpu.md §5.
        public void Reset()
        {
            AF = 0x01B0;
            BC = 0x0013;
            DE = 0x00D8;
            HL = 0x014D;
            SP = 0xFFFE;
            PC = 0x0100;

            Ime = false;
            Halted = false;
            _imeScheduled = false;
            _haltBug = false;
            Cycles = 0;
        }

        // Returns the T-cycles this step consumed; the caller runs the rest of the machine by that much.
        public int Step(byte interruptEnable, byte interruptFlags, out int servicedBit)
        {
            servicedBit = -1;

            bool pending = (interruptEnable & interruptFlags & 0x1F) != 0;
            _interruptPending = pending;

            // A halted CPU wakes on a pending interrupt whether or not it is allowed to service one.
            if (Halted)
            {
                if (!pending) return Advance(4);
                Halted = false;
            }

            if (Ime && pending)
            {
                servicedBit = LowestSetBit((byte)(interruptEnable & interruptFlags));
                return Advance(ServiceInterrupt(servicedBit));
            }

            // Scheduled by EI on the previous instruction, so the one after it runs with IME set.
            if (_imeScheduled)
            {
                _imeScheduled = false;
                Ime = true;
            }

            LastInstructionPC = PC;

            byte opcode = Fetch();
            if (_haltBug)
            {
                PC--;
                _haltBug = false;
            }

            return Advance(Execute(opcode));
        }

        private int Advance(int cycles)
        {
            Cycles += cycles;
            return cycles;
        }

        private static int LowestSetBit(byte value)
        {
            for (int bit = 0; bit < 5; bit++)
            {
                if ((value & (1 << bit)) != 0) return bit;
            }
            return -1;
        }

        // Two wait cycles, the push, then the vector: 20 T-cycles with IME cleared behind it.
        private int ServiceInterrupt(int bit)
        {
            Ime = false;
            Push(PC);
            PC = InterruptVectors[bit];
            return 20;
        }

        private byte Fetch() => _bus.Read(PC++);

        private ushort Fetch16()
        {
            byte low = Fetch();
            byte high = Fetch();
            return (ushort)(low | (high << 8));
        }

        private void Push(ushort value)
        {
            _bus.Write(--SP, (byte)(value >> 8));
            _bus.Write(--SP, (byte)value);
        }

        private ushort Pop()
        {
            byte low = _bus.Read(SP++);
            byte high = _bus.Read(SP++);
            return (ushort)(low | (high << 8));
        }

        private bool Flag(byte mask) => (F & mask) != 0;

        private void SetFlag(byte mask, bool on)
        {
            if (on) F |= mask;
            else F &= (byte)~mask;
        }
    }
}
