using System;
using System.IO;
using EmuSen.Cores.Nintendo.Venus.Memory.Mappers;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.Cores.Nintendo.Venus.Memory
{
    public class Cartridge
    {
        [EmuSen.Common.SkipInState] private byte[] _rom;
        private byte[] _sram;
        public int SramSize => _sram.Length;

        // All three are derived from the ROM file, which LoadRom re-reads
        // before any state load, so none belong in a save state.
        [EmuSen.Common.SkipInState] private readonly ICartridgeMapper _mapper;
        [EmuSen.Common.SkipInState] private readonly bool _isHiRom;
        [EmuSen.Common.SkipInState] private readonly ConsoleRegion _region;

        private const int LoRomHeader = 0x7FC0;
        private const int HiRomHeader = 0xFFC0;
        private const byte Sa1MapMode = 0x23;

        // The most Game Pak RAM a GSU can address - see Venus_SuperFX.md §1.
        private const int SuperFxRamSize = 0x20000;

        // How much of _sram is battery-backed and belongs in the .srm. Equal to
        // _sram.Length everywhere except SuperFX, where the RAM chip is larger
        // than the saved window.
        [EmuSen.Common.SkipInState] private readonly int _batteryRamSize;

        // Non-null only for an SA-1 cartridge - see Venus_SA1.md §1. Kept out
        // of Cartridge's own state blob so a state version 1 file, which
        // predates the SA-1 entirely, still has the layout it was written
        // with; VenusCore appends this separately - see EmuSen_Save_States.md §3.
        // MemoryBus routes cartridge writes straight here, so without this
        // `watch` never saw SRAM or either SA-1 memory - see Venus_SA1.md §11.4.
        [EmuSen.Common.SkipInState] public IWriteObserver? WriteObserver;

        // Coprocessor register-window traffic, when a debugger is watching - see `man copflow`.
        [EmuSen.Common.SkipInState] public EmuSen.DianaOS.DianaOS.Var.RegisterFlowRegistry? RegisterFlow;

        [EmuSen.Common.SkipInState] public Coprocessors.Sa1.Sa1? Sa1;

        // Non-null only for a SuperFX cartridge - see Venus_SuperFX.md §1. Same
        // save-state handling as Sa1 above.
        [EmuSen.Common.SkipInState] public Coprocessors.SuperFx.SuperFx? SuperFx;

        // Non-null only for a DSP-1/2/3/4 or ST010/ST011 cartridge whose
        // firmware was found - see Venus_NecDSP.md §2. Same save-state
        // handling as Sa1 above.
        [EmuSen.Common.SkipInState] public Coprocessors.NecDsp.NecDsp? NecDsp;

        // Non-null only for an OBC1 cartridge - see Venus_OBC1.md §1. Holds no
        // state of its own, so it never reaches a save state at all.
        [EmuSen.Common.SkipInState] public Coprocessors.Obc1.Obc1? Obc1;

        public string MapperName => _mapper.Name;

        // Which console this cartridge expects - see Venus_Memory.md §2.5.
        public ConsoleRegion Region => _region;

        // Saves/<rom-name>.srm - see Venus_Memory.md §2.4.
        public string SavePath { get; }

        // Everything the header says that both the constructor and the
        // no-load firmware query below need - see Venus_Memory.md §2.1a.
        private readonly record struct HeaderInfo(int Base, bool IsHiRom, byte MapMode, byte CartType, byte ChipType, string CartName);

        // SNES ROM copiers often appended a 512-byte header to the file. A
        // standard LoROM without one divides cleanly by 32KB.
        private static byte[] StripCopierHeader(byte[] fileBytes, bool announce)
        {
            int headerSize = (fileBytes.Length % 32768 == 512) ? 512 : 0;
            if (headerSize == 0) return fileBytes;

            if (announce) Console.WriteLine("[Cartridge] Copier header detected. Stripping 512 bytes...");
            byte[] stripped = new byte[fileBytes.Length - headerSize];
            Array.Copy(fileBytes, headerSize, stripped, 0, stripped.Length);
            return stripped;
        }

        private static HeaderInfo ReadHeader(byte[] rom)
        {
            // Which of the two header locations is real decides the whole
            // memory map - see Venus_Memory.md §2.1a.
            bool isHiRom = ScoreHeader(rom, HiRomHeader, hiRom: true) > ScoreHeader(rom, LoRomHeader, hiRom: false);
            int headerBase = isHiRom ? HiRomHeader : LoRomHeader;

            byte At(int offset) => rom.Length > headerBase + offset && headerBase + offset >= 0 ? rom[headerBase + offset] : (byte)0x00;

            // The chip-subtype byte at header -$01 disambiguates the $Fx
            // cartridge types, and the name picks between DSP revisions that
            // share one type byte - see Venus_NecDSP.md §1.
            return new HeaderInfo(headerBase, isHiRom, At(0x15), At(0x16), At(-0x01), ReadCartName(rom, headerBase));
        }

        public Cartridge(string romPath)
        {
            _rom = StripCopierHeader(File.ReadAllBytes(romPath), announce: true);

            HeaderInfo header = ReadHeader(_rom);
            _isHiRom = header.IsHiRom;
            int headerBase = header.Base;

            // SRAM size from the ROM header (+$18) - see Venus_Memory.md §2.2.
            // On an SA-1 cartridge this same chip is the SA-1's BW-RAM.
            int sramSize = 0;
            if (_rom.Length > headerBase + 0x18)
            {
                int ramSizeExponent = _rom[headerBase + 0x18];
                if (ramSizeExponent > 0)
                {
                    // Clamp to SnesLab's documented real-hardware max (512KB).
                    ramSizeExponent = Math.Min(ramSizeExponent, 7);
                    sramSize = 1024 << ramSizeExponent;
                }
            }
            byte mapMode = header.MapMode;
            byte cartType = header.CartType;
            Coprocessors.NecDsp.NecDspVariant? dspVariant = Coprocessors.NecDsp.NecDspDetection.Detect(cartType, header.ChipType, header.CartName);
            bool isObc1 = (cartType & 0x0F) >= 0x03 && (cartType & 0xF0) == 0x20;

            // A SuperFX cartridge declares no ordinary SRAM: its one RAM chip is
            // the GSU's Game Pak RAM, sized by the expansion-RAM byte instead -
            // see Venus_SuperFX.md §1.
            bool isSuperFx = !_isHiRom && (cartType & 0xF0) == 0x10 && (cartType & 0x0F) >= 0x03;
            int batteryRamSize = sramSize;
            if (isSuperFx)
            {
                batteryRamSize = ExpansionRamSize(_rom, headerBase);
                sramSize = SuperFxRamSize;
            }

            // An ST010/ST011 declares SRAM in the header, but the chip that
            // holds it is the DSP - see Venus_NecDSP.md §6.
            if (dspVariant is { } st && Coprocessors.NecDsp.NecDspProfile.IsSt01x(st))
            {
                sramSize = 0;
                batteryRamSize = 0;
            }

            _sram = new byte[sramSize];
            _batteryRamSize = System.Math.Min(batteryRamSize, sramSize);

            // Both of these only claim a window; the cartridge keeps its
            // ordinary map underneath - see Venus_NecDSP.md §3, Venus_OBC1.md §1.
            if (dspVariant is { } variant) NecDsp = BuildNecDsp(variant);
            else if (isObc1 && _sram.Length > 0) Obc1 = new Coprocessors.Obc1.Obc1(_sram);

            // Map mode $23 means the cartridge carries an SA-1, which takes
            // over addressing entirely - see Venus_SA1.md §1.
            if (!_isHiRom && mapMode == Sa1MapMode)
            {
                Sa1 = new Coprocessors.Sa1.Sa1(_rom, _sram);
                _mapper = new Sa1Mapper(Sa1);
            }
            else if (isSuperFx)
            {
                SuperFx = new Coprocessors.SuperFx.SuperFx(_rom, _sram);
                _mapper = new SuperFxMapper(SuperFx);
            }
            else
            {
                ICartridgeMapper baseMapper = _isHiRom ? new HiRomMapper() : new LoRomMapper();
                if (NecDsp is { } dsp) _mapper = new CoprocessorOverlayMapper(dsp.Name, baseMapper, dsp.ResolveScpu);
                else if (Obc1 is { } obc1) _mapper = new CoprocessorOverlayMapper("OBC1", baseMapper, obc1.ResolveScpu);
                else _mapper = baseMapper;
            }

            // Country byte (+$19) - see Venus_Memory.md §2.5.
            byte country = _rom.Length > headerBase + 0x19 ? _rom[headerBase + 0x19] : (byte)0x01;
            _region = ConsoleRegions.FromCountryCode(country);

            string saveDir = DianaOSSandbox.SavesDirectory;
            string romName = Path.GetFileNameWithoutExtension(romPath);
            SavePath = Path.Combine(saveDir, romName + ".srm");
            LoadSram();

            Console.WriteLine("=== Cartridge Loaded ===");
            Console.WriteLine($"Mapper: {_mapper.Name}");
            Console.WriteLine($"ROM Size: {_rom.Length / 1024} KB");
            Console.WriteLine($"SRAM Size: {_sram.Length / 1024} KB");
            Console.WriteLine($"Region: {(_region == ConsoleRegion.Pal ? "PAL" : "NTSC")} (country 0x{country:X2})");
            Console.WriteLine($"Save Path: {SavePath}");
            Console.WriteLine("========================");
        }

        // Tolerant of a save file that doesn't match the allocated SRAM
        // size - see Venus_Memory.md §2.4.
        private void LoadSram()
        {
            try
            {
                if (BatteryRamDisabled || !File.Exists(SavePath)) return;

                byte[] saved = File.ReadAllBytes(SavePath);

                // On an ST010/ST011 the .srm is the DSP's own data RAM - see Venus_NecDSP.md §6.
                if (NecDsp is { HasBatteryRam: true } dsp)
                {
                    dsp.ImportBatteryRam(saved);
                    Console.WriteLine($"[Cartridge] Loaded save: {SavePath} ({saved.Length} bytes, {dsp.Name} RAM)");
                    return;
                }

                int count = Math.Min(saved.Length, _batteryRamSize);
                Array.Copy(saved, _sram, count);
                Console.WriteLine($"[Cartridge] Loaded save: {SavePath} ({count} bytes)");
            }
            catch (Exception ex)
            {
                // A save that fails to load shouldn't prevent the game itself
                // from booting - worst case the player starts without their
                // save data, not with a crashed emulator.
                Console.WriteLine($"[Cartridge] Failed to load save ({SavePath}): {ex.Message}");
            }
        }

        // Pharaoh's --nobattery, now core-agnostic - see Venus_Memory.md §2.4a and EmuSen_Multicore.md §6.
        public static bool BatteryRamDisabled
        {
            get => EmuSen.Cores.CoreOptions.BatteryRamDisabled;
            set => EmuSen.Cores.CoreOptions.BatteryRamDisabled = value;
        }

        // Called periodically + on shutdown, not on every write - see
        // Venus_Memory.md §2.4.
        public void SaveSram()
        {
            try
            {
                if (BatteryRamDisabled) return;
                string? dir = Path.GetDirectoryName(SavePath);
                if (!string.IsNullOrEmpty(dir)) Directory.CreateDirectory(dir);

                if (NecDsp is { HasBatteryRam: true } dsp)
                {
                    File.WriteAllBytes(SavePath, dsp.ExportBatteryRam());
                    return;
                }

                if (_batteryRamSize == 0) return;
                File.WriteAllBytes(SavePath, _sram.AsSpan(0, _batteryRamSize).ToArray());
            }
            catch (Exception ex)
            {
                Console.WriteLine($"[Cartridge] Failed to save ({SavePath}): {ex.Message}");
            }
        }

        public byte Read8(uint address)
        {
            var mapped = _mapper.Resolve((byte)(address >> 16), (ushort)(address & 0xFFFF));
            switch (mapped.Region)
            {
                case CartridgeRegion.Rom:
                    return mapped.Offset < _rom.Length ? _rom[mapped.Offset] : (byte)0x00;

                // SRAM mirroring (modulo, not a hard range check) - see
                // Venus_Memory.md §2.3 for why this matters beyond correctness.
                case CartridgeRegion.Sram:
                    return _sram.Length > 0 ? _sram[mapped.Offset % _sram.Length] : (byte)0x00;

                // The three SA-1 regions - see Venus_SA1.md §3.
                case CartridgeRegion.IRam:
                    return Sa1!.IRam[mapped.Offset];

                case CartridgeRegion.CoprocessorRegister:
                {
                    byte value = Sa1 != null ? Sa1.ReadRegister((ushort)mapped.Offset)
                               : SuperFx != null ? SuperFx.ReadRegister((ushort)mapped.Offset)
                               : NecDsp != null ? NecDsp.ReadRegister(mapped.Offset)
                               : Obc1!.ReadRegister(mapped.Offset);
                    RegisterFlow?.Note(mapped.Offset, value, isWrite: false, "cpu");
                    return value;
                }

                case CartridgeRegion.Sa1Vector:
                    return Sa1!.VectorByte(mapped.Offset);

                default:
                    return 0x00; // open bus
            }
        }

        public void Write8(uint address, byte data)
        {
            // ROM is read-only; SRAM and the SA-1's own memories are writable.
            var mapped = _mapper.Resolve((byte)(address >> 16), (ushort)(address & 0xFFFF));
            switch (mapped.Region)
            {
                case CartridgeRegion.Sram:
                    if (_sram.Length == 0) return;
                    int sramIndex = mapped.Offset % _sram.Length;
                    _sram[sramIndex] = data;
                    // On an SA-1 cart this same array is BW-RAM, and that is
                    // the space name it is published under - see Venus_SA1.md §11.2.
                    WriteObserver?.OnWrite(Sa1 != null ? "BWRAM" : "SRAM", sramIndex, data);
                    return;

                case CartridgeRegion.IRam:
                    Sa1!.IRam[mapped.Offset] = data;
                    WriteObserver?.OnWrite("SA1IRAM", mapped.Offset, data);
                    return;

                case CartridgeRegion.CoprocessorRegister:
                    RegisterFlow?.Note(mapped.Offset, data, isWrite: true, "cpu");
                    if (Sa1 != null) Sa1.WriteRegister((ushort)mapped.Offset, data);
                    else if (SuperFx != null) SuperFx.WriteRegister((ushort)mapped.Offset, data);
                    else if (NecDsp != null) NecDsp.WriteRegister(mapped.Offset, data);
                    else Obc1!.WriteRegister(mapped.Offset, data);
                    return;
            }
        }

        // Lets MemoryBus route a low-half address in a hardware bank here
        // instead of returning open bus - see Venus_Memory.md §2.1a.
        public bool MapsAddress(uint address)
        {
            return _mapper.Resolve((byte)(address >> 16), (ushort)(address & 0xFFFF)).Region != CartridgeRegion.Unmapped;
        }

        // The same decode Read8 uses, reported rather than followed - see `man addr`.
        public EmuSen.DianaOS.DianaOS.Lib.PhysicalAddress? TryResolvePhysical(uint address)
        {
            var mapped = _mapper.Resolve((byte)(address >> 16), (ushort)(address & 0xFFFF));
            return mapped.Region switch
            {
                CartridgeRegion.Rom => new("ROM", mapped.Offset % System.Math.Max(1, _rom.Length)),
                CartridgeRegion.Sram => new(Sa1 != null ? "BWRAM" : "SRAM", mapped.Offset),
                CartridgeRegion.IRam => new("SA1IRAM", mapped.Offset),
                CartridgeRegion.CoprocessorRegister => new($"coprocessor register ${mapped.Offset:X4}", mapped.Offset, false),
                CartridgeRegion.Sa1Vector => new("SA-1 vector override", mapped.Offset, false),
                _ => null,
            };
        }

        // What this ROM will need before it can be fully emulated, decided
        // from the header alone - no Cartridge is built and nothing is
        // loaded. A ROM carrying its firmware appended needs nothing, so it
        // reports nothing. See EmuSen_Firmware.md §1 and Venus_NecDSP.md §2.
        public static IReadOnlyList<Common.Firmware.FirmwareRequest> FirmwareRequirements(string romPath)
        {
            byte[] rom;
            try
            {
                rom = StripCopierHeader(File.ReadAllBytes(romPath), announce: false);
            }
            catch (Exception ex) when (ex is IOException or UnauthorizedAccessException)
            {
                return Array.Empty<Common.Firmware.FirmwareRequest>();
            }

            HeaderInfo header = ReadHeader(rom);
            if (Coprocessors.NecDsp.NecDspDetection.Detect(header.CartType, header.ChipType, header.CartName) is not { } variant)
            {
                return Array.Empty<Common.Firmware.FirmwareRequest>();
            }

            var profile = Coprocessors.NecDsp.NecDspProfile.For(variant);
            if (Coprocessors.NecDsp.NecDspFirmware.EmbeddedSize(rom.Length) == profile.FirmwareBytes)
            {
                return Array.Empty<Common.Firmware.FirmwareRequest>();
            }

            return new[] { Coprocessors.NecDsp.NecDspFirmware.RequestFor(variant) };
        }

        // The 21-byte, space-padded title at the head of the header.
        private static string ReadCartName(byte[] rom, int headerBase)
        {
            if (rom.Length < headerBase + 0x15) return string.Empty;

            // Latin-1, not UTF-8: SD Gundam GX's title is half-width katakana
            // in the SNES's own encoding, and only a byte-for-byte reading of
            // it matches the name the DSP-3 is keyed on.
            return System.Text.Encoding.Latin1.GetString(rom, headerBase, 0x15).TrimEnd(' ', '\0');
        }

        // Takes the firmware off the end of the ROM if it's there, otherwise
        // out of Usr/Home/Firmware. Returns null - leaving the cartridge on its
        // plain map - if neither has it, since there is nothing useful to run
        // without it. See Venus_NecDSP.md §2.
        private Coprocessors.NecDsp.NecDsp? BuildNecDsp(Coprocessors.NecDsp.NecDspVariant variant)
        {
            var profile = Coprocessors.NecDsp.NecDspProfile.For(variant);

            int embedded = Coprocessors.NecDsp.NecDspFirmware.EmbeddedSize(_rom.Length);
            Coprocessors.NecDsp.NecDspFirmware? firmware = null;
            if (embedded == profile.FirmwareBytes)
            {
                byte[] blob = _rom.AsSpan(_rom.Length - embedded, embedded).ToArray();
                firmware = Coprocessors.NecDsp.NecDspFirmware.FromBlob(profile, blob);

                // The appended firmware is not part of the addressable ROM.
                if (firmware != null) _rom = _rom.AsSpan(0, _rom.Length - embedded).ToArray();
            }

            firmware ??= Coprocessors.NecDsp.NecDspFirmware.FromFirmwareDirectory(profile);
            if (firmware == null)
            {
                // Names the exact path, because every frontend except the
                // Avalonia one (which offers a picker) can only tell the user
                // where to put it - see EmuSen_Firmware.md §3.
                var request = Coprocessors.NecDsp.NecDspFirmware.RequestFor(variant);
                Console.WriteLine($"[Cartridge] {variant} firmware not found - the chip will not be emulated.");
                Console.WriteLine($"[Cartridge]   expected {request.Size:N0} bytes at {Common.Firmware.FirmwareLibrary.PathFor(request)}");
                return null;
            }

            return new Coprocessors.NecDsp.NecDsp(variant, firmware, _isHiRom);
        }

        // SuperFX carts carry their RAM size at header -$03 instead of +$18 - see Venus_SuperFX.md §1.
        private static int ExpansionRamSize(byte[] rom, int headerBase)
        {
            int offset = headerBase - 0x03;
            if (offset < 0 || offset >= rom.Length) return 0;
            int exponent = rom[offset];
            if (exponent == 0 || exponent > 7) return 0;
            return 1024 << exponent;
        }

        // Higher score wins - see Venus_Memory.md §2.1a.
        private static int ScoreHeader(byte[] rom, int baseAddr, bool hiRom)
        {
            if (rom.Length < baseAddr + 0x20) return -1;

            int score = 0;
            int mapMode = rom[baseAddr + 0x15] & 0x0F;
            bool modeMatches = hiRom ? (mapMode == 0x01 || mapMode == 0x05) : (mapMode == 0x00 || mapMode == 0x02 || mapMode == 0x03);
            if (modeMatches) score += 2;

            int complement = rom[baseAddr + 0x1C] | (rom[baseAddr + 0x1D] << 8);
            int checksum = rom[baseAddr + 0x1E] | (rom[baseAddr + 0x1F] << 8);
            if (checksum != 0 && (checksum ^ complement) == 0xFFFF) score += 2;

            return score;
        }
    }
}