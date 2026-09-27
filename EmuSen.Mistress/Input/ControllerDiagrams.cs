using System;
using System.Collections.Generic;
using EmuSen.Galaxia.Input;
using EmuSen.LunaP.Controls;

namespace EmuSen.Mistress.Input
{
    // Which drawing stands for each console's pad, and which region of it each bindable control is - see EmuSen_Settings_Reference.md §4.81.
    public static class ControllerDiagrams
    {
        public static ControllerLayout LayoutFor(string console) => console.ToUpperInvariant() switch
        {
            "NES" => ControllerLayout.Nes,
            "SNES" => ControllerLayout.Snes,
            "N64" => ControllerLayout.Nintendo64,
            "GB" or "GBC" => ControllerLayout.GameBoy,
            _ => ControllerLayout.Gamepad,
        };

        // The N64's Z is the template's L2, its C buttons the right stick and its stick the left one - see Mars_Core.md §5.
        private static readonly Dictionary<PadControl, string> Nintendo64 = new()
        {
            [PadControl.L2] = "Z",
            [PadControl.RightStickUp] = "CUp",
            [PadControl.RightStickDown] = "CDown",
            [PadControl.RightStickLeft] = "CLeft",
            [PadControl.RightStickRight] = "CRight",
            [PadControl.LeftStickUp] = "StickUp",
            [PadControl.LeftStickDown] = "StickDown",
            [PadControl.LeftStickLeft] = "StickLeft",
            [PadControl.LeftStickRight] = "StickRight",
        };

        // The drawings finished so far; a console without one shows its list alone.
        public static bool IsDrawn(ControllerLayout layout) => layout is ControllerLayout.Snes or ControllerLayout.Nintendo64;

        public static string RegionFor(string console, PadControl control) =>
            LayoutFor(console) == ControllerLayout.Nintendo64 && Nintendo64.TryGetValue(control, out string? region) ? region : control.ToString();

        // The control a region stands for on a console, or null for a region it has no control for.
        public static PadControl? ControlFor(string console, string region, IEnumerable<PadControl> controls)
        {
            foreach (PadControl control in controls)
                if (string.Equals(RegionFor(console, control), region, StringComparison.Ordinal)) return control;
            return null;
        }

        // Past this share of its travel a stick direction counts as held; Mars reads its C buttons at the same share (MarsCore.CButtonThreshold).
        public const double DirectionThreshold = 0.5;
    }
}
