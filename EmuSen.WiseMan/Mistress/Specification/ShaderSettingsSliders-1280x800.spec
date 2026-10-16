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
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
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
          14 TabItem at=455,136,109,48 by=clip drawn focus heard="SEGA CD" limit=88,136,1104,441 paints=#86868AFF role=tabitem sideways text="Sega CD"
            15 TextBlock at=467,142,85,36 box=85,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard="SEGA CD" layout=84.18,36 limit=455,136,109,48 owner=14 paints=#86868AFF role=text size=30 text="SEGA CD"
          16 TabItem at=564,136,156,48 by=clip drawn focus heard=PLAYSTATION limit=88,136,1104,441 paints=#86868AFF role=tabitem sideways text=PlayStation
            17 TextBlock at=576,142,132,36 box=132,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=PLAYSTATION layout=131.91,36 limit=564,136,156,48 owner=16 paints=#86868AFF role=text size=30 text=PLAYSTATION
          18 ShaderPanel at=100,194,1080,383 by=clip limit=88,186,1104,391 role=none
            19 Grid at=100,194,528,383 by=clip limit=88,186,1104,391 role=none
              20 FilterBar at=100,194,528,46 by=clip limit=88,186,1104,391 name=SNES.ShaderSearch paints=#9C9CA0FF role=group
                21 TextBlock at=100,202,73,29 box=73,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Category layout=72.82,28.8 limit=100,194,528,46 paints=#86868AFF part role=text size=24 text=Category
                22 Dropdown at=181,194,180,44 drawn focus heard=Category name=PART_Facet op paints=#0E0E10FF,#55555AFF,#9C9CA0FF part role=combobox sideways
                  23 TextBlock at=193,199,136,32 box=136,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=All layout=20.07,31.2 limit=193,199,136,32 owner=22 paints=#9C9CA0FF role=text size=26 text=All
                  24 ViewboxContainer at=339,212.73,12,6.54 by=clip limit=339,212.73,12,6.54 nohit role=none
                25 TextBox at=369,194,259,46 drawn focus heard=Search name=PART_Search op paints=#0E0E10FF,#55555AFF,#D2D2D6FF part role=edit
                  26 ScrollViewer allow="a text box scrolls its own text" at=380,201,237,32 name=PART_ScrollViewer paints=#D2D2D6FF part role=scrollviewer
                    27 TextBlock allow="a text box scrolls its own text" at=380,201,237,32 box=237,32 caps=18.1987 drawn face="Barlow Condensed" heard=Search in=26 layout=61,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text=Search
              28 GroupedList at=100,248,528,275 by=clip limit=88,186,1104,391 name=SNES.ShaderList paints=#1B1B1EFF,#FFFFFFFF role=list
                29 ScrollViewer at=100,248,528,275 fades=tb name=PART_ScrollViewer overflow=367 paints=#1E1E1EFF,#55555AFF,#FFFFFFFF part role=scrollviewer viewport=100,248,512,275
                  30 ListBoxItem at=100,206,512,44 by=scroll drawn focus heard="CRT (Lottes)" in=29 limit=100,248,512,275 op paints=#333337FF,#9C9CA0FF role=listitem seen=100,248,512,2
                    31 GroupedListRow at=100,206,512,43 by=clip in=29 limit=100,206,512,44 paints=#9C9CA0FF role=none seen=100,248,512,1
                  32 ListBoxItem at=100,250,512,44 by=scroll drawn focus heard=Scanlines in=29 limit=100,248,512,275 op paints=#333337FF,#9C9CA0FF role=listitem
                    33 GroupedListRow at=100,250,512,43 by=clip in=29 limit=100,250,512,44 role=none
                      34 Border at=104,251,504,41 by=clip in=29 limit=100,250,512,44 role=none
                        35 DockPanel at=114,257,484,29 by=clip in=29 limit=100,250,512,44 role=none
                          36 StackPanel at=114,257,484,29 by=clip in=29 limit=100,250,512,44 role=none
                            37 TextBlock at=114,257,484,29 box=484,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Scanlines in=29 layout=77.93,28.8 limit=100,250,512,44 owner=32 paints=#9C9CA0FF role=text size=24 text=Scanlines
                  38 ListBoxItem at=100,294,512,44 by=scroll drawn focus heard="Simple CRT" in=29 limit=100,248,512,275 op paints=#333337FF,#9C9CA0FF role=listitem
                    39 GroupedListRow at=100,294,512,43 by=clip in=29 limit=100,294,512,44 role=none
                      40 Border at=104,295,504,41 by=clip in=29 limit=100,294,512,44 role=none
                        41 DockPanel at=114,301,484,29 by=clip in=29 limit=100,294,512,44 role=none
                          42 StackPanel at=114,301,484,29 by=clip in=29 limit=100,294,512,44 role=none
                            43 TextBlock at=114,301,484,29 box=484,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Simple CRT" in=29 layout=91.2,28.8 limit=100,294,512,44 owner=38 paints=#9C9CA0FF role=text size=24 text="Simple CRT"
                  44 ListBoxItem at=100,338,512,86 by=scroll drawn focus heard="bezel / Mega Bezel / Presets / Base CRT Presets: MBZ 0 SMOOTH-ADV GDV" in=29 limit=100,248,512,275 op paints=#333337FF,#9C9CA0FF role=listitem
                    45 GroupedListRow at=100,338,512,85 by=clip in=29 limit=100,338,512,86 role=none
                      46 DockPanel at=110,352,492,24 by=clip in=29 limit=100,338,512,86 role=none
                        47 TextBlock at=596,352,6,24 box=6,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=1 in=29 layout=5.12,24 limit=100,338,512,86 owner=44 paints=#86868AFF role=text size=20 text=1
                        48 TextBlock at=110,352,478,24 box=478,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="BEZEL / MEGA BEZEL / PRESETS / BASE CRT PRESETS" in=29 layout=354.82,24 limit=100,338,512,86 owner=44 paints=#86868AFF role=text size=20 text="BEZEL / MEGA BEZEL / PRESETS / BASE CRT PRESETS"
                      49 Border at=104,381,504,41 by=clip in=29 limit=100,338,512,86 role=none
                        50 DockPanel at=114,387,484,29 by=clip in=29 limit=100,338,512,86 role=none
                          51 StackPanel at=114,387,484,29 by=clip in=29 limit=100,338,512,86 role=none
                            52 TextBlock at=114,387,484,29 box=484,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="MBZ 0 SMOOTH-ADV GDV" in=29 layout=194.47,28.8 limit=100,338,512,86 owner=44 paints=#9C9CA0FF role=text size=24 text="MBZ 0 SMOOTH-ADV GDV"
                  53 ListBoxItem at=100,424,512,86 by=scroll drawn focus heard="big: huge" in=29 limit=100,248,512,275 op paints=#333337FF,#9C9CA0FF role=listitem
                    54 GroupedListRow at=100,424,512,85 by=clip in=29 limit=100,424,512,86 role=none
                      55 DockPanel at=110,438,492,24 by=clip in=29 limit=100,424,512,86 role=none
                        56 TextBlock at=596,438,6,24 box=6,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=1 in=29 layout=5.12,24 limit=100,424,512,86 owner=53 paints=#86868AFF role=text size=20 text=1
                        57 TextBlock at=110,438,478,24 box=478,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=BIG in=29 layout=22.3,24 limit=100,424,512,86 owner=53 paints=#86868AFF role=text size=20 text=BIG
                      58 Border at=104,467,504,41 by=clip in=29 limit=100,424,512,86 paints=#4C9AE8FF role=none
                        59 DockPanel at=114,473,484,29 by=clip in=29 limit=100,424,512,86 role=none
                          60 StackPanel at=114,473,484,29 by=clip in=29 limit=100,424,512,86 role=none
                            61 TextBlock at=114,473,484,29 box=484,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=huge in=29 layout=40.03,28.8 limit=100,424,512,86 owner=53 paints=#FFFFFFFF role=text size=24 text=huge
                  62 ListBoxItem at=100,510,512,86 by=scroll drawn focus heard="crt: crt-lottes" in=29 limit=100,248,512,275 op paints=#333337FF,#9C9CA0FF role=listitem seen=100,510,512,13
                    63 GroupedListRow at=100,510,512,85 by=clip in=29 limit=100,510,512,86 paints=#86868AFF,#9C9CA0FF role=none seen=100,510,512,13
                  64 RepeatButton at=612,248,16,16 drawn heard="Line up" name=PART_LineUpButton paints=#FFFFFFFF part role=button
                    65 ViewboxContainer at=616,254,8,4.4 by=clip limit=616,254,8,4.4 role=none
                  66 RepeatButton at=612,264,16,66 drawn heard="Page up" name=PART_PageUpButton part role=button
                  67 RepeatButton at=612,433,16,74 drawn heard="Page down" name=PART_PageDownButton part role=button
                  68 RepeatButton at=612,507,16,16 drawn heard="Line down" name=PART_LineDownButton paints=#FFFFFFFF part role=button
                    69 ViewboxContainer at=616,513,8,4.4 by=clip limit=616,513,8,4.4 role=none
              70 Border at=100,533,528,44 by=clip limit=88,186,1104,391 role=none
                71 Button at=100,533,151,44 by=clip drawn focus heard="UPDATE PACK" limit=88,186,1104,391 name=SNES.DownloadPack op paints=#55555AFF,#9C9CA0FF role=button text="Update Pack"
                  72 TextBlock at=117,539,117,32 box=117,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="UPDATE PACK" layout=116.04,31.2 limit=100,533,151,44 owner=71 paints=#9C9CA0FF role=text size=26 text="UPDATE PACK"
            73 DockPanel at=652,194,528,383 by=clip limit=88,186,1104,391 role=none
              74 StackPanel at=652,194,528,159 by=clip limit=88,186,1104,391 role=none
                75 Grid at=652,194,528,44 by=clip limit=88,186,1104,391 role=none
                  76 TextBlock at=652,194,195,44 box=195,44 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=huge layout=40.03,28.8 limit=88,186,1104,391 name=SNES.ShaderName paints=#9C9CA0FF role=text size=24 text=huge
                  77 StackPanel at=859,194,321,44 by=clip limit=88,186,1104,391 role=none
                    78 Button at=859,194,126,44 by=clip drawn heard="RESET ALL" limit=88,186,1104,391 name=SNES.ResetShader off paints=#55555AFF,#9C9CA0FF role=button text="Reset All"
                      79 TextBlock at=876,200,92,32 box=92,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="RESET ALL" layout=91.91,31.2 limit=859,194,126,44 off owner=78 paints=#55555AFF role=text size=26 text="RESET ALL"
                    80 Button at=993,194,187,44 by=clip drawn focus heard="USE THIS SHADER" limit=88,186,1104,391 name=SNES.UseShader op paints=#55555AFF,#9C9CA0FF role=button text="Use This Shader"
                      81 TextBlock at=1010,200,153,32 box=153,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="USE THIS SHADER" layout=152.57,31.2 limit=993,194,187,44 owner=80 paints=#9C9CA0FF role=text size=26 text="USE THIS SHADER"
                82 MonoText at=652,244,528,25 box=528,25 by=clip caps=12.854 drawn face="Noto Sans" heard=big/huge.slangp layout=138.4,24.52 limit=88,186,1104,391 name=SNES.ShaderPath paints=#9C9CA0FF role=text size=18 text=big/huge.slangp
                83 HintText at=652,275,528,24 box=528,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="944 parameters." layout=111.16,24 limit=88,186,1104,391 paints=#86868AFF role=text size=20 text="944 parameters."
                84 FilterBar at=652,307,514,46 by=clip limit=88,186,1104,391 name=SNES.ParameterSearch paints=#9C9CA0FF role=group
                  85 TextBox at=652,307,514,46 drawn focus heard="Search parameters" name=PART_Search op paints=#0E0E10FF,#55555AFF,#D2D2D6FF part role=edit
                    86 ScrollViewer allow="a text box scrolls its own text" at=663,314,492,32 name=PART_ScrollViewer paints=#D2D2D6FF part role=scrollviewer
                      87 TextBlock allow="a text box scrolls its own text" at=663,314,492,32 box=492,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search parameters" in=86 layout=168.51,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text="Search parameters"
              88 SliderList at=652,361,528,216 by=clip limit=88,186,1104,391 name=SNES.ShaderParameters paints=#9C9CA0FF role=list
                89 ScrollViewer at=652,361,528,216 fades=b name=PART_ScrollViewer overflow=78407 paints=#1E1E1EFF,#333337FF,#4C9AE8FF,#55555AFF,#9C9CA0FF,#EEEEF0FF part role=scrollviewer viewport=652,361,512,216
                  90 ContentPresenter at=652,361,498,48 by=scroll in=89 limit=652,361,512,216 role=none
                    91 SectionHeader at=652,371,498,34 box=498,34 by=scroll caps=19.5986 drawn face="Barlow Condensed" heard="--- Section 0 ---" in=89 layout=160.3,33.6 limit=652,361,512,216 paints=#B4B4B8FF role=text size=28 text="--- Section 0 ---"
                  92 ContentPresenter at=652,409,498,94 by=scroll in=89 limit=652,361,512,216 role=none
                    93 SliderRow at=652,409,498,90 by=scroll in=89 limit=652,361,512,216 name=SNES.Parameter.P0001 role=none
                      94 StackPanel at=652,413,498,82 by=scroll in=89 limit=652,361,512,216 role=none
                        95 DockPanel at=652,413,498,30 by=scroll in=89 limit=652,361,512,216 role=none
                          96 Button at=1085,413,65,30 by=scroll drawn focus heard="Reset Parameter 0001" in=89 limit=652,361,512,216 op paints=#55555AFF,#9C9CA0FF role=button text=Reset
                            97 TextBlock at=1096,416,43,24 box=43,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=RESET in=89 layout=42.66,24 limit=1085,413,65,30 owner=96 paints=#9C9CA0FF role=text size=20 text=RESET
                          98 TextBlock at=1025,413,48,29 box=48,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard=0.50 in=89 layout=35.69,28.8 limit=652,361,512,216 paints=#9C9CA0FF role=text size=24 text=0.50
                          99 TextBlock at=652,413,361,29 box=361,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Parameter 0001" in=89 layout=128.71,28.8 limit=652,361,512,216 paints=#9C9CA0FF role=text size=24 text="Parameter 0001"
                        100 Slider at=652,445,498,50 by=scroll drawn focus heard="Parameter 0001" in=89 limit=652,361,512,216 op paints=#333337FF,#4C9AE8FF,#EEEEF0FF role=slider sideways
                          101 RepeatButton at=652,460,239,20 drawn in=89 name=PART_DecreaseButton paints=#4C9AE8FF part role=button
                          102 RepeatButton at=911,460,239,20 drawn in=89 name=PART_IncreaseButton paints=#333337FF,#4C9AE8FF part role=button
                  103 ContentPresenter at=652,503,498,94 by=scroll in=89 limit=652,361,512,216 role=none seen=652,503,498,74
                    104 SliderRow at=652,503,498,90 by=scroll in=89 limit=652,361,512,216 name=SNES.Parameter.P0002 role=none seen=652,503,498,74
                      105 StackPanel at=652,507,498,82 by=scroll in=89 limit=652,361,512,216 role=none seen=652,507,498,70
                        106 DockPanel at=652,507,498,30 by=scroll in=89 limit=652,361,512,216 role=none
                          107 Button at=1085,507,65,30 by=scroll drawn focus heard="Reset Parameter 0002" in=89 limit=652,361,512,216 op paints=#55555AFF,#9C9CA0FF role=button text=Reset
                            108 TextBlock at=1096,510,43,24 box=43,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=RESET in=89 layout=42.66,24 limit=1085,507,65,30 owner=107 paints=#9C9CA0FF role=text size=20 text=RESET
                          109 TextBlock at=1025,507,48,29 box=48,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard=0.50 in=89 layout=35.69,28.8 limit=652,361,512,216 paints=#9C9CA0FF role=text size=24 text=0.50
                          110 TextBlock at=652,507,361,29 box=361,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Parameter 0002" in=89 layout=132.22,28.8 limit=652,361,512,216 paints=#9C9CA0FF role=text size=24 text="Parameter 0002"
                        111 Slider at=652,539,498,50 by=scroll drawn focus heard="Parameter 0002" in=89 limit=652,361,512,216 op paints=#333337FF,#4C9AE8FF,#EEEEF0FF role=slider seen=652,539,498,38 sideways
                          112 RepeatButton at=652,554,239,20 drawn in=89 name=PART_DecreaseButton paints=#4C9AE8FF part role=button
                          113 RepeatButton at=911,554,239,20 drawn in=89 name=PART_IncreaseButton paints=#333337FF,#4C9AE8FF part role=button
                  114 RepeatButton at=1164,361,16,16 drawn heard="Line up" name=PART_LineUpButton paints=#FFFFFFFF part role=button
                    115 ViewboxContainer at=1168,367,8,4.4 by=clip limit=1168,367,8,4.4 role=none
                  116 RepeatButton at=1164,417,16,144 drawn heard="Page down" name=PART_PageDownButton part role=button
                  117 RepeatButton at=1164,561,16,16 drawn heard="Line down" name=PART_LineDownButton paints=#FFFFFFFF part role=button
                    118 ViewboxContainer at=1168,567,8,4.4 by=clip limit=1168,567,8,4.4 role=none
        119 ButtonBar at=1102,136,90,44 by=panel limit=64,124,1152,465 paints=#9C9CA0FF role=toolbar
          120 Button at=1102,136,90,44 by=clip drawn focus heard=CLOSE limit=1102,136,90,44 op paints=#55555AFF,#9C9CA0FF role=button text=Close
            121 TextBlock at=1119,142,56,32 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=55.51,31.2 limit=1102,136,90,44 owner=120 paints=#9C9CA0FF role=text size=26 text=CLOSE
focus 53
edge 8 up same
edge 8 down 22
edge 22 up 8
edge 22 down 32
edge 25 up 8
edge 25 down 32
edge 25 left 22
edge 25 right 80
edge 30 up row:28/1/CRT
edge 30 down 32
edge 30 left same
edge 30 right 85
edge 32 up 30
edge 32 down 38
edge 32 left same
edge 32 right 85
edge 38 up 32
edge 38 down 44
edge 38 left same
edge 38 right 85
edge 44 up 38
edge 44 down 53
edge 44 left same
edge 44 right 85
edge 53 up 44
edge 53 down 62
edge 53 left same
edge 53 right 100
edge 62 up 53
edge 62 down row:28/8/crt-royale-kurozumi
edge 62 left same
edge 62 right 85
edge 71 up "row:28/9/handheld: lcd-grid-v2"
edge 71 down same
edge 71 left same
edge 71 right "item:88/2/0/0/1/Scanline weight"
edge 80 up 120
edge 80 down 85
edge 80 left 25
edge 80 right same
edge 85 up 80
edge 85 down "item:88/1/0/0/0/0/Reset Glow strength"
edge 85 left 44
edge 85 right same
edge 96 up 85
edge 96 down 100
edge 96 left 44
edge 96 right same
edge 100 up 96
edge 100 down 107
edge 107 up 100
edge 107 down 111
edge 107 left 62
edge 107 right same
edge 111 up 107
edge 111 down "item:88/3/0/0/0/0/Reset Parameter 0003"
edge 120 up same
edge 120 down 80
edge 120 left 8
edge 120 right same
edge "item:88/1/0/0/0/0/Reset Glow strength" up 85
edge "item:88/1/0/0/0/0/Reset Glow strength" down "item:88/1/0/0/1/Glow strength"
edge "item:88/1/0/0/0/0/Reset Glow strength" left 53
edge "item:88/1/0/0/0/0/Reset Glow strength" right same
edge "item:88/1/0/0/1/Glow strength" up "item:88/1/0/0/0/0/Reset Glow strength"
edge "item:88/1/0/0/1/Glow strength" down "item:88/2/0/0/0/0/Reset Scanline weight"
edge "item:88/3/0/0/0/0/Reset Parameter 0003" up 111
edge "item:88/3/0/0/0/0/Reset Parameter 0003" down "item:88/3/0/0/1/Parameter 0003"
edge "item:88/3/0/0/0/0/Reset Parameter 0003" left 62
edge "item:88/3/0/0/0/0/Reset Parameter 0003" right same
edge row:28/1/CRT up "row:28/0/Built into EmuSen: None"
edge row:28/1/CRT down 30
edge row:28/1/CRT left same
edge row:28/1/CRT right 85
edge row:28/8/crt-royale-kurozumi up 62
edge row:28/8/crt-royale-kurozumi down "row:28/9/handheld: lcd-grid-v2"
edge row:28/8/crt-royale-kurozumi left same
edge row:28/8/crt-royale-kurozumi right 85
edge "row:28/9/handheld: lcd-grid-v2" up row:28/8/crt-royale-kurozumi
edge "row:28/9/handheld: lcd-grid-v2" down 71
edge "row:28/9/handheld: lcd-grid-v2" left same
edge "row:28/9/handheld: lcd-grid-v2" right 85
press pageup focus="Dropdown PART_Facet (Category)" tab=GB title=Shaders top=ShaderSettingsWindow
press pagedown focus="Dropdown PART_Facet (Category)" tab=SNES title=Shaders top=ShaderSettingsWindow
press first focus="Dropdown PART_Facet (Category)" tab=SNES title=Shaders top=ShaderSettingsWindow
press last focus="Dropdown PART_Facet (Category)" tab=SNES title=Shaders top=ShaderSettingsWindow
press back top=none
