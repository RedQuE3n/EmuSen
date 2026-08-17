using System;
using EmuSen.Debug;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.Cores.Nintendo.Venus.Memory
{
    public class DmaChannel
    {
        public byte Control;
        public byte DestinationReg;
        public ushort SourceAddress;
        public byte SourceBank;
        public ushort TransferSize;
        public byte IndirectBank;
        public ushort TableAddress;
        public byte LineCounter;
        
        public bool HdmaActive;
        public bool HdmaDoTransfer;
        public ushort IndirectAddress;
    }

    public class Dma
    {
        private DmaChannel[] _channels;
        [EmuSen.Common.SkipInState] private MemoryBus _bus;

        // Null until `dma log on` arms one, which is the whole cost on a normal run - see `man dma`.
        [EmuSen.Common.SkipInState] public DmaLogRegistry? DmaLog;

        // The live channel table, in the shape a core-agnostic command prints - see `man dma`.
        public IReadOnlyList<DebugDmaChannel> DebugChannels()
        {
            var result = new List<DebugDmaChannel>(_channels.Length);
            for (int i = 0; i < _channels.Length; i++)
            {
                DmaChannel ch = _channels[i];
                result.Add(new DebugDmaChannel(i,
                    generalEnabled: false,
                    hdmaEnabled: (HdmaEnable & (1 << i)) != 0,
                    ch.Control, ch.DestinationReg,
                    (ch.SourceBank << 16) | ch.SourceAddress,
                    ch.TransferSize == 0 ? 0x10000 : ch.TransferSize,
                    ch.HdmaActive, (ch.SourceBank << 16) | ch.TableAddress, ch.LineCounter,
                    (ch.IndirectBank << 16) | ch.IndirectAddress));
            }
            return result;
        }

        public byte HdmaEnable;

        // DMA charges real CPU time now, which SPC700 pacing depends on - see Venus_Memory.md §3.1.
        public int PendingCpuCycles;

        public Dma(MemoryBus bus)
        {
            _bus = bus;
            _channels = new DmaChannel[8];
            for (int i = 0; i < 8; i++) _channels[i] = new DmaChannel();
        }

        public byte ReadRegister(uint address)
        {
            int channel = (int)((address >> 4) & 0x07);
            int reg = (int)(address & 0x0F);
            DmaChannel ch = _channels[channel];

            switch (reg)
            {
                case 0x0: return ch.Control;
                case 0x1: return ch.DestinationReg;
                case 0x2: return (byte)(ch.SourceAddress & 0xFF);
                case 0x3: return (byte)(ch.SourceAddress >> 8);
                case 0x4: return ch.SourceBank;
                case 0x5: return (byte)(ch.TransferSize & 0xFF);
                case 0x6: return (byte)(ch.TransferSize >> 8);
                case 0x7: return ch.IndirectBank;
                case 0x8: return (byte)(ch.TableAddress & 0xFF);
                case 0x9: return (byte)(ch.TableAddress >> 8);
                case 0xA: return ch.LineCounter;
                default: return 0xFF; 
            }
        }

        // Logs which PC wrote a channel's source address - see Venus_Memory.md §3.3.
        private void LogSourceAddrWrite(int channel, DmaChannel ch)
        {
            if (!DebugSettings.DmaSourceAddrLogging) return;
            string pc = "unknown";
            string instrBytes = "";
            if (_bus.DebugPcProvider != null)
            {
                (byte pb, ushort pcVal) = _bus.DebugPcProvider();
                pc = $"0x{pb:X2}{pcVal:X4}";
                // Up to 4 bytes covers every 65816 instruction length.
                var b = new byte[4];
                for (int k = 0; k < 4; k++) b[k] = _bus.Read8((uint)((pb << 16) | ((pcVal + k) & 0xFFFF)));
                instrBytes = $" bytes={b[0]:X2} {b[1]:X2} {b[2]:X2} {b[3]:X2}";
            }
            Console.WriteLine($"[DMA-SRC] Ch{channel} SourceAddress now 0x{ch.SourceBank:X2}{ch.SourceAddress:X4}, written by PC={pc}{instrBytes}");
        }

        public void WriteRegister(uint address, byte data)
        {
            int channel = (int)((address >> 4) & 0x07);
            int reg = (int)(address & 0x0F);
            DmaChannel ch = _channels[channel];

            switch (reg)
            {
                case 0x0: ch.Control = data; break;
                case 0x1: ch.DestinationReg = data; break;
                case 0x2:
                    ch.SourceAddress = (ushort)((ch.SourceAddress & 0xFF00) | data);
                    LogSourceAddrWrite(channel, ch);
                    break;
                case 0x3:
                    ch.SourceAddress = (ushort)((ch.SourceAddress & 0x00FF) | (data << 8));
                    LogSourceAddrWrite(channel, ch);
                    break;
                case 0x4: ch.SourceBank = data; break;
                case 0x5: ch.TransferSize = (ushort)((ch.TransferSize & 0xFF00) | data); break;
                case 0x6: ch.TransferSize = (ushort)((ch.TransferSize & 0x00FF) | (data << 8)); break;
                case 0x7: ch.IndirectBank = data; break;
                case 0x8: ch.TableAddress = (ushort)((ch.TableAddress & 0xFF00) | data); break;
                case 0x9: ch.TableAddress = (ushort)((ch.TableAddress & 0x00FF) | (data << 8)); break;
                case 0xA: ch.LineCounter = data; break;
            }
        }

        private static readonly int[][] TransferPatterns = new int[][]
        {
            new[] { 0 },
            new[] { 0, 1 },
            new[] { 0, 0 },
            new[] { 0, 0, 1, 1 },
            new[] { 0, 1, 2, 3 },
            new[] { 0, 1, 0, 1 }, 
            new[] { 0, 0, 0, 0 },
            new[] { 0, 0, 1, 1 },
        };

        // A-bus arbitration and the WRAM/$2180 conflict - see Venus_Memory.md §3.1a and §3.1b.
        private byte CopyDmaByte(uint aBusAddress, uint bBusAddress, bool fromBtoA)
        {
            ushort aBusOffset = (ushort)(aBusAddress & 0xFFFF);
            // Bank matters here, not just the offset - see Venus_Memory.md §3.1a.
            byte aBusBank = (byte)(aBusAddress >> 16);
            bool aBusIsHardwareBank = aBusBank <= 0x3F || (aBusBank >= 0x80 && aBusBank <= 0xBF);
            bool aBusBlocked = aBusIsHardwareBank && (
                (aBusOffset >= 0x2100 && aBusOffset <= 0x21FF)
                || aBusOffset == 0x420B
                || aBusOffset == 0x420C
                || (aBusOffset >= 0x4300 && aBusOffset <= 0x437F));

            if (aBusBlocked)
            {
                byte openBus = _bus.LastBusValue;
                if (!fromBtoA)
                {
                    // Read side blocked: the B-bus destination still receives open bus - see §3.1b.
                    _bus.Write8(bBusAddress, openBus);
                }
                // Write side blocked: nothing reaches the A-bus address at all - see §3.1b.
                return openBus;
            }

            bool conflict = bBusAddress == 0x2180 && MemoryBus.IsWorkRam(aBusAddress);

            if (fromBtoA)
            {
                if (!conflict)
                {
                    byte value = _bus.Read8(bBusAddress);
                    _bus.Write8(aBusAddress, value);
                    return value;
                }
                _bus.Write8(aBusAddress, 0xFF);
                return 0xFF;
            }

            if (!conflict)
            {
                byte value = _bus.Read8(aBusAddress);
                _bus.Write8(bBusAddress, value);
                return value;
            }
            return 0; // WRAM -> $2180 conflict: no write occurs at all.
        }

        public void ExecuteGeneralDma(byte channelMask)
        {
            for (int i = 0; i < 8; i++)
            {
                if ((channelMask & (1 << i)) == 0) continue;

                DmaChannel ch = _channels[i];
                uint destB = (uint)(0x2100 | ch.DestinationReg);
                int remaining = ch.TransferSize == 0 ? 0x10000 : ch.TransferSize;
                int loggedLength = remaining;
                int loggedSource = (ch.SourceBank << 16) | ch.SourceAddress;
                bool bToA = (ch.Control & 0x80) != 0;
                int aStep = (ch.Control & 0x08) != 0 ? 0 : ((ch.Control & 0x10) != 0 ? -1 : 1);
                int[] pattern = TransferPatterns[ch.Control & 0x07];

                if (DebugSettings.DmaVerboseLogging)
                {
                    string targetInfo = "";
                    if (ch.DestinationReg == 0x18 || ch.DestinationReg == 0x19)
                    {
                        int vramByteAddr = _bus.Ppu.CurrentVramAddr * 2;
                        targetInfo = $" VRAM@0x{vramByteAddr:X4}-0x{(vramByteAddr + remaining - 1) & 0xFFFF:X4}";
                    }
                    else if (ch.DestinationReg == 0x22)
                    {
                        targetInfo = $" CGRAM@0x{_bus.Ppu.CurrentCgAddr * 2:X3}";
                    }
                    Console.WriteLine($"[DMA] Ch{i}: {(bToA ? "PPU->CPU" : "CPU->PPU")} src=0x{ch.SourceBank:X2}{ch.SourceAddress:X4} destReg=0x{destB:X4}{targetInfo} size={remaining} pattern={ch.Control & 0x07} step={aStep}");
                }

                // Both directions run; only the read and write sides swap - see Venus_Memory.md §3.1a.
                PendingCpuCycles += 1 + remaining;

                ushort addr = ch.SourceAddress;
                int patternIdx = 0;

                while (remaining > 0)
                {
                    uint aBusAddr = (uint)((ch.SourceBank << 16) | addr);
                    uint bBusAddr = destB + (uint)pattern[patternIdx];
                    CopyDmaByte(aBusAddr, bBusAddr, bToA);

                    addr = (ushort)(addr + aStep);
                    patternIdx = (patternIdx + 1) % pattern.Length;
                    remaining--;
                }

                ch.SourceAddress = addr;
                ch.TransferSize = 0;

                DmaLog?.Note(i, DmaTransferKind.General, ch.Control, ch.DestinationReg, loggedSource, loggedLength);
            }
        }

        // A $420C bit going 0->1 arms that channel now - see Venus_Memory.md §3.2a.
        public void WriteHdmaEnable(byte data)
        {
            int newlyEnabled = data & ~HdmaEnable;
            HdmaEnable = data;
            if (newlyEnabled == 0) return;
            for (int i = 0; i < 8; i++)
            {
                if ((newlyEnabled & (1 << i)) != 0) _channels[i].HdmaActive = true;
            }
            ArmHdmaChannels(newlyEnabled);
        }

        // Frame start: $420C alone decides which channels are live this frame.
        public void InitHdma()
        {
            for (int i = 0; i < 8; i++) _channels[i].HdmaActive = (HdmaEnable & (1 << i)) != 0;
            ArmHdmaChannels(HdmaEnable);
        }

        private void ArmHdmaChannels(int mask)
        {
            for (int i = 0; i < 8; i++)
            {
                if ((mask & (1 << i)) == 0) continue;
                DmaChannel ch = _channels[i];
                if (!ch.HdmaActive) continue;

                ch.TableAddress = ch.SourceAddress;
                ch.LineCounter = _bus.Read8((uint)((ch.SourceBank << 16) | ch.TableAddress++));
                ch.HdmaDoTransfer = true; 

                if (ch.LineCounter == 0) 
                {
                    ch.HdmaActive = false;
                }
                else if ((ch.Control & 0x40) != 0) 
                {
                    byte low = _bus.Read8((uint)((ch.SourceBank << 16) | ch.TableAddress++));
                    byte high = _bus.Read8((uint)((ch.SourceBank << 16) | ch.TableAddress++));
                    ch.IndirectAddress = (ushort)((high << 8) | low);
                }
            }
        }

        public void ExecuteHdma()
        {
            for (int i = 0; i < 8; i++)
            {
                if ((HdmaEnable & (1 << i)) == 0) continue;
                
                DmaChannel ch = _channels[i];
                if (!ch.HdmaActive) continue;

                if (ch.HdmaDoTransfer)
                {
                    bool bToA = (ch.Control & 0x80) != 0; // real hardware honors this bit for HDMA too, however rarely a game sets it
                    int[] pattern = TransferPatterns[ch.Control & 0x07];
                    DmaLog?.Note(i, DmaTransferKind.Hdma, ch.Control, ch.DestinationReg,
                        (ch.Control & 0x40) != 0
                            ? (ch.IndirectBank << 16) | ch.IndirectAddress
                            : (ch.SourceBank << 16) | ch.TableAddress,
                        pattern.Length);
                    for (int p = 0; p < pattern.Length; p++)
                    {
                        uint aBusAddr;
                        if ((ch.Control & 0x40) != 0)
                            aBusAddr = (uint)((ch.IndirectBank << 16) | ch.IndirectAddress++);
                        else
                            aBusAddr = (uint)((ch.SourceBank << 16) | ch.TableAddress++);

                        uint bBusAddr = (uint)(0x2100 + ch.DestinationReg + pattern[p]);
                        byte val = CopyDmaByte(aBusAddr, bBusAddr, bToA);

                        // Is anything driving the window registers? That is how an animated wipe is done.
                        if (bBusAddr >= 0x2126 && bBusAddr <= 0x2129 && DebugSettings.WindowHdmaLogging)
                        {
                            Console.WriteLine($"[HDMA-WINDOW] Ch{i} wrote 0x{val:X2} to $21{bBusAddr & 0xFF:X2} (scanline={_bus.CurrentScanline})");
                        }
                    }
                }

                // The whole byte decrements, flag included, so 0x80 borrows to 0x7F - see Venus_Memory.md §3.2.
                ch.LineCounter--;
                ch.HdmaDoTransfer = (ch.LineCounter & 0x80) != 0;

                // If the block is finished, fetch the next one
                if ((ch.LineCounter & 0x7F) == 0)
                {
                    ushort fetchedFrom = ch.TableAddress;
                    ch.LineCounter = _bus.Read8((uint)((ch.SourceBank << 16) | ch.TableAddress++));

                    if (i == 7 && (ch.DestinationReg == 0x26 || ch.DestinationReg == 0x27) && DebugSettings.WindowHdmaLogging)
                    {
                        Console.WriteLine($"[HDMA-BLOCK] Ch{i} fetched new block header 0x{ch.LineCounter:X2} from table@0x{fetchedFrom:X4} (bank=0x{ch.SourceBank:X2})");
                    }
                    
                    if ((ch.Control & 0x40) != 0) 
                    {
                        if (ch.LineCounter != 0)
                        {
                            byte low = _bus.Read8((uint)((ch.SourceBank << 16) | ch.TableAddress++));
                            byte high = _bus.Read8((uint)((ch.SourceBank << 16) | ch.TableAddress++));
                            ch.IndirectAddress = (ushort)((high << 8) | low);
                        }
                    }

                    if (ch.LineCounter == 0) ch.HdmaActive = false;
                    ch.HdmaDoTransfer = true; 
                }
            }
        }
    }
}