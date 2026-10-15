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
                27 ScrollViewer at=100,248,528,275 fades=t name=PART_ScrollViewer overflow=375 paints=#1E1E1EFF,#55555AFF,#FFFFFFFF part role=scrollviewer viewport=100,248,512,275
                  28 ListBoxItem at=100,221,512,86 by=scroll drawn focus heard="crt: crt-lottes" in=27 limit=100,248,512,275 op paints=#333337FF,#9C9CA0FF role=listitem seen=100,248,512,59
                    29 GroupedListRow at=100,221,512,85 by=clip in=27 limit=100,221,512,86 role=none seen=100,248,512,58
                      30 DockPanel at=110,235,492,24 by=clip in=27 limit=100,221,512,86 role=none seen=110,248,492,11
                        31 TextBlock at=593,235,9,24 box=9,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=2 in=27 layout=8.04,24 limit=100,221,512,86 owner=28 paints=#86868AFF role=text seen=593,248,9,11 size=20 text=2
                        32 TextBlock at=110,235,475,24 box=475,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=CRT in=27 layout=26.24,24 limit=100,221,512,86 owner=28 paints=#86868AFF role=text seen=110,248,475,11 size=20 text=CRT
                      33 Border at=104,264,504,41 by=clip in=27 limit=100,221,512,86 role=none
                        34 DockPanel at=114,270,484,29 by=clip in=27 limit=100,221,512,86 role=none
                          35 StackPanel at=114,270,484,29 by=clip in=27 limit=100,221,512,86 role=none
                            36 TextBlock at=114,270,484,29 box=484,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=crt-lottes in=27 layout=76.39,28.8 limit=100,221,512,86 owner=28 paints=#9C9CA0FF role=text size=24 text=crt-lottes
                  37 ListBoxItem at=100,307,512,44 by=scroll drawn focus heard=crt-royale-kurozumi in=27 limit=100,248,512,275 op paints=#333337FF,#9C9CA0FF role=listitem
                    38 GroupedListRow at=100,307,512,43 by=clip in=27 limit=100,307,512,44 role=none
                      39 Border at=104,308,504,41 by=clip in=27 limit=100,307,512,44 role=none
                        40 DockPanel at=114,314,484,29 by=clip in=27 limit=100,307,512,44 role=none
                          41 StackPanel at=114,314,484,29 by=clip in=27 limit=100,307,512,44 role=none
                            42 TextBlock at=114,314,484,29 box=484,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=crt-royale-kurozumi in=27 layout=163.49,28.8 limit=100,307,512,44 owner=37 paints=#9C9CA0FF role=text size=24 text=crt-royale-kurozumi
                  43 ListBoxItem at=100,351,512,86 by=scroll drawn focus heard="handheld: lcd-grid-v2" in=27 limit=100,248,512,275 op paints=#333337FF,#9C9CA0FF role=listitem
                    44 GroupedListRow at=100,351,512,85 by=clip in=27 limit=100,351,512,86 role=none
                      45 DockPanel at=110,365,492,24 by=clip in=27 limit=100,351,512,86 role=none
                        46 TextBlock at=596,365,6,24 box=6,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=1 in=27 layout=5.12,24 limit=100,351,512,86 owner=43 paints=#86868AFF role=text size=20 text=1
                        47 TextBlock at=110,365,478,24 box=478,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=HANDHELD in=27 layout=71.9,24 limit=100,351,512,86 owner=43 paints=#86868AFF role=text size=20 text=HANDHELD
                      48 Border at=104,394,504,41 by=clip in=27 limit=100,351,512,86 role=none
                        49 DockPanel at=114,400,484,29 by=clip in=27 limit=100,351,512,86 role=none
                          50 StackPanel at=114,400,484,29 by=clip in=27 limit=100,351,512,86 role=none
                            51 TextBlock at=114,400,484,29 box=484,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=lcd-grid-v2 in=27 layout=89.64,28.8 limit=100,351,512,86 owner=43 paints=#9C9CA0FF role=text size=24 text=lcd-grid-v2
                  52 ListBoxItem at=100,437,512,86 by=scroll drawn focus heard="long: long-names" in=27 limit=100,248,512,275 op paints=#333337FF,#9C9CA0FF role=listitem
                    53 GroupedListRow at=100,437,512,85 by=clip in=27 limit=100,437,512,86 role=none
                      54 DockPanel at=110,451,492,24 by=clip in=27 limit=100,437,512,86 role=none
                        55 TextBlock at=596,451,6,24 box=6,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=1 in=27 layout=5.12,24 limit=100,437,512,86 owner=52 paints=#86868AFF role=text size=20 text=1
                        56 TextBlock at=110,451,478,24 box=478,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=LONG in=27 layout=35.9,24 limit=100,437,512,86 owner=52 paints=#86868AFF role=text size=20 text=LONG
                      57 Border at=104,480,504,41 by=clip in=27 limit=100,437,512,86 paints=#4C9AE8FF role=none
                        58 DockPanel at=114,486,484,29 by=clip in=27 limit=100,437,512,86 role=none
                          59 StackPanel at=114,486,484,29 by=clip in=27 limit=100,437,512,86 role=none
                            60 TextBlock at=114,486,484,29 box=484,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=long-names in=27 layout=96.24,28.8 limit=100,437,512,86 owner=52 paints=#FFFFFFFF role=text size=24 text=long-names
                  61 RepeatButton at=612,248,16,16 drawn heard="Line up" name=PART_LineUpButton paints=#FFFFFFFF part role=button
                    62 ViewboxContainer at=616,254,8,4.4 by=clip limit=616,254,8,4.4 role=none
                  63 RepeatButton at=612,264,16,141 drawn heard="Page up" name=PART_PageUpButton part role=button
                  64 RepeatButton at=612,507,16,16 drawn heard="Line down" name=PART_LineDownButton paints=#FFFFFFFF part role=button
                    65 ViewboxContainer at=616,513,8,4.4 by=clip limit=616,513,8,4.4 role=none
              66 Border at=100,533,528,44 by=clip limit=88,186,1104,391 role=none
                67 Button at=100,533,151,44 by=clip drawn focus heard="UPDATE PACK" limit=88,186,1104,391 name=SNES.DownloadPack op paints=#55555AFF,#9C9CA0FF role=button text="Update Pack"
                  68 TextBlock at=117,539,117,32 box=117,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="UPDATE PACK" layout=116.04,31.2 limit=100,533,151,44 owner=67 paints=#9C9CA0FF role=text size=26 text="UPDATE PACK"
            69 DockPanel at=652,194,528,383 by=clip limit=88,186,1104,391 role=none
              70 StackPanel at=652,194,528,159 by=clip limit=88,186,1104,391 role=none
                71 Grid at=652,194,528,44 by=clip limit=88,186,1104,391 role=none
                  72 TextBlock at=652,194,195,44 box=195,44 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=long-names layout=96.24,28.8 limit=88,186,1104,391 name=SNES.ShaderName paints=#9C9CA0FF role=text size=24 text=long-names
                  73 StackPanel at=859,194,321,44 by=clip limit=88,186,1104,391 role=none
                    74 Button at=859,194,126,44 by=clip drawn heard="RESET ALL" limit=88,186,1104,391 name=SNES.ResetShader off paints=#55555AFF,#9C9CA0FF role=button text="Reset All"
                      75 TextBlock at=876,200,92,32 box=92,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="RESET ALL" layout=91.91,31.2 limit=859,194,126,44 off owner=74 paints=#55555AFF role=text size=26 text="RESET ALL"
                    76 Button at=993,194,187,44 by=clip drawn focus heard="USE THIS SHADER" limit=88,186,1104,391 name=SNES.UseShader op paints=#55555AFF,#9C9CA0FF role=button text="Use This Shader"
                      77 TextBlock at=1010,200,153,32 box=153,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="USE THIS SHADER" layout=152.57,31.2 limit=993,194,187,44 owner=76 paints=#9C9CA0FF role=text size=26 text="USE THIS SHADER"
                78 MonoText at=652,244,528,25 box=528,25 by=clip caps=12.854 drawn face="Noto Sans" heard=long/long-names.slangp layout=206.23,24.52 limit=88,186,1104,391 name=SNES.ShaderPath paints=#9C9CA0FF role=text size=18 text=long/long-names.slangp
                79 HintText at=652,275,528,24 box=528,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="2 parameters." layout=94.54,24 limit=88,186,1104,391 paints=#86868AFF role=text size=20 text="2 parameters."
                80 FilterBar at=652,307,514,46 by=clip limit=88,186,1104,391 name=SNES.ParameterSearch paints=#9C9CA0FF role=group
                  81 TextBox at=652,307,514,46 drawn focus heard="Search parameters" name=PART_Search op paints=#0E0E10FF,#55555AFF,#D2D2D6FF part role=edit
                    82 ScrollViewer allow="a text box scrolls its own text" at=663,314,492,32 name=PART_ScrollViewer paints=#D2D2D6FF part role=scrollviewer
                      83 TextBlock allow="a text box scrolls its own text" at=663,314,492,32 box=492,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search parameters" in=82 layout=168.51,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text="Search parameters"
              84 SliderList at=652,361,528,216 by=clip limit=88,186,1104,391 name=SNES.ShaderParameters paints=#9C9CA0FF role=list
                85 ScrollViewer at=652,361,528,216 fades=b name=PART_ScrollViewer overflow=172 paints=#1E1E1EFF,#333337FF,#4C9AE8FF,#55555AFF,#9C9CA0FF,#EEEEF0FF part role=scrollviewer viewport=652,361,512,216
                  86 ContentPresenter at=652,361,498,115 by=scroll in=85 limit=652,361,512,216 role=none
                    87 SectionHeader at=652,371,498,101 box=498,101 by=scroll caps=19.5986 drawn face="Barlow Condensed" heard="--- Beam, mask and the curvature of the tube's glass, for every console that draws its picture this way ---" in=85 layout=484.9,100.8 limit=652,361,512,216 paints=#B4B4B8FF role=text size=28 text="--- Beam, mask and the curvature of the tube's glass, for every console that draws its picture this way ---"
                  88 ContentPresenter at=652,476,498,122 by=scroll in=85 limit=652,361,512,216 role=none seen=652,476,498,101
                    89 SliderRow at=652,476,498,118 by=scroll in=85 limit=652,361,512,216 name=SNES.Parameter.BEAM_WIDTH_AT_THE_BRIGHTEST_PART_OF_THE_PICTURE role=none seen=652,476,498,101
                      90 StackPanel at=652,480,498,110 by=scroll in=85 limit=652,361,512,216 role=none seen=652,480,498,97
                        91 DockPanel at=652,480,512,58 by=scroll in=85 limit=652,361,512,216 role=none
                          92 Button at=1099,494,65,30 by=scroll drawn focus heard="Reset Horizontal beam width at the brightest part of the picture, in pixels of the source image" in=85 limit=652,361,512,216 op paints=#55555AFF,#9C9CA0FF role=button text=Reset
                            93 TextBlock at=1110,497,43,24 box=43,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=RESET in=85 layout=42.66,24 limit=1099,494,65,30 owner=92 paints=#9C9CA0FF role=text size=20 text=RESET
                          94 TextBlock at=1039,494,48,29 box=48,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard=0.60 in=85 layout=35.71,28.8 limit=652,361,512,216 paints=#9C9CA0FF role=text size=24 text=0.60
                          95 TextBlock at=652,480,375,58 box=375,58 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Horizontal beam width at the brightest part of the picture, in pixels of the source image" in=85 layout=373.3,57.6 limit=652,361,512,216 paints=#9C9CA0FF role=text size=24 text="Horizontal beam width at the brightest part of the picture, in pixels of the source image"
                        96 Slider at=652,540,498,50 by=scroll drawn focus heard="Horizontal beam width at the brightest part of the picture, in pixels of the source image" in=85 limit=652,361,512,216 op paints=#333337FF,#4C9AE8FF,#EEEEF0FF role=slider seen=652,540,498,37 sideways
                          97 RepeatButton at=652,555,144,20 drawn in=85 name=PART_DecreaseButton paints=#4C9AE8FF part role=button
                          98 RepeatButton at=815,555,335,20 drawn in=85 name=PART_IncreaseButton paints=#333337FF,#4C9AE8FF part role=button
                  99 RepeatButton at=1164,361,16,16 drawn heard="Line up" name=PART_LineUpButton paints=#FFFFFFFF part role=button
                    100 ViewboxContainer at=1168,367,8,4.4 by=clip limit=1168,367,8,4.4 role=none
                  101 RepeatButton at=1164,479,16,82 drawn heard="Page down" name=PART_PageDownButton part role=button
                  102 RepeatButton at=1164,561,16,16 drawn heard="Line down" name=PART_LineDownButton paints=#FFFFFFFF part role=button
                    103 ViewboxContainer at=1168,567,8,4.4 by=clip limit=1168,567,8,4.4 role=none
        104 ButtonBar at=1102,136,90,44 by=panel limit=64,124,1152,465 paints=#9C9CA0FF role=toolbar
          105 Button at=1102,136,90,44 by=clip drawn focus heard=CLOSE limit=1102,136,90,44 op paints=#55555AFF,#9C9CA0FF role=button text=Close
            106 TextBlock at=1119,142,56,32 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=55.51,31.2 limit=1102,136,90,44 owner=105 paints=#9C9CA0FF role=text size=26 text=CLOSE
focus 52
edge 8 up same
edge 8 down 20
edge 20 up 8
edge 20 down 28
edge 23 up 8
edge 23 down 28
edge 23 left 20
edge 23 right 76
edge 28 up "row:26/5/bezel / Mega Bezel / Presets / Base CRT Presets: MBZ 0 SMOOTH-ADV GDV"
edge 28 down 37
edge 28 left same
edge 28 right 81
edge 37 up 28
edge 37 down 43
edge 37 left same
edge 37 right 81
edge 43 up 37
edge 43 down 52
edge 43 left same
edge 43 right 81
edge 52 up 43
edge 52 down 67
edge 52 left same
edge 52 right 92
edge 67 up 52
edge 67 down "item:84/2/0/0/1/Shadow mask strength (0 = off, 1 = the whole of the phosphor pattern, as a television of the time)"
edge 67 left same
edge 67 right 96
edge 76 up 105
edge 76 down 81
edge 76 left 23
edge 76 right same
edge 81 up 76
edge 81 down "item:84/1/0/0/0/0/Reset Glow strength"
edge 81 left 37
edge 81 right same
edge 92 up 81
edge 92 down 96
edge 92 left 52
edge 92 right same
edge 96 up 92
edge 96 down "item:84/2/0/0/0/0/Reset Shadow mask strength (0 = off, 1 = the whole of the phosphor pattern, as a television of the time)"
edge 105 up same
edge 105 down 76
edge 105 left 8
edge 105 right same
edge "item:84/1/0/0/0/0/Reset Glow strength" up 81
edge "item:84/1/0/0/0/0/Reset Glow strength" down "item:84/1/0/0/1/Glow strength"
edge "item:84/1/0/0/0/0/Reset Glow strength" left 43
edge "item:84/1/0/0/0/0/Reset Glow strength" right same
edge "item:84/1/0/0/1/Glow strength" up "item:84/1/0/0/0/0/Reset Glow strength"
edge "item:84/1/0/0/1/Glow strength" down "item:84/2/0/0/0/0/Reset Scanline weight"
edge "item:84/2/0/0/0/0/Reset Shadow mask strength (0 = off, 1 = the whole of the phosphor pattern, as a television of the time)" up 96
edge "item:84/2/0/0/0/0/Reset Shadow mask strength (0 = off, 1 = the whole of the phosphor pattern, as a television of the time)" down "item:84/2/0/0/1/Shadow mask strength (0 = off, 1 = the whole of the phosphor pattern, as a television of the time)"
edge "item:84/2/0/0/0/0/Reset Shadow mask strength (0 = off, 1 = the whole of the phosphor pattern, as a television of the time)" left 52
edge "item:84/2/0/0/0/0/Reset Shadow mask strength (0 = off, 1 = the whole of the phosphor pattern, as a television of the time)" right same
edge "item:84/2/0/0/1/Shadow mask strength (0 = off, 1 = the whole of the phosphor pattern, as a television of the time)" up "item:84/2/0/0/0/0/Reset Shadow mask strength (0 = off, 1 = the whole of the phosphor pattern, as a television of the time)"
edge "item:84/2/0/0/1/Shadow mask strength (0 = off, 1 = the whole of the phosphor pattern, as a television of the time)" down same
edge "row:26/4/Simple CRT" up row:26/3/Scanlines
edge "row:26/4/Simple CRT" down "row:26/5/bezel / Mega Bezel / Presets / Base CRT Presets: MBZ 0 SMOOTH-ADV GDV"
edge "row:26/4/Simple CRT" left same
edge "row:26/4/Simple CRT" right 81
edge "row:26/5/bezel / Mega Bezel / Presets / Base CRT Presets: MBZ 0 SMOOTH-ADV GDV" up "row:26/4/Simple CRT"
edge "row:26/5/bezel / Mega Bezel / Presets / Base CRT Presets: MBZ 0 SMOOTH-ADV GDV" down 28
edge "row:26/5/bezel / Mega Bezel / Presets / Base CRT Presets: MBZ 0 SMOOTH-ADV GDV" left same
edge "row:26/5/bezel / Mega Bezel / Presets / Base CRT Presets: MBZ 0 SMOOTH-ADV GDV" right 81
press pageup focus="Dropdown PART_Facet (Category)" tab=GB title=Shaders top=ShaderSettingsWindow
press pagedown focus="Dropdown PART_Facet (Category)" tab=SNES title=Shaders top=ShaderSettingsWindow
press first focus="Dropdown PART_Facet (Category)" tab=SNES title=Shaders top=ShaderSettingsWindow
press last focus="Dropdown PART_Facet (Category)" tab=SNES title=Shaders top=ShaderSettingsWindow
press back top=none
