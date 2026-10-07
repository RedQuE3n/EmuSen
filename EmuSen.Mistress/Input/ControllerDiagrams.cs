using System;
using System.Collections.Generic;
using EmuSen.Galaxia.Input;
using EmuSen.LunaP.Controls;

namespace EmuSen.Mistress.Input
{
    // Which drawing stands for each console's pad, and which region of it each bindable control is - see EmuSen_Settings_Reference.md §4.81.
    public static class ControllerDiagrams
    {
        // The three-button pad's controller id in the Genesis engine's info; any other pad of the console is drawn as the six-button one.
        public const string GenesisThreeButton = "md.pad3";

        // <controller> is the pad the port is set to where the window can tell, by its id in the engine's info - see EmuSen_Settings_Reference.md §4.93.
        public static ControllerLayout LayoutFor(string console, string? controller = null) => console.ToUpperInvariant() switch
        {
            "NES" => ControllerLayout.Nes,
            "SNES" => ControllerLayout.Snes,
            "N64" => ControllerLayout.Nintendo64,
            "GB" or "GBC" => ControllerLayout.GameBoy,
            _ when IsGenesis(console) => controller == GenesisThreeButton ? ControllerLayout.Genesis : ControllerLayout.GenesisSixButton,
            _ => ControllerLayout.Gamepad,
        };

        // The Genesis and its two attachments share the console's pads.
        public static bool IsGenesis(string console) => console.ToUpperInvariant() is "GENESIS" or "SEGA CD" or "32X";

        // The Genesis pad's buttons by the RetroPad controls the core gives them: A on Y, C on A, X on L, Y on X, Z on R and Mode on Select - see Nephrite_Plan.md §4.2.
        private static readonly Dictionary<PadControl, string> Genesis = new()
        {
            [PadControl.Y] = "A",
            [PadControl.A] = "C",
            [PadControl.L] = "X",
            [PadControl.X] = "Y",
            [PadControl.R] = "Z",
            [PadControl.Select] = "Mode",
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

        public static string RegionFor(string console, PadControl control) =>
            LayoutFor(console) == ControllerLayout.Nintendo64 && Nintendo64.TryGetValue(control, out string? region) ? region
            : IsGenesis(console) && Genesis.TryGetValue(control, out string? button) ? button
            : control.ToString();

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
