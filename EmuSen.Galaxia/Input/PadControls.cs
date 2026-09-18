using System;
using System.Collections.Generic;

namespace EmuSen.Galaxia.Input
{
    // Between the bindable controls and what a core takes: a button, or one direction of a stick - see EmuSen_Input.md §7.2.
    public static class PadControls
    {
        public static PadControl From(PadButton button) => (PadControl)(int)button;

        public static bool IsButton(PadControl control, out PadButton button)
        {
            button = (PadButton)(int)control;
            return control <= PadControl.R3;
        }

        // The two controls that push one stick axis, negative first, or none for a trigger.
        public static (PadControl Negative, PadControl Positive)? Directions(PadAxis axis) => axis switch
        {
            PadAxis.LeftX => (PadControl.LeftStickLeft, PadControl.LeftStickRight),
            PadAxis.LeftY => (PadControl.LeftStickUp, PadControl.LeftStickDown),
            PadAxis.RightX => (PadControl.RightStickLeft, PadControl.RightStickRight),
            PadAxis.RightY => (PadControl.RightStickUp, PadControl.RightStickDown),
            _ => null,
        };

        // The trigger button whose press stands in for the trigger's full travel, for a trigger axis.
        public static PadButton? TriggerButton(PadAxis axis) => axis switch
        {
            PadAxis.LeftTrigger => PadButton.L2,
            PadAxis.RightTrigger => PadButton.R2,
            _ => null,
        };

        // A console's bindable controls: its buttons, then up, down, left and right of each stick axis it reads.
        public static IReadOnlyList<PadControl> For(IEnumerable<PadButton> buttons, IEnumerable<PadAxis> axes)
        {
            var controls = new List<PadControl>();
            foreach (PadButton button in buttons) controls.Add(From(button));

            var read = new HashSet<PadAxis>(axes);
            foreach ((PadAxis x, PadAxis y) in new[] { (PadAxis.LeftX, PadAxis.LeftY), (PadAxis.RightX, PadAxis.RightY) })
            {
                if (read.Contains(y) && Directions(y) is { } vertical) controls.AddRange(new[] { vertical.Negative, vertical.Positive });
                if (read.Contains(x) && Directions(x) is { } horizontal) controls.AddRange(new[] { horizontal.Negative, horizontal.Positive });
            }

            return controls;
        }

        // A key held for one direction pushes the axis all the way; both held cancel, leaving the stick's own reading.
        public static double Combine(double analog, bool negative, bool positive)
        {
            if (negative != positive) return positive ? 1.0 : -1.0;
            return Math.Clamp(analog, -1.0, 1.0);
        }

        // One axis from everything that can move it: the pad's own reading, and the keys or buttons standing in for it.
        public static double Resolve(PadAxis axis, double analog, Func<PadControl, bool> held)
        {
            if (Directions(axis) is { } pair) return Combine(analog, held(pair.Negative), held(pair.Positive));
            if (TriggerButton(axis) is { } button && held(From(button))) return 1.0;
            return Math.Clamp(analog, 0.0, 1.0);
        }
    }
}
