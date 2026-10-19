; The pads: ports $DC and $DD copied to $C000 and $C001 every pass (Endou_Native.md §9.8).
        di
        ld sp,$DFF0
loop:   in a,($DC)
        ld ($C000),a
        in a,($DD)
        ld ($C001),a
        jr loop
