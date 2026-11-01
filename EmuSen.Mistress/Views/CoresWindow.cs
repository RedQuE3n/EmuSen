using System;
using System.Threading.Tasks;
using EmuSen.LunaP.Windowing;
using EmuSen.Tools.Cores;

namespace EmuSen.Mistress.Views
{
    // Mistress's thin host for Cores on ToolPaneWindow, the runtime handed to the pane and its approval asked over this window - see EmuSen_Cores.md §3.4.
    public sealed class CoresWindow : ToolPaneWindow
    {
        // The approval question, which a test answers in place of a person.
        internal static Func<Avalonia.Controls.Window, string, string, Task<bool>> Confirm =
            (owner, title, message) => Dialogs.ConfirmAsync(owner, title, message, "Approve", "Not Now");

        // Parameterless constructor exists only for tooling - real code always uses the one below.
        public CoresWindow() : this(null, null) { }

        public CoresWindow(CoresRuntime? runtime, CoresHost? host) : this(new Owner(runtime, host)) { }

        private CoresWindow(Owner owner) : base("Cores", owner.Pane)
        {
            owner.Window = this;
            Pane = owner.Pane;
            Pane.MarksChanged += ShowHints;
        }

        public CoresPane Pane { get; }

        protected override void FirstShown() => Pane.Refresh();

        // The pane, made before the window exists, asking its question over the window once it does.
        private sealed class Owner
        {
            public Owner(CoresRuntime? runtime, CoresHost? host)
            {
                var asked = host ?? new CoresHost();
                Pane = new CoresPane(runtime, new CoresHost { Options = asked.Options, Firmware = asked.Firmware, OpenFolder = asked.OpenFolder, Confirm = (title, message) => Confirm(Window!, title, message) });
            }

            public CoresPane Pane { get; }
            public Avalonia.Controls.Window? Window { get; set; }
        }
    }
}
