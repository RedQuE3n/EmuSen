using Avalonia.Controls;

namespace EmuSen.LunaP.Controls
{
    // The three text idioms the frontends retyped per window; all styling lives in Theme/Controls.axaml - see EmuSen_LunaP.md §5.1.
    public class SectionHeader : TextBlock
    {
    }

    // Grey 11pt wrapping explanatory text, under a label or a checkbox.
    public class HintText : TextBlock
    {
    }

    // Body text in the monospace stack, for register dumps and runtime figures.
    public class MonoText : TextBlock
    {
    }
}
