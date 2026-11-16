import EmuSen.Kit as Kit

// The kit's PadGlyph by name, as a theme places it, with the pad's buttons named here since a theme cannot reach the kit's enumeration (EmuSen_Qt.md §3.2).
Kit.PadGlyph {
    enum Button { Up, Down, Left, Right, Accept, Back, Menu, Options, PageUp, PageDown, First, Last, Guide, Search, Random, Screensaver }
    // The kit's glyph styles in its order (kit-qt kinds.rs GlyphStyle), for `style` and HelpBar's `leadStyle`; since 1.4.
    enum Style { Outlined, Filled }
}
