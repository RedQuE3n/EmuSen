# D-10: GPUSTAT's ready bits while a polygon, a line and a rectangle are received and drawn by the CPU
# (Diamond_Disputes.md D-10). For each, GPUSTAT once the GPU is idle, after the command word, after each further
# word, about 300 cycles into the drawing and once it is done; then a triangle drawn with a second one queued behind
# it. Results from 80100000h, one GPUSTAT a word, in that order: the triangle's 7, the line's 5, the rectangle's 5,
# the queued pair's 3. 800FFFF0h holds D1A0D00Eh when done.
        j start
        nop
start:  mtc0 zero, SR
        lui a0, 0x1F80
        lui s0, 0x8010
        sw zero, 0x1814(a0)           # GP1(00h)
        lui t0, 0xE100
        ori t0, t0, 0x0400
        sw t0, 0x1810(a0)             # GP0(E1h): drawing to the display allowed
        lui t0, 0xE400
        lui t1, 0x0007
        ori t1, t1, 0xFFFF
        or t0, t0, t1
        sw t0, 0x1810(a0)             # GP0(E4h): area to 1023,511
        jal wait
        nop
# a flat triangle 512 wide and 256 high
        jal stat
        nop
        lui t0, 0x2000
        ori t0, t0, 0x00FF
        sw t0, 0x1810(a0)
        jal stat
        nop
        sw zero, 0x1810(a0)           # 0,0
        jal stat
        nop
        ori t0, zero, 0x0200
        sw t0, 0x1810(a0)             # 512,0
        jal stat
        nop
        lui t0, 0x0100
        sw t0, 0x1810(a0)             # 0,256
        jal stat
        nop
        jal short
        nop
        jal stat
        nop
        jal wait
        nop
        jal stat
        nop
# a flat line from 0,300 to 1000,310
        lui t0, 0x4000
        ori t0, t0, 0xFF00
        sw t0, 0x1810(a0)
        jal stat
        nop
        lui t0, 0x012C
        sw t0, 0x1810(a0)
        jal stat
        nop
        lui t0, 0x0136
        ori t0, t0, 0x03E8
        sw t0, 0x1810(a0)
        jal stat
        nop
        jal short
        nop
        jal stat
        nop
        jal wait
        nop
        jal stat
        nop
# a flat rectangle 256x256 at 512,0
        lui t0, 0x6000
        ori t0, t0, 0xFF00
        sw t0, 0x1810(a0)
        jal stat
        nop
        ori t0, zero, 0x0200
        sw t0, 0x1810(a0)
        jal stat
        nop
        lui t0, 0x0100
        ori t0, t0, 0x0100
        sw t0, 0x1810(a0)
        jal stat
        nop
        jal short
        nop
        jal stat
        nop
        jal wait
        nop
        jal stat
        nop
# the triangle again, a second triangle queued behind it
        lui t0, 0x2000
        ori t0, t0, 0x0FF0
        sw t0, 0x1810(a0)
        sw zero, 0x1810(a0)
        ori t0, zero, 0x0200
        sw t0, 0x1810(a0)
        lui t0, 0x0100
        sw t0, 0x1810(a0)
        lui t0, 0x2000
        ori t0, t0, 0x00F0
        sw t0, 0x1810(a0)
        sw zero, 0x1810(a0)
        ori t0, zero, 0x0200
        sw t0, 0x1810(a0)
        lui t0, 0x0100
        sw t0, 0x1810(a0)
        jal stat
        nop
        jal short
        nop
        jal stat
        nop
        jal wait
        nop
        jal stat
        nop
        lui t1, 0xD1A0
        ori t1, t1, 0xD00E
        lui t0, 0x8010
        sw t1, -16(t0)
done:   j done
        nop
# GPUSTAT to s0, then on
stat:   lw t0, 0x1814(a0)
        addiu s0, s0, 4
        jr ra
        sw t0, -4(s0)
# about 300 cycles
short:  addiu t0, zero, 100
s1:     addiu t0, t0, -1
        bne t0, zero, s1
        nop
        jr ra
        nop
# about 400,000 cycles
wait:   lui t0, 0x0002
w1:     addiu t0, t0, -1
        bne t0, zero, w1
        nop
        jr ra
        nop
