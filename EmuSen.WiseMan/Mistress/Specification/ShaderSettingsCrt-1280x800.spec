# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ShaderSettingsCrt-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 64,24,1152,668
title Shaders
footer "Drawn over the picture as it is shown, per console. A choice and every slider are saved at once and reach a running game of that console while its shader is in use. Not downloaded. libretro's slang pack, about 54 MB from buildbot.libretro.com, is the file RetroArch's updater fetches: some 2,600 presets. EmuSen ships none of it, and each shader keeps its authors' licence."
case ShaderSettingsCrt
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
          12 TabItem at=345,136,107,48 by=clip drawn focus heard=SG-1000 limit=88,136,1104,441 paints=#86868AFF role=tabitem sideways text=SG-1000
            13 TextBlock at=357,142,83,36 box=83,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=SG-1000 layout=82.68,36 limit=345,136,107,48 owner=12 paints=#86868AFF role=text size=30 text=SG-1000
          14 TabItem at=452,136,186,48 by=clip drawn focus heard="MASTER SYSTEM" limit=88,136,1104,441 paints=#86868AFF role=tabitem sideways text="Master System"
            15 TextBlock at=464,142,162,36 box=162,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard="MASTER SYSTEM" layout=161.79,36 limit=452,136,186,48 owner=14 paints=#86868AFF role=text size=30 text="MASTER SYSTEM"
          16 TabItem at=638,136,110,48 by=clip drawn focus heard=GENESIS limit=88,136,1104,441 paints=#86868AFF role=tabitem sideways text=Genesis
            17 TextBlock at=650,142,86,36 box=86,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GENESIS layout=85.47,36 limit=638,136,110,48 owner=16 paints=#86868AFF role=text size=30 text=GENESIS
          18 TabItem at=748,136,109,48 by=clip drawn focus heard="SEGA CD" limit=88,136,1104,441 paints=#86868AFF role=tabitem sideways text="Sega CD"
            19 TextBlock at=760,142,85,36 box=85,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard="SEGA CD" layout=84.18,36 limit=748,136,109,48 owner=18 paints=#86868AFF role=text size=30 text="SEGA CD"
          20 TabItem at=857,136,156,48 by=clip drawn focus heard=PLAYSTATION limit=88,136,1104,441 paints=#86868AFF role=tabitem sideways text=PlayStation
            21 TextBlock at=869,142,132,36 box=132,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=PLAYSTATION layout=131.91,36 limit=857,136,156,48 owner=20 paints=#86868AFF role=text size=30 text=PLAYSTATION
          22 ShaderPanel at=100,194,1080,383 by=clip limit=88,186,1104,391 role=none
            23 Grid at=100,194,528,383 by=clip limit=88,186,1104,391 role=none
              24 FilterBar at=100,194,528,46 by=clip limit=88,186,1104,391 name=SNES.ShaderSearch paints=#9C9CA0FF role=group
                25 TextBlock at=100,202,73,29 box=73,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Category layout=72.82,28.8 limit=100,194,528,46 paints=#86868AFF part role=text size=24 text=Category
                26 Dropdown at=181,194,180,44 drawn focus heard=Category name=PART_Facet op paints=#0E0E10FF,#55555AFF,#9C9CA0FF part role=combobox sideways
                  27 TextBlock at=193,199,136,32 box=136,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=All layout=20.07,31.2 limit=193,199,136,32 owner=26 paints=#9C9CA0FF role=text size=26 text=All
                  28 ViewboxContainer at=339,212.73,12,6.54 by=clip limit=339,212.73,12,6.54 nohit role=none
                29 TextBox at=369,194,259,46 drawn focus heard=Search name=PART_Search op paints=#0E0E10FF,#55555AFF,#D2D2D6FF part role=edit
                  30 ScrollViewer allow="a text box scrolls its own text" at=380,201,237,32 name=PART_ScrollViewer paints=#D2D2D6FF part role=scrollviewer
                    31 TextBlock allow="a text box scrolls its own text" at=380,201,237,32 box=237,32 caps=18.1987 drawn face="Barlow Condensed" heard=Search in=30 layout=61,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text=Search
              32 GroupedList at=100,248,528,262 by=clip limit=88,186,1104,391 name=SNES.ShaderList paints=#1B1B1EFF,#FFFFFFFF role=list
                33 ScrollViewer at=100,248,528,262 name=PART_ScrollViewer overflow=0 paints=#1E1E1EFF,#FFFFFFFF part role=scrollviewer viewport=100,248,528,262
                  34 ListBoxItem at=100,248,528,86 by=fixed-scroll drawn focus heard="Built into EmuSen: None" in=33 limit=100,248,528,262 op paints=#333337FF,#9C9CA0FF role=listitem
                    35 GroupedListRow at=100,248,528,85 by=clip in=33 limit=100,248,528,86 role=none
                      36 DockPanel at=110,262,508,24 by=clip in=33 limit=100,248,528,86 role=none
                        37 TextBlock at=609,262,9,24 box=9,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=5 in=33 layout=8.18,24 limit=100,248,528,86 owner=34 paints=#86868AFF role=text size=20 text=5
                        38 TextBlock at=110,262,491,24 box=491,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="BUILT INTO EMUSEN" in=33 layout=132.18,24 limit=100,248,528,86 owner=34 paints=#86868AFF role=text size=20 text="BUILT INTO EMUSEN"
                      39 Border at=104,291,520,41 by=clip in=33 limit=100,248,528,86 role=none
                        40 DockPanel at=114,297,500,29 by=clip in=33 limit=100,248,528,86 role=none
                          41 Border at=557,299,57,26 by=clip in=33 limit=100,248,528,86 paints=#0E0E10FF role=none
                            42 TextBlock at=565,300,41,24 box=41,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="In use" in=33 layout=40.88,24 limit=100,248,528,86 owner=34 paints=#9C9CA0FF role=text size=20 text="In use"
                          43 StackPanel at=114,297,435,29 by=clip in=33 limit=100,248,528,86 role=none
                            44 TextBlock at=114,297,435,29 box=435,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=None in=33 layout=41.74,28.8 limit=100,248,528,86 owner=34 paints=#9C9CA0FF role=text size=24 text=None
                  45 ListBoxItem at=100,334,528,44 by=fixed-scroll drawn focus heard=CRT in=33 limit=100,248,528,262 op paints=#333337FF,#9C9CA0FF role=listitem
                    46 GroupedListRow at=100,334,528,43 by=clip in=33 limit=100,334,528,44 role=none
                      47 Border at=104,335,520,41 by=clip in=33 limit=100,334,528,44 paints=#4C9AE8FF role=none
                        48 DockPanel at=114,341,500,29 by=clip in=33 limit=100,334,528,44 role=none
                          49 StackPanel at=114,341,500,29 by=clip in=33 limit=100,334,528,44 role=none
                            50 TextBlock at=114,341,500,29 box=500,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=CRT in=33 layout=31.49,28.8 limit=100,334,528,44 owner=45 paints=#FFFFFFFF role=text size=24 text=CRT
                  51 ListBoxItem at=100,378,528,44 by=fixed-scroll drawn focus heard="CRT (Lottes)" in=33 limit=100,248,528,262 op paints=#333337FF,#9C9CA0FF role=listitem
                    52 GroupedListRow at=100,378,528,43 by=clip in=33 limit=100,378,528,44 role=none
                      53 Border at=104,379,520,41 by=clip in=33 limit=100,378,528,44 role=none
                        54 DockPanel at=114,385,500,29 by=clip in=33 limit=100,378,528,44 role=none
                          55 StackPanel at=114,385,500,29 by=clip in=33 limit=100,378,528,44 role=none
                            56 TextBlock at=114,385,500,29 box=500,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="CRT (Lottes)" in=33 layout=97.92,28.8 limit=100,378,528,44 owner=51 paints=#9C9CA0FF role=text size=24 text="CRT (Lottes)"
                  57 ListBoxItem at=100,422,528,44 by=fixed-scroll drawn focus heard=Scanlines in=33 limit=100,248,528,262 op paints=#333337FF,#9C9CA0FF role=listitem
                    58 GroupedListRow at=100,422,528,43 by=clip in=33 limit=100,422,528,44 role=none
                      59 Border at=104,423,520,41 by=clip in=33 limit=100,422,528,44 role=none
                        60 DockPanel at=114,429,500,29 by=clip in=33 limit=100,422,528,44 role=none
                          61 StackPanel at=114,429,500,29 by=clip in=33 limit=100,422,528,44 role=none
                            62 TextBlock at=114,429,500,29 box=500,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Scanlines in=33 layout=77.93,28.8 limit=100,422,528,44 owner=57 paints=#9C9CA0FF role=text size=24 text=Scanlines
                  63 ListBoxItem at=100,466,528,44 by=fixed-scroll drawn focus heard="Simple CRT" in=33 limit=100,248,528,262 op paints=#333337FF,#9C9CA0FF role=listitem
                    64 GroupedListRow at=100,466,528,43 by=clip in=33 limit=100,466,528,44 role=none
                      65 Border at=104,467,520,41 by=clip in=33 limit=100,466,528,44 role=none
                        66 DockPanel at=114,473,500,29 by=clip in=33 limit=100,466,528,44 role=none
                          67 StackPanel at=114,473,500,29 by=clip in=33 limit=100,466,528,44 role=none
                            68 TextBlock at=114,473,500,29 box=500,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Simple CRT" in=33 layout=91.2,28.8 limit=100,466,528,44 owner=63 paints=#9C9CA0FF role=text size=24 text="Simple CRT"
              69 Border at=100,520,528,57 by=clip limit=88,186,1104,391 role=none
                70 Button at=100,520,182,44 by=clip drawn focus heard="DOWNLOAD PACK" limit=88,186,1104,391 name=SNES.DownloadPack op paints=#55555AFF,#9C9CA0FF role=button text="Download Pack"
                  71 TextBlock at=117,526,148,32 box=148,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="DOWNLOAD PACK" layout=147.39,31.2 limit=100,520,182,44 owner=70 paints=#9C9CA0FF role=text size=26 text="DOWNLOAD PACK"
            72 DockPanel at=652,194,528,383 by=clip limit=88,186,1104,391 role=none
              73 StackPanel at=652,194,528,184 by=clip limit=88,186,1104,391 role=none
                74 Grid at=652,194,528,44 by=clip limit=88,186,1104,391 role=none
                  75 TextBlock at=652,194,195,44 box=195,44 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=CRT layout=31.49,28.8 limit=88,186,1104,391 name=SNES.ShaderName paints=#9C9CA0FF role=text size=24 text=CRT
                  76 StackPanel at=859,194,321,44 by=clip limit=88,186,1104,391 role=none
                    77 Button at=859,194,126,44 by=clip drawn focus heard="RESET ALL" limit=88,186,1104,391 name=SNES.ResetShader op paints=#55555AFF,#9C9CA0FF role=button text="Reset All"
                      78 TextBlock at=876,200,92,32 box=92,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="RESET ALL" layout=91.91,31.2 limit=859,194,126,44 owner=77 paints=#9C9CA0FF role=text size=26 text="RESET ALL"
                    79 Button at=993,194,187,44 by=clip drawn focus heard="USE THIS SHADER" limit=88,186,1104,391 name=SNES.UseShader op paints=#55555AFF,#9C9CA0FF role=button text="Use This Shader"
                      80 TextBlock at=1010,200,153,32 box=153,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="USE THIS SHADER" layout=152.57,31.2 limit=993,194,187,44 owner=79 paints=#9C9CA0FF role=text size=26 text="USE THIS SHADER"
                81 MonoText at=652,244,528,50 box=528,50 by=clip caps=12.854 drawn face="Noto Sans" heard="Built into EmuSen. EmuSen's own, from the physics and measurements cited in EmuSen_CRT.md" layout=467.8,49.03 limit=88,186,1104,391 name=SNES.ShaderPath paints=#9C9CA0FF role=text size=18 text="Built into EmuSen. EmuSen's own, from the physics and measurements cited in EmuSen_CRT.md"
                82 HintText at=652,300,528,24 box=528,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="28 parameters." layout=102.98,24 limit=88,186,1104,391 paints=#86868AFF role=text size=20 text="28 parameters."
                83 FilterBar at=652,332,514,46 by=clip limit=88,186,1104,391 name=SNES.ParameterSearch paints=#9C9CA0FF role=group
                  84 TextBox at=652,332,514,46 drawn focus heard="Search parameters" name=PART_Search op paints=#0E0E10FF,#55555AFF,#D2D2D6FF part role=edit
                    85 ScrollViewer allow="a text box scrolls its own text" at=663,339,492,32 name=PART_ScrollViewer paints=#D2D2D6FF part role=scrollviewer
                      86 TextBlock allow="a text box scrolls its own text" at=663,339,492,32 box=492,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search parameters" in=85 layout=168.51,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text="Search parameters"
              87 SliderList at=652,386,528,191 by=clip limit=88,186,1104,391 name=SNES.ShaderParameters paints=#9C9CA0FF role=list
                88 ScrollViewer at=652,386,528,191 fades=b name=PART_ScrollViewer overflow=2441 paints=#1E1E1EFF,#333337FF,#4C9AE8FF,#55555AFF,#86868AFF,#9C9CA0FF,#EEEEF0FF part role=scrollviewer viewport=652,386,512,191
                  89 ContentPresenter at=652,386,498,94 by=scroll in=88 limit=652,386,512,191 role=none
                    90 SliderRow at=652,386,498,90 by=scroll in=88 limit=652,386,512,191 name=SNES.Parameter.quality role=none
                      91 StackPanel at=652,390,498,82 by=scroll in=88 limit=652,386,512,191 role=none
                        92 DockPanel at=652,390,498,30 by=scroll in=88 limit=652,386,512,191 role=none
                          93 Button at=1085,390,65,30 by=scroll drawn focus heard="Reset Quality" in=88 limit=652,386,512,191 op paints=#55555AFF,#9C9CA0FF role=button text=Reset
                            94 TextBlock at=1096,393,43,24 box=43,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=RESET in=88 layout=42.66,24 limit=1085,390,65,30 owner=93 paints=#9C9CA0FF role=text size=20 text=RESET
                          95 TextBlock at=998,390,75,29 box=75,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard=Balanced in=88 layout=74.69,28.8 limit=652,386,512,191 paints=#9C9CA0FF role=text size=24 text=Balanced
                          96 TextBlock at=652,390,334,29 box=334,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard=Quality in=88 layout=55.3,28.8 limit=652,386,512,191 paints=#9C9CA0FF role=text size=24 text=Quality
                        97 Slider at=652,422,498,50 by=scroll drawn focus heard=Quality in=88 limit=652,386,512,191 op paints=#333337FF,#4C9AE8FF,#EEEEF0FF role=slider sideways
                          98 RepeatButton at=652,437,239,20 drawn in=88 name=PART_DecreaseButton paints=#4C9AE8FF part role=button
                          99 RepeatButton at=911,437,239,20 drawn in=88 name=PART_IncreaseButton paints=#333337FF,#4C9AE8FF part role=button
                  100 ContentPresenter at=652,480,498,94 by=scroll in=88 limit=652,386,512,191 role=none
                    101 SliderRow at=652,480,498,90 by=scroll in=88 limit=652,386,512,191 name=SNES.Parameter.signal role=none
                      102 StackPanel at=652,484,498,82 by=scroll in=88 limit=652,386,512,191 role=none
                        103 DockPanel at=652,484,498,30 by=scroll in=88 limit=652,386,512,191 role=none
                          104 Button at=1085,484,65,30 by=scroll drawn focus heard="Reset Signal" in=88 limit=652,386,512,191 op paints=#55555AFF,#9C9CA0FF role=button text=Reset
                            105 TextBlock at=1096,487,43,24 box=43,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=RESET in=88 layout=42.66,24 limit=1085,484,65,30 owner=104 paints=#9C9CA0FF role=text size=20 text=RESET
                          106 TextBlock at=986,484,87,29 box=87,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard=Composite in=88 layout=86.3,28.8 limit=652,386,512,191 paints=#9C9CA0FF role=text size=24 text=Composite
                          107 TextBlock at=652,484,322,29 box=322,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard=Signal in=88 layout=49.32,28.8 limit=652,386,512,191 paints=#9C9CA0FF role=text size=24 text=Signal
                        108 Slider at=652,516,498,50 by=scroll drawn focus heard=Signal in=88 limit=652,386,512,191 op paints=#333337FF,#4C9AE8FF,#EEEEF0FF role=slider sideways
                          109 RepeatButton at=652,531,478,20 drawn in=88 name=PART_DecreaseButton paints=#4C9AE8FF part role=button
                  110 ContentPresenter at=652,574,498,94 by=scroll in=88 limit=652,386,512,191 role=none seen=652,574,498,3
                    111 SliderRow at=652,574,498,90 by=scroll in=88 limit=652,386,512,191 name=SNES.Parameter.standard paints=#333337FF,#4C9AE8FF,#55555AFF,#9C9CA0FF,#EEEEF0FF role=none seen=652,574,498,3
                  112 RepeatButton at=1164,386,16,16 drawn heard="Line up" name=PART_LineUpButton paints=#FFFFFFFF part role=button
                    113 ViewboxContainer at=1168,392,8,4.4 by=clip limit=1168,392,8,4.4 role=none
                  114 RepeatButton at=1164,442,16,119 drawn heard="Page down" name=PART_PageDownButton part role=button
                  115 RepeatButton at=1164,561,16,16 drawn heard="Line down" name=PART_LineDownButton paints=#FFFFFFFF part role=button
                    116 ViewboxContainer at=1168,567,8,4.4 by=clip limit=1168,567,8,4.4 role=none
        117 ButtonBar at=1102,136,90,44 by=panel limit=64,124,1152,465 paints=#9C9CA0FF role=toolbar
          118 Button at=1102,136,90,44 by=clip drawn focus heard=CLOSE limit=1102,136,90,44 op paints=#55555AFF,#9C9CA0FF role=button text=Close
            119 TextBlock at=1119,142,56,32 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=55.51,31.2 limit=1102,136,90,44 owner=118 paints=#EEEEF0FF role=text size=26 text=CLOSE
focus 118
edge 8 up 118
edge 8 down 26
edge 26 up 8
edge 26 down 34
edge 29 up 8
edge 29 down 34
edge 29 left 26
edge 29 right 77
edge 34 up 29
edge 34 down 45
edge 34 left same
edge 34 right 84
edge 45 up 34
edge 45 down 51
edge 45 left same
edge 45 right 84
edge 51 up 45
edge 51 down 57
edge 51 left same
edge 51 right 93
edge 57 up 51
edge 57 down 63
edge 57 left same
edge 57 right 84
edge 63 up 57
edge 63 down 70
edge 63 left same
edge 63 right 104
edge 70 up 63
edge 70 down "item:87/2/0/0/1/Television standard"
edge 70 left same
edge 70 right 108
edge 77 up 8
edge 77 down 84
edge 77 left 29
edge 77 right 79
edge 79 up 118
edge 79 down 84
edge 79 left 77
edge 79 right same
edge 84 up 77
edge 84 down 93
edge 84 left 45
edge 84 right same
edge 93 up 84
edge 93 down 97
edge 93 left 51
edge 93 right same
edge 97 up 93
edge 97 down 104
edge 104 up 97
edge 104 down 108
edge 104 left 63
edge 104 right same
edge 108 up 104
edge 108 down "item:87/2/0/0/0/0/Reset Television standard"
edge 118 up same
edge 118 down 79
edge 118 left 8
edge 118 right same
edge "item:87/2/0/0/0/0/Reset Television standard" up 108
edge "item:87/2/0/0/0/0/Reset Television standard" down "item:87/2/0/0/1/Television standard"
edge "item:87/2/0/0/0/0/Reset Television standard" left 63
edge "item:87/2/0/0/0/0/Reset Television standard" right same
press pageup focus="Dropdown PART_Facet (Category)" tab=GB title=Shaders top=ShaderSettingsWindow
press pagedown focus="Dropdown PART_Facet (Category)" tab=SNES title=Shaders top=ShaderSettingsWindow
press first focus="Dropdown PART_Facet (Category)" tab=SNES title=Shaders top=ShaderSettingsWindow
press last focus="Dropdown PART_Facet (Category)" tab=SNES title=Shaders top=ShaderSettingsWindow
press back top=none
