namespace EmuSen.Cores.Nintendo.Venus.Coprocessors.SuperFx
{
    // $3000-$32FF as the S-CPU sees it: the GSU's own register file, its
    // control registers, and the instruction cache - see Venus_SuperFX.md §3.
    // The GSU itself reaches all of this directly, not through here.
    public sealed partial class SuperFx
    {
        // GSU-2, the faster part Yoshi's Island uses.
        private const byte VersionCode = 0x04;

        // The cache is 512 bytes at $3100, so the index is a subtraction, not a
        // mask - $3100 & $1FF is $100, not 0.
        private const ushort CacheWindowBase = 0x3100;

        public byte ReadRegister(ushort offset)
        {
            if (offset >= 0x3100 && offset <= 0x32FF) return Cache[offset - CacheWindowBase];

            if (offset <= 0x301F)
            {
                int index = (offset & 0x1F) >> 1;
                return (offset & 1) == 0 ? (byte)R[index] : (byte)(R[index] >> 8);
            }

            switch (offset)
            {
                case 0x3030: return (byte)_sfr;
                case 0x3031:
                {
                    // Reading the high byte acknowledges the interrupt.
                    byte value = (byte)(_sfr >> 8);
                    SetFlag(FlagIrq, false);
                    return value;
                }
                case 0x3034: return _pbr;
                case 0x3036: return _rombr;
                case 0x3038: return _scbr;
                case 0x3039: return _clsr;
                case 0x303A: return _scmr;
                case 0x303B: return VersionCode;
                case 0x303C: return _rambr;
                case 0x303E: return (byte)_cbr;
                case 0x303F: return (byte)(_cbr >> 8);
            }

            return 0x00;
        }

        public void WriteRegister(ushort offset, byte data)
        {
            if (offset >= 0x3100 && offset <= 0x32FF)
            {
                int index = offset - CacheWindowBase;
                Cache[index] = data;
                // A line only counts as loaded once its last byte has been written.
                if ((index & 0x0F) == 0x0F) _cacheValid[index >> 4] = true;
                return;
            }

            if (offset <= 0x301F)
            {
                // The S-CPU may not disturb the GSU's registers mid-run.
                if (Running) return;

                int index = (offset & 0x1F) >> 1;
                if ((offset & 1) == 0) R[index] = (ushort)((R[index] & 0xFF00) | data);
                else R[index] = (ushort)((R[index] & 0x00FF) | (data << 8));

                // Writing R15's high byte is what launches the GSU - see Venus_SuperFX.md §3.3.
                if (offset == 0x301F) Start();
                return;
            }

            switch (offset)
            {
                case 0x3030:
                {
                    bool wasRunning = Running;
                    _sfr = (ushort)((_sfr & 0xFF00) | data);
                    // Clearing GO from outside stops the GSU where it stands.
                    if (wasRunning && !Running) _sreg = _dreg = 0;
                    return;
                }
                case 0x3031: _sfr = (ushort)((_sfr & 0x00FF) | (data << 8)); return;
                case 0x3033: _bramr = data; return;
                case 0x3034: _pbr = (byte)(data & 0x7F); InvalidateCache(); return;
                case 0x3037: _cfgr = data; return;
                case 0x3038: _scbr = data; return;
                case 0x3039: _clsr = data; return;
                case 0x303A: _scmr = data; return;
            }
        }
    }
}
