namespace EmuSen.Cores.Nintendo.Venus.Memory.Mappers
{
    // Cartridge type $1x. Static arithmetic, unlike the SA-1's - the GSU has no
    // bank registers that move the S-CPU's view - but it lives on the chip
    // anyway so both sides share one decode. See Venus_SuperFX.md §3.
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
