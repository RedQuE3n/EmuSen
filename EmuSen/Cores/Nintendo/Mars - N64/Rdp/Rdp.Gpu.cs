using EmuSen.Cores.Nintendo.Mars.Rdp.Gpu;

namespace EmuSen.Cores.Nintendo.Mars.Rdp
{
    // A processor at the multiple that walks and hands its rows to a device instead of shading them - see Mars_Gpu.md §5.
    public sealed partial class Rdp
    {
        [EmuSen.Common.SkipInState] private GpuRasteriser? _gpu;

        // Only a processor at a multiple, drawing alone: the machine's own picture is never the device's - see Mars_GpuPlan.md §0.
        public void ShadeOn(GpuRasteriser? gpu)
        {
            if (gpu is not null && !_scaled) throw new InvalidOperationException("the device shades the multiple, never the machine's picture");
            _gpu = gpu;
            if (gpu is not null) { _alone = true; _workers = 1; }
        }

        private void RecordForTheDevice((int First, int Last) rows)
        {
            GpuRasteriser gpu = _gpu!;
            if (CycleType != FillCycle) { gpu.NotShaded(); return; }

            gpu.Image(_colorImage & ~(uint)Math.Max(_colorImageBytes - 1, 0), _colorImageWidth, _colorImageBytes);
            int primitive = gpu.FillPrimitive(_fillColor);
            for (int y = rows.First; y <= rows.Last; y++)
                if (_spanDrawn[y]) gpu.Row(primitive, y, _spanLeft[y], _spanRight[y]);
        }
    }
}
