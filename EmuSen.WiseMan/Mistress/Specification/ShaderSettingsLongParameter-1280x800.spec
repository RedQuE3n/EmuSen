# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ShaderSettingsLongParameter-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 64,24,1152,668
title Shaders
footer "Drawn over the picture as it is shown, per console. A choice and every slider are saved at once and reach a running game of that console while its shader is in use. RetroArch: 5 presets, pack built 2026-09-22 02:00 UTC. The presets are in <run>/Home/Shaders/RetroArch."
case ShaderSettingsLongParameter
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
                29 ScrollViewer at=100,248,528,275 fades=t name=PART_ScrollViewer overflow=375 paints=#1E1E1EFF,#55555AFF,#FFFFFFFF part role=scrollviewer viewport=100,248,512,275
                  30 ListBoxItem at=100,221,512,86 by=scroll drawn focus heard="crt: crt-lottes" in=29 limit=100,248,512,275 op paints=#333337FF,#9C9CA0FF role=listitem seen=100,248,512,59
                    31 GroupedListRow at=100,221,512,85 by=clip in=29 limit=100,221,512,86 role=none seen=100,248,512,58
                      32 DockPanel at=110,235,492,24 by=clip in=29 limit=100,221,512,86 role=none seen=110,248,492,11
                        33 TextBlock at=593,235,9,24 box=9,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=2 in=29 layout=8.04,24 limit=100,221,512,86 owner=30 paints=#86868AFF role=text seen=593,248,9,11 size=20 text=2
                        34 TextBlock at=110,235,475,24 box=475,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=CRT in=29 layout=26.24,24 limit=100,221,512,86 owner=30 paints=#86868AFF role=text seen=110,248,475,11 size=20 text=CRT
                      35 Border at=104,264,504,41 by=clip in=29 limit=100,221,512,86 role=none
                        36 DockPanel at=114,270,484,29 by=clip in=29 limit=100,221,512,86 role=none
                          37 StackPanel at=114,270,484,29 by=clip in=29 limit=100,221,512,86 role=none
                            38 TextBlock at=114,270,484,29 box=484,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=crt-lottes in=29 layout=76.39,28.8 limit=100,221,512,86 owner=30 paints=#9C9CA0FF role=text size=24 text=crt-lottes
                  39 ListBoxItem at=100,307,512,44 by=scroll drawn focus heard=crt-royale-kurozumi in=29 limit=100,248,512,275 op paints=#333337FF,#9C9CA0FF role=listitem
                    40 GroupedListRow at=100,307,512,43 by=clip in=29 limit=100,307,512,44 role=none
                      41 Border at=104,308,504,41 by=clip in=29 limit=100,307,512,44 role=none
                        42 DockPanel at=114,314,484,29 by=clip in=29 limit=100,307,512,44 role=none
                          43 StackPanel at=114,314,484,29 by=clip in=29 limit=100,307,512,44 role=none
                            44 TextBlock at=114,314,484,29 box=484,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=crt-royale-kurozumi in=29 layout=163.49,28.8 limit=100,307,512,44 owner=39 paints=#9C9CA0FF role=text size=24 text=crt-royale-kurozumi
                  45 ListBoxItem at=100,351,512,86 by=scroll drawn focus heard="handheld: lcd-grid-v2" in=29 limit=100,248,512,275 op paints=#333337FF,#9C9CA0FF role=listitem
                    46 GroupedListRow at=100,351,512,85 by=clip in=29 limit=100,351,512,86 role=none
                      47 DockPanel at=110,365,492,24 by=clip in=29 limit=100,351,512,86 role=none
                        48 TextBlock at=596,365,6,24 box=6,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=1 in=29 layout=5.12,24 limit=100,351,512,86 owner=45 paints=#86868AFF role=text size=20 text=1
                        49 TextBlock at=110,365,478,24 box=478,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=HANDHELD in=29 layout=71.9,24 limit=100,351,512,86 owner=45 paints=#86868AFF role=text size=20 text=HANDHELD
                      50 Border at=104,394,504,41 by=clip in=29 limit=100,351,512,86 role=none
                        51 DockPanel at=114,400,484,29 by=clip in=29 limit=100,351,512,86 role=none
                          52 StackPanel at=114,400,484,29 by=clip in=29 limit=100,351,512,86 role=none
                            53 TextBlock at=114,400,484,29 box=484,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=lcd-grid-v2 in=29 layout=89.64,28.8 limit=100,351,512,86 owner=45 paints=#9C9CA0FF role=text size=24 text=lcd-grid-v2
                  54 ListBoxItem at=100,437,512,86 by=scroll drawn focus heard="long: long-names" in=29 limit=100,248,512,275 op paints=#333337FF,#9C9CA0FF role=listitem
                    55 GroupedListRow at=100,437,512,85 by=clip in=29 limit=100,437,512,86 role=none
                      56 DockPanel at=110,451,492,24 by=clip in=29 limit=100,437,512,86 role=none
                        57 TextBlock at=596,451,6,24 box=6,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=1 in=29 layout=5.12,24 limit=100,437,512,86 owner=54 paints=#86868AFF role=text size=20 text=1
                        58 TextBlock at=110,451,478,24 box=478,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=LONG in=29 layout=35.9,24 limit=100,437,512,86 owner=54 paints=#86868AFF role=text size=20 text=LONG
                      59 Border at=104,480,504,41 by=clip in=29 limit=100,437,512,86 paints=#4C9AE8FF role=none
                        60 DockPanel at=114,486,484,29 by=clip in=29 limit=100,437,512,86 role=none
                          61 StackPanel at=114,486,484,29 by=clip in=29 limit=100,437,512,86 role=none
                            62 TextBlock at=114,486,484,29 box=484,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=long-names in=29 layout=96.24,28.8 limit=100,437,512,86 owner=54 paints=#FFFFFFFF role=text size=24 text=long-names
                  63 RepeatButton at=612,248,16,16 drawn heard="Line up" name=PART_LineUpButton paints=#FFFFFFFF part role=button
                    64 ViewboxContainer at=616,254,8,4.4 by=clip limit=616,254,8,4.4 role=none
                  65 RepeatButton at=612,264,16,141 drawn heard="Page up" name=PART_PageUpButton part role=button
                  66 RepeatButton at=612,507,16,16 drawn heard="Line down" name=PART_LineDownButton paints=#FFFFFFFF part role=button
                    67 ViewboxContainer at=616,513,8,4.4 by=clip limit=616,513,8,4.4 role=none
              68 Border at=100,533,528,44 by=clip limit=88,186,1104,391 role=none
                69 Button at=100,533,151,44 by=clip drawn focus heard="UPDATE PACK" limit=88,186,1104,391 name=SNES.DownloadPack op paints=#55555AFF,#9C9CA0FF role=button text="Update Pack"
                  70 TextBlock at=117,539,117,32 box=117,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="UPDATE PACK" layout=116.04,31.2 limit=100,533,151,44 owner=69 paints=#9C9CA0FF role=text size=26 text="UPDATE PACK"
            71 DockPanel at=652,194,528,383 by=clip limit=88,186,1104,391 role=none
              72 StackPanel at=652,194,528,159 by=clip limit=88,186,1104,391 role=none
                73 Grid at=652,194,528,44 by=clip limit=88,186,1104,391 role=none
                  74 TextBlock at=652,194,195,44 box=195,44 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=long-names layout=96.24,28.8 limit=88,186,1104,391 name=SNES.ShaderName paints=#9C9CA0FF role=text size=24 text=long-names
                  75 StackPanel at=859,194,321,44 by=clip limit=88,186,1104,391 role=none
                    76 Button at=859,194,126,44 by=clip drawn heard="RESET ALL" limit=88,186,1104,391 name=SNES.ResetShader off paints=#55555AFF,#9C9CA0FF role=button text="Reset All"
                      77 TextBlock at=876,200,92,32 box=92,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="RESET ALL" layout=91.91,31.2 limit=859,194,126,44 off owner=76 paints=#55555AFF role=text size=26 text="RESET ALL"
                    78 Button at=993,194,187,44 by=clip drawn focus heard="USE THIS SHADER" limit=88,186,1104,391 name=SNES.UseShader op paints=#55555AFF,#9C9CA0FF role=button text="Use This Shader"
                      79 TextBlock at=1010,200,153,32 box=153,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="USE THIS SHADER" layout=152.57,31.2 limit=993,194,187,44 owner=78 paints=#9C9CA0FF role=text size=26 text="USE THIS SHADER"
                80 MonoText at=652,244,528,25 box=528,25 by=clip caps=12.854 drawn face="Noto Sans" heard=long/long-names.slangp layout=206.23,24.52 limit=88,186,1104,391 name=SNES.ShaderPath paints=#9C9CA0FF role=text size=18 text=long/long-names.slangp
                81 HintText at=652,275,528,24 box=528,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="2 parameters." layout=94.54,24 limit=88,186,1104,391 paints=#86868AFF role=text size=20 text="2 parameters."
                82 FilterBar at=652,307,514,46 by=clip limit=88,186,1104,391 name=SNES.ParameterSearch paints=#9C9CA0FF role=group
                  83 TextBox at=652,307,514,46 drawn focus heard="Search parameters" name=PART_Search op paints=#0E0E10FF,#55555AFF,#D2D2D6FF part role=edit
                    84 ScrollViewer allow="a text box scrolls its own text" at=663,314,492,32 name=PART_ScrollViewer paints=#D2D2D6FF part role=scrollviewer
                      85 TextBlock allow="a text box scrolls its own text" at=663,314,492,32 box=492,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search parameters" in=84 layout=168.51,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text="Search parameters"
              86 SliderList at=652,361,528,216 by=clip limit=88,186,1104,391 name=SNES.ShaderParameters paints=#9C9CA0FF role=list
                87 ScrollViewer at=652,361,528,216 fades=b name=PART_ScrollViewer overflow=172 paints=#1E1E1EFF,#333337FF,#4C9AE8FF,#55555AFF,#9C9CA0FF,#EEEEF0FF part role=scrollviewer viewport=652,361,512,216
                  88 ContentPresenter at=652,361,498,115 by=scroll in=87 limit=652,361,512,216 role=none
                    89 SectionHeader at=652,371,498,101 box=498,101 by=scroll caps=19.5986 drawn face="Barlow Condensed" heard="--- Beam, mask and the curvature of the tube's glass, for every console that draws its picture this way ---" in=87 layout=484.9,100.8 limit=652,361,512,216 paints=#B4B4B8FF role=text size=28 text="--- Beam, mask and the curvature of the tube's glass, for every console that draws its picture this way ---"
                  90 ContentPresenter at=652,476,498,122 by=scroll in=87 limit=652,361,512,216 role=none seen=652,476,498,101
                    91 SliderRow at=652,476,498,118 by=scroll in=87 limit=652,361,512,216 name=SNES.Parameter.BEAM_WIDTH_AT_THE_BRIGHTEST_PART_OF_THE_PICTURE role=none seen=652,476,498,101
                      92 StackPanel at=652,480,498,110 by=scroll in=87 limit=652,361,512,216 role=none seen=652,480,498,97
                        93 DockPanel at=652,480,512,58 by=scroll in=87 limit=652,361,512,216 role=none
                          94 Button at=1099,494,65,30 by=scroll drawn focus heard="Reset Horizontal beam width at the brightest part of the picture, in pixels of the source image" in=87 limit=652,361,512,216 op paints=#55555AFF,#9C9CA0FF role=button text=Reset
                            95 TextBlock at=1110,497,43,24 box=43,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=RESET in=87 layout=42.66,24 limit=1099,494,65,30 owner=94 paints=#9C9CA0FF role=text size=20 text=RESET
                          96 TextBlock at=1039,494,48,29 box=48,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard=0.60 in=87 layout=35.71,28.8 limit=652,361,512,216 paints=#9C9CA0FF role=text size=24 text=0.60
                          97 TextBlock at=652,480,375,58 box=375,58 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Horizontal beam width at the brightest part of the picture, in pixels of the source image" in=87 layout=373.3,57.6 limit=652,361,512,216 paints=#9C9CA0FF role=text size=24 text="Horizontal beam width at the brightest part of the picture, in pixels of the source image"
                        98 Slider at=652,540,498,50 by=scroll drawn focus heard="Horizontal beam width at the brightest part of the picture, in pixels of the source image" in=87 limit=652,361,512,216 op paints=#333337FF,#4C9AE8FF,#EEEEF0FF role=slider seen=652,540,498,37 sideways
                          99 RepeatButton at=652,555,144,20 drawn in=87 name=PART_DecreaseButton paints=#4C9AE8FF part role=button
                          100 RepeatButton at=815,555,335,20 drawn in=87 name=PART_IncreaseButton paints=#333337FF,#4C9AE8FF part role=button
                  101 RepeatButton at=1164,361,16,16 drawn heard="Line up" name=PART_LineUpButton paints=#FFFFFFFF part role=button
                    102 ViewboxContainer at=1168,367,8,4.4 by=clip limit=1168,367,8,4.4 role=none
                  103 RepeatButton at=1164,479,16,82 drawn heard="Page down" name=PART_PageDownButton part role=button
                  104 RepeatButton at=1164,561,16,16 drawn heard="Line down" name=PART_LineDownButton paints=#FFFFFFFF part role=button
                    105 ViewboxContainer at=1168,567,8,4.4 by=clip limit=1168,567,8,4.4 role=none
        106 ButtonBar at=1102,136,90,44 by=panel limit=64,124,1152,465 paints=#9C9CA0FF role=toolbar
          107 Button at=1102,136,90,44 by=clip drawn focus heard=CLOSE limit=1102,136,90,44 op paints=#55555AFF,#9C9CA0FF role=button text=Close
            108 TextBlock at=1119,142,56,32 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=55.51,31.2 limit=1102,136,90,44 owner=107 paints=#9C9CA0FF role=text size=26 text=CLOSE
focus 54
edge 8 up same
edge 8 down 22
edge 22 up 8
edge 22 down 30
edge 25 up 8
edge 25 down 30
edge 25 left 22
edge 25 right 78
edge 30 up "row:28/5/bezel / Mega Bezel / Presets / Base CRT Presets: MBZ 0 SMOOTH-ADV GDV"
edge 30 down 39
edge 30 left same
edge 30 right 83
edge 39 up 30
edge 39 down 45
edge 39 left same
edge 39 right 83
edge 45 up 39
edge 45 down 54
edge 45 left same
edge 45 right 83
edge 54 up 45
edge 54 down 69
edge 54 left same
edge 54 right 94
edge 69 up 54
edge 69 down "item:86/2/0/0/1/Shadow mask strength (0 = off, 1 = the whole of the phosphor pattern, as a television of the time)"
edge 69 left same
edge 69 right 98
edge 78 up 107
edge 78 down 83
edge 78 left 25
edge 78 right same
edge 83 up 78
edge 83 down "item:86/1/0/0/0/0/Reset Glow strength"
edge 83 left 39
edge 83 right same
edge 94 up 83
edge 94 down 98
edge 94 left 54
edge 94 right same
edge 98 up 94
edge 98 down "item:86/2/0/0/0/0/Reset Shadow mask strength (0 = off, 1 = the whole of the phosphor pattern, as a television of the time)"
edge 107 up same
edge 107 down 78
edge 107 left 8
edge 107 right same
edge "item:86/1/0/0/0/0/Reset Glow strength" up 83
edge "item:86/1/0/0/0/0/Reset Glow strength" down "item:86/1/0/0/1/Glow strength"
edge "item:86/1/0/0/0/0/Reset Glow strength" left 45
edge "item:86/1/0/0/0/0/Reset Glow strength" right same
edge "item:86/1/0/0/1/Glow strength" up "item:86/1/0/0/0/0/Reset Glow strength"
edge "item:86/1/0/0/1/Glow strength" down "item:86/2/0/0/0/0/Reset Scanline weight"
edge "item:86/2/0/0/0/0/Reset Shadow mask strength (0 = off, 1 = the whole of the phosphor pattern, as a television of the time)" up 98
edge "item:86/2/0/0/0/0/Reset Shadow mask strength (0 = off, 1 = the whole of the phosphor pattern, as a television of the time)" down "item:86/2/0/0/1/Shadow mask strength (0 = off, 1 = the whole of the phosphor pattern, as a television of the time)"
edge "item:86/2/0/0/0/0/Reset Shadow mask strength (0 = off, 1 = the whole of the phosphor pattern, as a television of the time)" left 54
edge "item:86/2/0/0/0/0/Reset Shadow mask strength (0 = off, 1 = the whole of the phosphor pattern, as a television of the time)" right same
edge "item:86/2/0/0/1/Shadow mask strength (0 = off, 1 = the whole of the phosphor pattern, as a television of the time)" up "item:86/2/0/0/0/0/Reset Shadow mask strength (0 = off, 1 = the whole of the phosphor pattern, as a television of the time)"
edge "item:86/2/0/0/1/Shadow mask strength (0 = off, 1 = the whole of the phosphor pattern, as a television of the time)" down same
edge "row:28/4/Simple CRT" up row:28/3/Scanlines
edge "row:28/4/Simple CRT" down "row:28/5/bezel / Mega Bezel / Presets / Base CRT Presets: MBZ 0 SMOOTH-ADV GDV"
edge "row:28/4/Simple CRT" left same
edge "row:28/4/Simple CRT" right 83
edge "row:28/5/bezel / Mega Bezel / Presets / Base CRT Presets: MBZ 0 SMOOTH-ADV GDV" up "row:28/4/Simple CRT"
edge "row:28/5/bezel / Mega Bezel / Presets / Base CRT Presets: MBZ 0 SMOOTH-ADV GDV" down 30
edge "row:28/5/bezel / Mega Bezel / Presets / Base CRT Presets: MBZ 0 SMOOTH-ADV GDV" left same
edge "row:28/5/bezel / Mega Bezel / Presets / Base CRT Presets: MBZ 0 SMOOTH-ADV GDV" right 83
press pageup focus="Dropdown PART_Facet (Category)" tab=GB title=Shaders top=ShaderSettingsWindow
press pagedown focus="Dropdown PART_Facet (Category)" tab=SNES title=Shaders top=ShaderSettingsWindow
press first focus="Dropdown PART_Facet (Category)" tab=SNES title=Shaders top=ShaderSettingsWindow
press last focus="Dropdown PART_Facet (Category)" tab=SNES title=Shaders top=ShaderSettingsWindow
press back top=none
