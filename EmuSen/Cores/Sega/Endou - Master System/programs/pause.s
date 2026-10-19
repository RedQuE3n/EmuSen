; The Pause button: each NMI counts in $C000; RETN returns (Endou_Native.md §9.8).
        di
        ld sp,$DFF0
        xor a
        ld ($C000),a
loop:   jr loop
        org $66
        push af
        ld a,($C000)
        inc a
        ld ($C000),a
        pop af
        retn
