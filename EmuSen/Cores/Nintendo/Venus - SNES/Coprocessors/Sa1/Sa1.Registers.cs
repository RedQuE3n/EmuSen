namespace EmuSen.Cores.Nintendo.Venus.Coprocessors.Sa1
{
    // The $2200-$23FF register file. Both CPUs see the whole thing at the
    // same addresses; which side is expected to drive which register is a
    // software convention, not a decode - see Venus_SA1.md §4.
    public sealed partial class Sa1
    {
        public byte ReadRegister(ushort offset)
        {
            switch (offset)
            {
                // SFR - what the S-CPU sees of the SA-1.
                case 0x2300:
                    return (byte)((_sa1IrqToScpu ? 0x80 : 0)
                                | ((_scnt & 0x10) != 0 ? 0x40 : 0)
                                | (_dmaIrqToScpu ? 0x20 : 0)
                                | ((_scnt & 0x20) != 0 ? 0x10 : 0)
                                | (_scnt & 0x0F));

                // CFR - what the SA-1 sees of the S-CPU.
                case 0x2301:
                    return (byte)((_scpuIrqToSa1 ? 0x80 : 0)
                                | (_scpuNmiToSa1 ? 0x40 : 0)
                                | (_timerIrqToSa1 ? 0x20 : 0)
                                | (_dmaIrqToSa1 ? 0x10 : 0)
                                | (_ccnt & 0x0F));

                case 0x2302: LatchTimerRead(); return (byte)_hcrLatch;
                case 0x2303: return (byte)(_hcrLatch >> 8);
                case 0x2304: return (byte)_vcrLatch;
                case 0x2305: return (byte)(_vcrLatch >> 8);

                case 0x2306: return (byte)Math.Result;
                case 0x2307: return (byte)(Math.Result >> 8);
                case 0x2308: return (byte)(Math.Result >> 16);
                case 0x2309: return (byte)(Math.Result >> 24);
                case 0x230A: return (byte)(Math.Result >> 32);
                case 0x230B: return Math.Overflow ? (byte)0x80 : (byte)0x00;

                case 0x230C: return (byte)BitStream.Data;
                case 0x230D:
                    byte high = (byte)(BitStream.Data >> 8);
                    BitStream.OnDataHighRead();
                    return high;

                // VC - version code. Every documented SA-1 reports 1.
                case 0x230E: return 0x01;
            }

            return 0x00;
        }

        public void WriteRegister(ushort offset, byte data)
        {
            switch (offset)
            {
                case 0x2200:
                {
                    byte previous = _ccnt;
                    _ccnt = data;
                    OnControlWrite(previous, data);
                    return;
                }

                case 0x2201: _sie = data; return;
                case 0x2202:
                    if ((data & 0x80) != 0) _sa1IrqToScpu = false;
                    if ((data & 0x20) != 0) _dmaIrqToScpu = false;
                    return;

                case 0x2203: _crv = (ushort)((_crv & 0xFF00) | data); return;
                case 0x2204: _crv = (ushort)((_crv & 0x00FF) | (data << 8)); return;
                case 0x2205: _cnv = (ushort)((_cnv & 0xFF00) | data); return;
                case 0x2206: _cnv = (ushort)((_cnv & 0x00FF) | (data << 8)); return;
                case 0x2207: _civ = (ushort)((_civ & 0xFF00) | data); return;
                case 0x2208: _civ = (ushort)((_civ & 0x00FF) | (data << 8)); return;

                case 0x2209:
                    _scnt = data;
                    if ((data & 0x80) != 0) _sa1IrqToScpu = true;
                    return;

                case 0x220A: _cie = data; return;
                case 0x220B:
                    if ((data & 0x80) != 0) { _scpuIrqToSa1 = false; _sa1IrqTaken = false; }
                    if ((data & 0x40) != 0) { _timerIrqToSa1 = false; _sa1IrqTaken = false; }
                    if ((data & 0x20) != 0) { _dmaIrqToSa1 = false; _sa1IrqTaken = false; }
                    if ((data & 0x10) != 0) { _scpuNmiToSa1 = false; _sa1NmiTaken = false; }
                    return;

                case 0x220C: _snv = (ushort)((_snv & 0xFF00) | data); return;
                case 0x220D: _snv = (ushort)((_snv & 0x00FF) | (data << 8)); return;
                case 0x220E: _siv = (ushort)((_siv & 0xFF00) | data); return;
                case 0x220F: _siv = (ushort)((_siv & 0x00FF) | (data << 8)); return;

                case 0x2210: _tmc = data; return;
                case 0x2211: _hCounter = 0; _vCounter = 0; _linearCounter = 0; return;
                case 0x2212: _hCompare = (ushort)((_hCompare & 0xFF00) | data); return;
                case 0x2213: _hCompare = (ushort)((_hCompare & 0x00FF) | (data << 8)); return;
                case 0x2214: _vCompare = (ushort)((_vCompare & 0xFF00) | data); return;
                case 0x2215: _vCompare = (ushort)((_vCompare & 0x00FF) | (data << 8)); return;

                case 0x2220: _cxb = data; return;
                case 0x2221: _dxb = data; return;
                case 0x2222: _exb = data; return;
                case 0x2223: _fxb = data; return;
                case 0x2224: _bmaps = data; return;
                case 0x2225: _bmap = data; return;
                case 0x2226: _sbwe = data; return;
                case 0x2227: _cbwe = data; return;
                case 0x2228: _bwpa = data; return;
                case 0x2229: _siwp = data; return;
                case 0x222A: _ciwp = data; return;

                case 0x2230: Dma.WriteControl(data); return;
                case 0x2231: Dma.WriteCharacterParams(data); return;
                case 0x2232: Dma.WriteSourceByte(0, data); return;
                case 0x2233: Dma.WriteSourceByte(1, data); return;
                case 0x2234: Dma.WriteSourceByte(2, data); return;
                case 0x2235: Dma.WriteDestByte(0, data); return;
                case 0x2236: Dma.WriteDestByte(1, data); return;
                case 0x2237: Dma.WriteDestByte(2, data); return;
                case 0x2238: Dma.WriteCounterByte(0, data); return;
                case 0x2239: Dma.WriteCounterByte(1, data); return;
                case 0x223F: _bbf = data; return;

                case 0x2250: Math.WriteControl(data); return;
                case 0x2251: Math.WriteALow(data); return;
                case 0x2252: Math.WriteAHigh(data); return;
                case 0x2253: Math.WriteBLow(data); return;
                case 0x2254: Math.WriteBHighTrigger(data); return;

                case 0x2258: BitStream.WriteControl(data); return;
                case 0x2259: BitStream.WriteAddressByte(0, data); return;
                case 0x225A: BitStream.WriteAddressByte(1, data); return;
                case 0x225B: BitStream.WriteAddressByte(2, data); return;
            }

            // BRF - the 16-byte staging buffer type-2 character conversion reads from.
            if (offset >= 0x2240 && offset <= 0x224F) Dma.WriteBrf(offset - 0x2240, data);
        }

        // Raised by Sa1Dma when a character-conversion transfer completes.
        internal void RaiseDmaIrq()
        {
            _dmaIrqToScpu = true;
            _dmaIrqToSa1 = true;
        }
    }
}
