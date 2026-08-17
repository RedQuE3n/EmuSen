using System;

namespace EmuSen.Cores.Nintendo.Venus.Memory.Mappers
{
    // A chip that only claims a small window and leaves the rest of the cartridge on the ordinary - see Venus_NecDSP.md §3.
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
