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

        // The PlayStation's digital pad in its engine's info; any other pad of the console is drawn as the DualShock.
        public const string PlayStationDigital = "psx.pad";

        // <controller> is the pad the port is set to where the window can tell, by its id in the engine's info - see EmuSen_Settings_Reference.md §4.93.
        public static ControllerLayout LayoutFor(string console, string? controller = null) => console.ToUpperInvariant() switch
        {
            "NES" => ControllerLayout.Nes,
            "SNES" => ControllerLayout.Snes,
            "N64" => ControllerLayout.Nintendo64,
            "GB" or "GBC" => ControllerLayout.GameBoy,
            "MASTER SYSTEM" => ControllerLayout.MasterSystem,
            "SG-1000" => ControllerLayout.Sg1000,
            _ when IsGenesis(console) => controller == GenesisThreeButton ? ControllerLayout.Genesis : ControllerLayout.GenesisSixButton,
            "PLAYSTATION" => controller == PlayStationDigital ? ControllerLayout.PlayStation : ControllerLayout.DualShock,
            _ => ControllerLayout.Gamepad,
        };

        // The Genesis and its attachments, which plug into it and share its pads - see EmuSen_Serenity.md §3.12.
        public static bool IsGenesis(string console) => string.Equals(Library.RuntimeCatalog.HostOf(console), Library.RuntimeCatalog.ConsoleOfSystem(MegaDriveSystem), StringComparison.OrdinalIgnoreCase);

        // The systems table's id of the console whose pads the Genesis layouts draw.
        private const string MegaDriveSystem = "md";

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

        // The PlayStation's buttons by the RetroPad controls the core gives them, position for position: Triangle on X, Circle on A, Cross on B, Square on Y - see Diamond_Plan.md §4.2.
        private static readonly Dictionary<PadControl, string> PlayStation = new()
        {
            [PadControl.X] = "Triangle",
            [PadControl.A] = "Circle",
            [PadControl.B] = "Cross",
            [PadControl.Y] = "Square",
            [PadControl.L] = "L1",
            [PadControl.R] = "R1",
        };

        // Buttons 1 and 2 and the console's Pause and Reset by the RetroPad controls Endou gives them: 1 on B, 2 on A, Pause on Start, Reset on Select - see Endou_Plan.md §4.2.
        private static readonly Dictionary<PadControl, string> MasterSystem = new()
        {
            [PadControl.B] = "1",
            [PadControl.A] = "2",
            [PadControl.Start] = "Pause",
            [PadControl.Select] = "Reset",
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
            : string.Equals(console, "PlayStation", StringComparison.OrdinalIgnoreCase) && PlayStation.TryGetValue(control, out string? face) ? face
            : LayoutFor(console) is ControllerLayout.MasterSystem or ControllerLayout.Sg1000 && MasterSystem.TryGetValue(control, out string? sega) ? sega
            : control.ToString();

        // The control a region stands for on a console, or null for a region it has no control for.
        public static PadControl? ControlFor(string console, string region, IEnumerable<PadControl> controls)
        {
            foreach (PadControl control in controls)
                if (string.Equals(RegionFor(console, control), region, StringComparison.Ordinal)) return control;
            return null;
        }

        // Past this share of its travel a stick direction counts as held; the N64 reads its C buttons at the same share (MarsRT's set_axis, Mars_Core.md §5).
        public const double DirectionThreshold = 0.5;
    }
}
