namespace EmuSen.Cores.Nintendo.Venus.Memory.Mappers
{
    // Cartridge type $1x - see Venus_SuperFX.md §3.
    public sealed class SuperFxMapper : ICartridgeMapper
    {
        private readonly Coprocessors.SuperFx.SuperFx _gsu;

        public SuperFxMapper(Coprocessors.SuperFx.SuperFx gsu)
        {
            _gsu = gsu;
        }

        public string Name => "SuperFX";

        public CartridgeAddress Resolve(byte bank, ushort offset) => _gsu.ResolveScpu(bank, offset);
    }
}
