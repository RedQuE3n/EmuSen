# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ShaderSettingsSliders-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 64,24,1152,668
title Shaders
footer "Drawn over the picture as it is shown, per console. A choice and every slider are saved at once and reach a running game of that console while its shader is in use. RetroArch: 5 presets, pack built 2026-09-22 02:00 UTC. The presets are in <run>/Home/Shaders/RetroArch."
case ShaderSettingsSliders
palette #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=64,124,1152,465 by=panel limit=64,124,1152,465 paints=#9C9CA0FF role=none
    1 DockPanel at=88,136,1104,441 by=panel limit=64,124,1152,465 role=none
      2 Grid at=88,136,1104,441 by=panel limit=64,124,1152,465 role=none
        3 Tabs at=88,136,1104,441 by=panel limit=64,124,1152,465 name=ConsoleTabs paints=#9C9CA0FF role=tab
          4 TabItem at=88,136,65,48 by=clip drawn focus heard=NES limit=88,136,1104,441 paints=#86868AFF role=tabitem sideways text=NES
            5 TextBlock at=100,142,41,36 box=41,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=NES layout=40.14,36 limit=88,136,65,48 owner=4 paints=#86868AFF role=text size=30 text=NES
          6 TabItem at=153,136,51,48 by=clip drawn focus heard=GB limit=88,136,1104,441 paints=#86868AFF role=tabitem sideways text=GB
            7 TextBlock at=165,142,27,36 box=27,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GB layout=27,36 limit=153,136,51,48 owner=6 paints=#86868AFF role=text size=30 text=GB
          8 TabItem at=204,136,77,48 by=clip drawn focus heard=SNES limit=88,136,1104,441 op paints=#EEEEF0FF role=tabitem sideways text=SNES
            9 TextBlock at=216,142,53,36 box=53,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=SNES layout=52.41,36 limit=204,136,77,48 owner=8 paints=#EEEEF0FF role=text size=30 text=SNES
          10 TabItem at=281,136,64,48 by=clip drawn focus heard=N64 limit=88,136,1104,441 paints=#86868AFF role=tabitem sideways text=N64
            11 TextBlock at=293,142,40,36 box=40,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=N64 layout=39.54,36 limit=281,136,64,48 owner=10 paints=#86868AFF role=text size=30 text=N64
          12 TabItem at=345,136,110,48 by=clip drawn focus heard=GENESIS limit=88,136,1104,441 paints=#86868AFF role=tabitem sideways text=Genesis
            13 TextBlock at=357,142,86,36 box=86,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GENESIS layout=85.47,36 limit=345,136,110,48 owner=12 paints=#86868AFF role=text size=30 text=GENESIS
          14 TabItem at=455,136,156,48 by=clip drawn focus heard=PLAYSTATION limit=88,136,1104,441 paints=#86868AFF role=tabitem sideways text=PlayStation
            15 TextBlock at=467,142,132,36 box=132,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=PLAYSTATION layout=131.91,36 limit=455,136,156,48 owner=14 paints=#86868AFF role=text size=30 text=PLAYSTATION
          16 ShaderPanel at=100,194,1080,383 by=clip limit=88,186,1104,391 role=none
            17 Grid at=100,194,528,383 by=clip limit=88,186,1104,391 role=none
              18 FilterBar at=100,194,528,46 by=clip limit=88,186,1104,391 name=SNES.ShaderSearch paints=#9C9CA0FF role=group
                19 TextBlock at=100,202,73,29 box=73,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Category layout=72.82,28.8 limit=100,194,528,46 paints=#86868AFF part role=text size=24 text=Category
                20 Dropdown at=181,194,180,44 drawn focus heard=Category name=PART_Facet op paints=#0E0E10FF,#55555AFF,#9C9CA0FF part role=combobox sideways
                  21 TextBlock at=193,199,136,32 box=136,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=All layout=20.07,31.2 limit=193,199,136,32 owner=20 paints=#9C9CA0FF role=text size=26 text=All
                  22 ViewboxContainer at=339,212.73,12,6.54 by=clip limit=339,212.73,12,6.54 nohit role=none
                23 TextBox at=369,194,259,46 drawn focus heard=Search name=PART_Search op paints=#0E0E10FF,#55555AFF,#D2D2D6FF part role=edit
                  24 ScrollViewer allow="a text box scrolls its own text" at=380,201,237,32 name=PART_ScrollViewer paints=#D2D2D6FF part role=scrollviewer
                    25 TextBlock allow="a text box scrolls its own text" at=380,201,237,32 box=237,32 caps=18.1987 drawn face="Barlow Condensed" heard=Search in=24 layout=61,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text=Search
              26 GroupedList at=100,248,528,275 by=clip limit=88,186,1104,391 name=SNES.ShaderList paints=#1B1B1EFF,#FFFFFFFF role=list
                27 ScrollViewer at=100,248,528,275 fades=tb name=PART_ScrollViewer overflow=367 paints=#1E1E1EFF,#55555AFF,#FFFFFFFF part role=scrollviewer viewport=100,248,512,275
                  28 ListBoxItem at=100,206,512,44 by=scroll drawn focus heard="CRT (Lottes)" in=27 limit=100,248,512,275 op paints=#333337FF,#9C9CA0FF role=listitem seen=100,248,512,2
                    29 GroupedListRow at=100,206,512,43 by=clip in=27 limit=100,206,512,44 paints=#9C9CA0FF role=none seen=100,248,512,1
                  30 ListBoxItem at=100,250,512,44 by=scroll drawn focus heard=Scanlines in=27 limit=100,248,512,275 op paints=#333337FF,#9C9CA0FF role=listitem
                    31 GroupedListRow at=100,250,512,43 by=clip in=27 limit=100,250,512,44 role=none
                      32 Border at=104,251,504,41 by=clip in=27 limit=100,250,512,44 role=none
                        33 DockPanel at=114,257,484,29 by=clip in=27 limit=100,250,512,44 role=none
                          34 StackPanel at=114,257,484,29 by=clip in=27 limit=100,250,512,44 role=none
                            35 TextBlock at=114,257,484,29 box=484,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Scanlines in=27 layout=77.93,28.8 limit=100,250,512,44 owner=30 paints=#9C9CA0FF role=text size=24 text=Scanlines
                  36 ListBoxItem at=100,294,512,44 by=scroll drawn focus heard="Simple CRT" in=27 limit=100,248,512,275 op paints=#333337FF,#9C9CA0FF role=listitem
                    37 GroupedListRow at=100,294,512,43 by=clip in=27 limit=100,294,512,44 role=none
                      38 Border at=104,295,504,41 by=clip in=27 limit=100,294,512,44 role=none
                        39 DockPanel at=114,301,484,29 by=clip in=27 limit=100,294,512,44 role=none
                          40 StackPanel at=114,301,484,29 by=clip in=27 limit=100,294,512,44 role=none
                            41 TextBlock at=114,301,484,29 box=484,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Simple CRT" in=27 layout=91.2,28.8 limit=100,294,512,44 owner=36 paints=#9C9CA0FF role=text size=24 text="Simple CRT"
                  42 ListBoxItem at=100,338,512,86 by=scroll drawn focus heard="bezel / Mega Bezel / Presets / Base CRT Presets: MBZ 0 SMOOTH-ADV GDV" in=27 limit=100,248,512,275 op paints=#333337FF,#9C9CA0FF role=listitem
                    43 GroupedListRow at=100,338,512,85 by=clip in=27 limit=100,338,512,86 role=none
                      44 DockPanel at=110,352,492,24 by=clip in=27 limit=100,338,512,86 role=none
                        45 TextBlock at=596,352,6,24 box=6,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=1 in=27 layout=5.12,24 limit=100,338,512,86 owner=42 paints=#86868AFF role=text size=20 text=1
                        46 TextBlock at=110,352,478,24 box=478,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="BEZEL / MEGA BEZEL / PRESETS / BASE CRT PRESETS" in=27 layout=354.82,24 limit=100,338,512,86 owner=42 paints=#86868AFF role=text size=20 text="BEZEL / MEGA BEZEL / PRESETS / BASE CRT PRESETS"
                      47 Border at=104,381,504,41 by=clip in=27 limit=100,338,512,86 role=none
                        48 DockPanel at=114,387,484,29 by=clip in=27 limit=100,338,512,86 role=none
                          49 StackPanel at=114,387,484,29 by=clip in=27 limit=100,338,512,86 role=none
                            50 TextBlock at=114,387,484,29 box=484,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="MBZ 0 SMOOTH-ADV GDV" in=27 layout=194.47,28.8 limit=100,338,512,86 owner=42 paints=#9C9CA0FF role=text size=24 text="MBZ 0 SMOOTH-ADV GDV"
                  51 ListBoxItem at=100,424,512,86 by=scroll drawn focus heard="big: huge" in=27 limit=100,248,512,275 op paints=#333337FF,#9C9CA0FF role=listitem
                    52 GroupedListRow at=100,424,512,85 by=clip in=27 limit=100,424,512,86 role=none
                      53 DockPanel at=110,438,492,24 by=clip in=27 limit=100,424,512,86 role=none
                        54 TextBlock at=596,438,6,24 box=6,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=1 in=27 layout=5.12,24 limit=100,424,512,86 owner=51 paints=#86868AFF role=text size=20 text=1
                        55 TextBlock at=110,438,478,24 box=478,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=BIG in=27 layout=22.3,24 limit=100,424,512,86 owner=51 paints=#86868AFF role=text size=20 text=BIG
                      56 Border at=104,467,504,41 by=clip in=27 limit=100,424,512,86 paints=#4C9AE8FF role=none
                        57 DockPanel at=114,473,484,29 by=clip in=27 limit=100,424,512,86 role=none
                          58 StackPanel at=114,473,484,29 by=clip in=27 limit=100,424,512,86 role=none
                            59 TextBlock at=114,473,484,29 box=484,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=huge in=27 layout=40.03,28.8 limit=100,424,512,86 owner=51 paints=#FFFFFFFF role=text size=24 text=huge
                  60 ListBoxItem at=100,510,512,86 by=scroll drawn focus heard="crt: crt-lottes" in=27 limit=100,248,512,275 op paints=#333337FF,#9C9CA0FF role=listitem seen=100,510,512,13
                    61 GroupedListRow at=100,510,512,85 by=clip in=27 limit=100,510,512,86 paints=#86868AFF,#9C9CA0FF role=none seen=100,510,512,13
                  62 RepeatButton at=612,248,16,16 drawn heard="Line up" name=PART_LineUpButton paints=#FFFFFFFF part role=button
                    63 ViewboxContainer at=616,254,8,4.4 by=clip limit=616,254,8,4.4 role=none
                  64 RepeatButton at=612,264,16,66 drawn heard="Page up" name=PART_PageUpButton part role=button
                  65 RepeatButton at=612,433,16,74 drawn heard="Page down" name=PART_PageDownButton part role=button
                  66 RepeatButton at=612,507,16,16 drawn heard="Line down" name=PART_LineDownButton paints=#FFFFFFFF part role=button
                    67 ViewboxContainer at=616,513,8,4.4 by=clip limit=616,513,8,4.4 role=none
              68 Border at=100,533,528,44 by=clip limit=88,186,1104,391 role=none
                69 Button at=100,533,151,44 by=clip drawn focus heard="UPDATE PACK" limit=88,186,1104,391 name=SNES.DownloadPack op paints=#55555AFF,#9C9CA0FF role=button text="Update Pack"
                  70 TextBlock at=117,539,117,32 box=117,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="UPDATE PACK" layout=116.04,31.2 limit=100,533,151,44 owner=69 paints=#9C9CA0FF role=text size=26 text="UPDATE PACK"
            71 DockPanel at=652,194,528,383 by=clip limit=88,186,1104,391 role=none
              72 StackPanel at=652,194,528,159 by=clip limit=88,186,1104,391 role=none
                73 Grid at=652,194,528,44 by=clip limit=88,186,1104,391 role=none
                  74 TextBlock at=652,194,195,44 box=195,44 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=huge layout=40.03,28.8 limit=88,186,1104,391 name=SNES.ShaderName paints=#9C9CA0FF role=text size=24 text=huge
                  75 StackPanel at=859,194,321,44 by=clip limit=88,186,1104,391 role=none
                    76 Button at=859,194,126,44 by=clip drawn heard="RESET ALL" limit=88,186,1104,391 name=SNES.ResetShader off paints=#55555AFF,#9C9CA0FF role=button text="Reset All"
                      77 TextBlock at=876,200,92,32 box=92,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="RESET ALL" layout=91.91,31.2 limit=859,194,126,44 off owner=76 paints=#55555AFF role=text size=26 text="RESET ALL"
                    78 Button at=993,194,187,44 by=clip drawn focus heard="USE THIS SHADER" limit=88,186,1104,391 name=SNES.UseShader op paints=#55555AFF,#9C9CA0FF role=button text="Use This Shader"
                      79 TextBlock at=1010,200,153,32 box=153,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="USE THIS SHADER" layout=152.57,31.2 limit=993,194,187,44 owner=78 paints=#9C9CA0FF role=text size=26 text="USE THIS SHADER"
                80 MonoText at=652,244,528,25 box=528,25 by=clip caps=12.854 drawn face="Noto Sans" heard=big/huge.slangp layout=138.4,24.52 limit=88,186,1104,391 name=SNES.ShaderPath paints=#9C9CA0FF role=text size=18 text=big/huge.slangp
                81 HintText at=652,275,528,24 box=528,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="944 parameters." layout=111.16,24 limit=88,186,1104,391 paints=#86868AFF role=text size=20 text="944 parameters."
                82 FilterBar at=652,307,514,46 by=clip limit=88,186,1104,391 name=SNES.ParameterSearch paints=#9C9CA0FF role=group
                  83 TextBox at=652,307,514,46 drawn focus heard="Search parameters" name=PART_Search op paints=#0E0E10FF,#55555AFF,#D2D2D6FF part role=edit
                    84 ScrollViewer allow="a text box scrolls its own text" at=663,314,492,32 name=PART_ScrollViewer paints=#D2D2D6FF part role=scrollviewer
                      85 TextBlock allow="a text box scrolls its own text" at=663,314,492,32 box=492,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search parameters" in=84 layout=168.51,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text="Search parameters"
              86 SliderList at=652,361,528,216 by=clip limit=88,186,1104,391 name=SNES.ShaderParameters paints=#9C9CA0FF role=list
                87 ScrollViewer at=652,361,528,216 fades=b name=PART_ScrollViewer overflow=78407 paints=#1E1E1EFF,#333337FF,#4C9AE8FF,#55555AFF,#9C9CA0FF,#EEEEF0FF part role=scrollviewer viewport=652,361,512,216
                  88 ContentPresenter at=652,361,498,48 by=scroll in=87 limit=652,361,512,216 role=none
                    89 SectionHeader at=652,371,498,34 box=498,34 by=scroll caps=19.5986 drawn face="Barlow Condensed" heard="--- Section 0 ---" in=87 layout=160.3,33.6 limit=652,361,512,216 paints=#B4B4B8FF role=text size=28 text="--- Section 0 ---"
                  90 ContentPresenter at=652,409,498,94 by=scroll in=87 limit=652,361,512,216 role=none
                    91 SliderRow at=652,409,498,90 by=scroll in=87 limit=652,361,512,216 name=SNES.Parameter.P0001 role=none
                      92 StackPanel at=652,413,498,82 by=scroll in=87 limit=652,361,512,216 role=none
                        93 DockPanel at=652,413,498,30 by=scroll in=87 limit=652,361,512,216 role=none
                          94 Button at=1085,413,65,30 by=scroll drawn focus heard="Reset Parameter 0001" in=87 limit=652,361,512,216 op paints=#55555AFF,#9C9CA0FF role=button text=Reset
                            95 TextBlock at=1096,416,43,24 box=43,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=RESET in=87 layout=42.66,24 limit=1085,413,65,30 owner=94 paints=#9C9CA0FF role=text size=20 text=RESET
                          96 TextBlock at=1025,413,48,29 box=48,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard=0.50 in=87 layout=35.69,28.8 limit=652,361,512,216 paints=#9C9CA0FF role=text size=24 text=0.50
                          97 TextBlock at=652,413,361,29 box=361,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Parameter 0001" in=87 layout=128.71,28.8 limit=652,361,512,216 paints=#9C9CA0FF role=text size=24 text="Parameter 0001"
                        98 Slider at=652,445,498,50 by=scroll drawn focus heard="Parameter 0001" in=87 limit=652,361,512,216 op paints=#333337FF,#4C9AE8FF,#EEEEF0FF role=slider sideways
                          99 RepeatButton at=652,460,239,20 drawn in=87 name=PART_DecreaseButton paints=#4C9AE8FF part role=button
                          100 RepeatButton at=911,460,239,20 drawn in=87 name=PART_IncreaseButton paints=#333337FF,#4C9AE8FF part role=button
                  101 ContentPresenter at=652,503,498,94 by=scroll in=87 limit=652,361,512,216 role=none seen=652,503,498,74
                    102 SliderRow at=652,503,498,90 by=scroll in=87 limit=652,361,512,216 name=SNES.Parameter.P0002 role=none seen=652,503,498,74
                      103 StackPanel at=652,507,498,82 by=scroll in=87 limit=652,361,512,216 role=none seen=652,507,498,70
                        104 DockPanel at=652,507,498,30 by=scroll in=87 limit=652,361,512,216 role=none
                          105 Button at=1085,507,65,30 by=scroll drawn focus heard="Reset Parameter 0002" in=87 limit=652,361,512,216 op paints=#55555AFF,#9C9CA0FF role=button text=Reset
                            106 TextBlock at=1096,510,43,24 box=43,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=RESET in=87 layout=42.66,24 limit=1085,507,65,30 owner=105 paints=#9C9CA0FF role=text size=20 text=RESET
                          107 TextBlock at=1025,507,48,29 box=48,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard=0.50 in=87 layout=35.69,28.8 limit=652,361,512,216 paints=#9C9CA0FF role=text size=24 text=0.50
                          108 TextBlock at=652,507,361,29 box=361,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Parameter 0002" in=87 layout=132.22,28.8 limit=652,361,512,216 paints=#9C9CA0FF role=text size=24 text="Parameter 0002"
                        109 Slider at=652,539,498,50 by=scroll drawn focus heard="Parameter 0002" in=87 limit=652,361,512,216 op paints=#333337FF,#4C9AE8FF,#EEEEF0FF role=slider seen=652,539,498,38 sideways
                          110 RepeatButton at=652,554,239,20 drawn in=87 name=PART_DecreaseButton paints=#4C9AE8FF part role=button
                          111 RepeatButton at=911,554,239,20 drawn in=87 name=PART_IncreaseButton paints=#333337FF,#4C9AE8FF part role=button
                  112 RepeatButton at=1164,361,16,16 drawn heard="Line up" name=PART_LineUpButton paints=#FFFFFFFF part role=button
                    113 ViewboxContainer at=1168,367,8,4.4 by=clip limit=1168,367,8,4.4 role=none
                  114 RepeatButton at=1164,417,16,144 drawn heard="Page down" name=PART_PageDownButton part role=button
                  115 RepeatButton at=1164,561,16,16 drawn heard="Line down" name=PART_LineDownButton paints=#FFFFFFFF part role=button
                    116 ViewboxContainer at=1168,567,8,4.4 by=clip limit=1168,567,8,4.4 role=none
        117 ButtonBar at=1102,136,90,44 by=panel limit=64,124,1152,465 paints=#9C9CA0FF role=toolbar
          118 Button at=1102,136,90,44 by=clip drawn focus heard=CLOSE limit=1102,136,90,44 op paints=#55555AFF,#9C9CA0FF role=button text=Close
            119 TextBlock at=1119,142,56,32 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=55.51,31.2 limit=1102,136,90,44 owner=118 paints=#9C9CA0FF role=text size=26 text=CLOSE
focus 51
edge 8 up same
edge 8 down 20
edge 20 up 8
edge 20 down 30
edge 23 up 8
edge 23 down 30
edge 23 left 20
edge 23 right 78
edge 28 up row:26/1
edge 28 down 30
edge 28 left same
edge 28 right 83
edge 30 up 28
edge 30 down 36
edge 30 left same
edge 30 right 83
edge 36 up 30
edge 36 down 42
edge 36 left same
edge 36 right 83
edge 42 up 36
edge 42 down 51
edge 42 left same
edge 42 right 83
edge 51 up 42
edge 51 down 60
edge 51 left same
edge 51 right 98
edge 60 up 51
edge 60 down row:26/8
edge 60 left same
edge 60 right 83
edge 69 up row:26/9
edge 69 down same
edge 69 left same
edge 69 right "item:86/2/0/0/1/Scanline weight"
edge 78 up 118
edge 78 down 83
edge 78 left 23
edge 78 right same
edge 83 up 78
edge 83 down "item:86/1/0/0/0/0/Reset Glow strength"
edge 83 left 42
edge 83 right same
edge 94 up 83
edge 94 down 98
edge 94 left 42
edge 94 right same
edge 98 up 94
edge 98 down 105
edge 105 up 98
edge 105 down 109
edge 105 left 60
edge 105 right same
edge 109 up 105
edge 109 down "item:86/3/0/0/0/0/Reset Parameter 0003"
edge 118 up same
edge 118 down 78
edge 118 left 8
edge 118 right same
edge "item:86/1/0/0/0/0/Reset Glow strength" up 83
edge "item:86/1/0/0/0/0/Reset Glow strength" down "item:86/1/0/0/1/Glow strength"
edge "item:86/1/0/0/0/0/Reset Glow strength" left 51
edge "item:86/1/0/0/0/0/Reset Glow strength" right same
edge "item:86/1/0/0/1/Glow strength" up "item:86/1/0/0/0/0/Reset Glow strength"
edge "item:86/1/0/0/1/Glow strength" down "item:86/2/0/0/0/0/Reset Scanline weight"
edge "item:86/3/0/0/0/0/Reset Parameter 0003" up 109
edge "item:86/3/0/0/0/0/Reset Parameter 0003" down "item:86/3/0/0/1/Parameter 0003"
edge "item:86/3/0/0/0/0/Reset Parameter 0003" left 60
edge "item:86/3/0/0/0/0/Reset Parameter 0003" right same
edge row:26/1 up row:26/0
edge row:26/1 down 28
edge row:26/1 left same
edge row:26/1 right 83
edge row:26/8 up 60
edge row:26/8 down row:26/9
edge row:26/8 left same
edge row:26/8 right 83
edge row:26/9 up row:26/8
edge row:26/9 down 69
edge row:26/9 left same
edge row:26/9 right 83
press pageup focus="Dropdown PART_Facet (Category)" tab=GB title=Shaders top=ShaderSettingsWindow
press pagedown focus="Dropdown PART_Facet (Category)" tab=SNES title=Shaders top=ShaderSettingsWindow
press first focus="Dropdown PART_Facet (Category)" tab=SNES title=Shaders top=ShaderSettingsWindow
press last focus="Dropdown PART_Facet (Category)" tab=SNES title=Shaders top=ShaderSettingsWindow
press back top=none
