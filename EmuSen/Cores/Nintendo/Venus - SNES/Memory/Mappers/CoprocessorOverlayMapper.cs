using System;

namespace EmuSen.Cores.Nintendo.Venus.Memory.Mappers
{
    // A chip that only claims a small window and leaves the rest of the
    // cartridge on the ordinary LoROM/HiROM map - the NEC DSPs and the OBC1,
    // unlike the SA-1 and the GSU, which replace the map outright.
    // See Venus_NecDSP.md §3 and Venus_OBC1.md §1.
    public sealed class CoprocessorOverlayMapper : ICartridgeMapper
    {
        private readonly ICartridgeMapper _baseMapper;
        private readonly Func<byte, ushort, CartridgeAddress> _overlay;

        public CoprocessorOverlayMapper(string name, ICartridgeMapper baseMapper, Func<byte, ushort, CartridgeAddress> overlay)
        {
            Name = name;
            _baseMapper = baseMapper;
            _overlay = overlay;
        }

        public string Name { get; }

        public CartridgeAddress Resolve(byte bank, ushort offset)
        {
            CartridgeAddress claimed = _overlay(bank, offset);
            return claimed.Region != CartridgeRegion.Unmapped ? claimed : _baseMapper.Resolve(bank, offset);
        }
    }
}
