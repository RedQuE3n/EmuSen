| cddprobe, the main CPU's half: a boot ROM in mode 2 that loads the sub program into PRG-RAM and lets it run.
| The drive probe of Nephrite_Disputes.md D-36. Built with GNU binutils for m68k:
|   m68k-linux-gnu-as -m68000 --register-prefix-optional -o sub.o sub.s
|   m68k-linux-gnu-ld -Ttext=0 --oformat=binary -o sub.bin sub.o
|   m68k-linux-gnu-as -m68000 --register-prefix-optional -o main.o main.s
|   m68k-linux-gnu-ld -Ttext=0 --oformat=binary -o cddprobe.bin main.o
| cddprobe.bin is a 128 KiB boot ROM; the records are 16 bytes each from PRG-RAM $40000 (sub.s says what they hold).
    .text
    .org 0
    dc.l 0x00FFFE00, start
    .rept 62
    dc.l nothing
    .endr
    .org 0x100
    .ascii "SEGA MEGA DRIVE "
    .ascii "(C)TEST 2026.OCT"
    .org 0x200
start:
    move.w #0x2700,sr
    move.b #2,0xA12001
1:  btst #1,0xA12001
    beq 1b
    move.b #0,0xA12002
    move.b #0,0xA12003
    lea subprog,a0
    lea 0x020000,a1
    move.w #(subend-subprog)/2-1,d0
2:  move.w (a0)+,(a1)+
    dbra d0,2b
    move.b #1,0xA12001
3:  bra 3b
nothing:
    rte
    .balign 2
subprog:
    .incbin "sub.bin"
subend:
    .org 0x1FFFE
    dc.w 0
