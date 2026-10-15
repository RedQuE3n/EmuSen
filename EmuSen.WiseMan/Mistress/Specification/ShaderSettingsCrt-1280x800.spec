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
              26 GroupedList at=100,248,528,262 by=clip limit=88,186,1104,391 name=SNES.ShaderList paints=#1B1B1EFF,#FFFFFFFF role=list
                27 ScrollViewer at=100,248,528,262 name=PART_ScrollViewer overflow=0 paints=#1E1E1EFF,#FFFFFFFF part role=scrollviewer viewport=100,248,528,262
                  28 ListBoxItem at=100,248,528,86 by=fixed-scroll drawn focus heard="Built into EmuSen: None" in=27 limit=100,248,528,262 op paints=#333337FF,#9C9CA0FF role=listitem
                    29 GroupedListRow at=100,248,528,85 by=clip in=27 limit=100,248,528,86 role=none
                      30 DockPanel at=110,262,508,24 by=clip in=27 limit=100,248,528,86 role=none
                        31 TextBlock at=609,262,9,24 box=9,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=5 in=27 layout=8.18,24 limit=100,248,528,86 owner=28 paints=#86868AFF role=text size=20 text=5
                        32 TextBlock at=110,262,491,24 box=491,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="BUILT INTO EMUSEN" in=27 layout=132.18,24 limit=100,248,528,86 owner=28 paints=#86868AFF role=text size=20 text="BUILT INTO EMUSEN"
                      33 Border at=104,291,520,41 by=clip in=27 limit=100,248,528,86 role=none
                        34 DockPanel at=114,297,500,29 by=clip in=27 limit=100,248,528,86 role=none
                          35 Border at=557,299,57,26 by=clip in=27 limit=100,248,528,86 paints=#0E0E10FF role=none
                            36 TextBlock at=565,300,41,24 box=41,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="In use" in=27 layout=40.88,24 limit=100,248,528,86 owner=28 paints=#9C9CA0FF role=text size=20 text="In use"
                          37 StackPanel at=114,297,435,29 by=clip in=27 limit=100,248,528,86 role=none
                            38 TextBlock at=114,297,435,29 box=435,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=None in=27 layout=41.74,28.8 limit=100,248,528,86 owner=28 paints=#9C9CA0FF role=text size=24 text=None
                  39 ListBoxItem at=100,334,528,44 by=fixed-scroll drawn focus heard=CRT in=27 limit=100,248,528,262 op paints=#333337FF,#9C9CA0FF role=listitem
                    40 GroupedListRow at=100,334,528,43 by=clip in=27 limit=100,334,528,44 role=none
                      41 Border at=104,335,520,41 by=clip in=27 limit=100,334,528,44 paints=#4C9AE8FF role=none
                        42 DockPanel at=114,341,500,29 by=clip in=27 limit=100,334,528,44 role=none
                          43 StackPanel at=114,341,500,29 by=clip in=27 limit=100,334,528,44 role=none
                            44 TextBlock at=114,341,500,29 box=500,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=CRT in=27 layout=31.49,28.8 limit=100,334,528,44 owner=39 paints=#FFFFFFFF role=text size=24 text=CRT
                  45 ListBoxItem at=100,378,528,44 by=fixed-scroll drawn focus heard="CRT (Lottes)" in=27 limit=100,248,528,262 op paints=#333337FF,#9C9CA0FF role=listitem
                    46 GroupedListRow at=100,378,528,43 by=clip in=27 limit=100,378,528,44 role=none
                      47 Border at=104,379,520,41 by=clip in=27 limit=100,378,528,44 role=none
                        48 DockPanel at=114,385,500,29 by=clip in=27 limit=100,378,528,44 role=none
                          49 StackPanel at=114,385,500,29 by=clip in=27 limit=100,378,528,44 role=none
                            50 TextBlock at=114,385,500,29 box=500,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="CRT (Lottes)" in=27 layout=97.92,28.8 limit=100,378,528,44 owner=45 paints=#9C9CA0FF role=text size=24 text="CRT (Lottes)"
                  51 ListBoxItem at=100,422,528,44 by=fixed-scroll drawn focus heard=Scanlines in=27 limit=100,248,528,262 op paints=#333337FF,#9C9CA0FF role=listitem
                    52 GroupedListRow at=100,422,528,43 by=clip in=27 limit=100,422,528,44 role=none
                      53 Border at=104,423,520,41 by=clip in=27 limit=100,422,528,44 role=none
                        54 DockPanel at=114,429,500,29 by=clip in=27 limit=100,422,528,44 role=none
                          55 StackPanel at=114,429,500,29 by=clip in=27 limit=100,422,528,44 role=none
                            56 TextBlock at=114,429,500,29 box=500,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Scanlines in=27 layout=77.93,28.8 limit=100,422,528,44 owner=51 paints=#9C9CA0FF role=text size=24 text=Scanlines
                  57 ListBoxItem at=100,466,528,44 by=fixed-scroll drawn focus heard="Simple CRT" in=27 limit=100,248,528,262 op paints=#333337FF,#9C9CA0FF role=listitem
                    58 GroupedListRow at=100,466,528,43 by=clip in=27 limit=100,466,528,44 role=none
                      59 Border at=104,467,520,41 by=clip in=27 limit=100,466,528,44 role=none
                        60 DockPanel at=114,473,500,29 by=clip in=27 limit=100,466,528,44 role=none
                          61 StackPanel at=114,473,500,29 by=clip in=27 limit=100,466,528,44 role=none
                            62 TextBlock at=114,473,500,29 box=500,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Simple CRT" in=27 layout=91.2,28.8 limit=100,466,528,44 owner=57 paints=#9C9CA0FF role=text size=24 text="Simple CRT"
              63 Border at=100,520,528,57 by=clip limit=88,186,1104,391 role=none
                64 Button at=100,520,182,44 by=clip drawn focus heard="DOWNLOAD PACK" limit=88,186,1104,391 name=SNES.DownloadPack op paints=#55555AFF,#9C9CA0FF role=button text="Download Pack"
                  65 TextBlock at=117,526,148,32 box=148,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="DOWNLOAD PACK" layout=147.39,31.2 limit=100,520,182,44 owner=64 paints=#9C9CA0FF role=text size=26 text="DOWNLOAD PACK"
            66 DockPanel at=652,194,528,383 by=clip limit=88,186,1104,391 role=none
              67 StackPanel at=652,194,528,184 by=clip limit=88,186,1104,391 role=none
                68 Grid at=652,194,528,44 by=clip limit=88,186,1104,391 role=none
                  69 TextBlock at=652,194,195,44 box=195,44 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=CRT layout=31.49,28.8 limit=88,186,1104,391 name=SNES.ShaderName paints=#9C9CA0FF role=text size=24 text=CRT
                  70 StackPanel at=859,194,321,44 by=clip limit=88,186,1104,391 role=none
                    71 Button at=859,194,126,44 by=clip drawn focus heard="RESET ALL" limit=88,186,1104,391 name=SNES.ResetShader op paints=#55555AFF,#9C9CA0FF role=button text="Reset All"
                      72 TextBlock at=876,200,92,32 box=92,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="RESET ALL" layout=91.91,31.2 limit=859,194,126,44 owner=71 paints=#9C9CA0FF role=text size=26 text="RESET ALL"
                    73 Button at=993,194,187,44 by=clip drawn focus heard="USE THIS SHADER" limit=88,186,1104,391 name=SNES.UseShader op paints=#55555AFF,#9C9CA0FF role=button text="Use This Shader"
                      74 TextBlock at=1010,200,153,32 box=153,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="USE THIS SHADER" layout=152.57,31.2 limit=993,194,187,44 owner=73 paints=#9C9CA0FF role=text size=26 text="USE THIS SHADER"
                75 MonoText at=652,244,528,50 box=528,50 by=clip caps=12.854 drawn face="Noto Sans" heard="Built into EmuSen. EmuSen's own, from the physics and measurements cited in EmuSen_CRT.md" layout=467.8,49.03 limit=88,186,1104,391 name=SNES.ShaderPath paints=#9C9CA0FF role=text size=18 text="Built into EmuSen. EmuSen's own, from the physics and measurements cited in EmuSen_CRT.md"
                76 HintText at=652,300,528,24 box=528,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="28 parameters." layout=102.98,24 limit=88,186,1104,391 paints=#86868AFF role=text size=20 text="28 parameters."
                77 FilterBar at=652,332,514,46 by=clip limit=88,186,1104,391 name=SNES.ParameterSearch paints=#9C9CA0FF role=group
                  78 TextBox at=652,332,514,46 drawn focus heard="Search parameters" name=PART_Search op paints=#0E0E10FF,#55555AFF,#D2D2D6FF part role=edit
                    79 ScrollViewer allow="a text box scrolls its own text" at=663,339,492,32 name=PART_ScrollViewer paints=#D2D2D6FF part role=scrollviewer
                      80 TextBlock allow="a text box scrolls its own text" at=663,339,492,32 box=492,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search parameters" in=79 layout=168.51,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text="Search parameters"
              81 SliderList at=652,386,528,191 by=clip limit=88,186,1104,391 name=SNES.ShaderParameters paints=#9C9CA0FF role=list
                82 ScrollViewer at=652,386,528,191 fades=b name=PART_ScrollViewer overflow=2441 paints=#1E1E1EFF,#333337FF,#4C9AE8FF,#55555AFF,#86868AFF,#9C9CA0FF,#EEEEF0FF part role=scrollviewer viewport=652,386,512,191
                  83 ContentPresenter at=652,386,498,94 by=scroll in=82 limit=652,386,512,191 role=none
                    84 SliderRow at=652,386,498,90 by=scroll in=82 limit=652,386,512,191 name=SNES.Parameter.quality role=none
                      85 StackPanel at=652,390,498,82 by=scroll in=82 limit=652,386,512,191 role=none
                        86 DockPanel at=652,390,498,30 by=scroll in=82 limit=652,386,512,191 role=none
                          87 Button at=1085,390,65,30 by=scroll drawn focus heard="Reset Quality" in=82 limit=652,386,512,191 op paints=#55555AFF,#9C9CA0FF role=button text=Reset
                            88 TextBlock at=1096,393,43,24 box=43,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=RESET in=82 layout=42.66,24 limit=1085,390,65,30 owner=87 paints=#9C9CA0FF role=text size=20 text=RESET
                          89 TextBlock at=998,390,75,29 box=75,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard=Balanced in=82 layout=74.69,28.8 limit=652,386,512,191 paints=#9C9CA0FF role=text size=24 text=Balanced
                          90 TextBlock at=652,390,334,29 box=334,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard=Quality in=82 layout=55.3,28.8 limit=652,386,512,191 paints=#9C9CA0FF role=text size=24 text=Quality
                        91 Slider at=652,422,498,50 by=scroll drawn focus heard=Quality in=82 limit=652,386,512,191 op paints=#333337FF,#4C9AE8FF,#EEEEF0FF role=slider sideways
                          92 RepeatButton at=652,437,239,20 drawn in=82 name=PART_DecreaseButton paints=#4C9AE8FF part role=button
                          93 RepeatButton at=911,437,239,20 drawn in=82 name=PART_IncreaseButton paints=#333337FF,#4C9AE8FF part role=button
                  94 ContentPresenter at=652,480,498,94 by=scroll in=82 limit=652,386,512,191 role=none
                    95 SliderRow at=652,480,498,90 by=scroll in=82 limit=652,386,512,191 name=SNES.Parameter.signal role=none
                      96 StackPanel at=652,484,498,82 by=scroll in=82 limit=652,386,512,191 role=none
                        97 DockPanel at=652,484,498,30 by=scroll in=82 limit=652,386,512,191 role=none
                          98 Button at=1085,484,65,30 by=scroll drawn focus heard="Reset Signal" in=82 limit=652,386,512,191 op paints=#55555AFF,#9C9CA0FF role=button text=Reset
                            99 TextBlock at=1096,487,43,24 box=43,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=RESET in=82 layout=42.66,24 limit=1085,484,65,30 owner=98 paints=#9C9CA0FF role=text size=20 text=RESET
                          100 TextBlock at=986,484,87,29 box=87,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard=Composite in=82 layout=86.3,28.8 limit=652,386,512,191 paints=#9C9CA0FF role=text size=24 text=Composite
                          101 TextBlock at=652,484,322,29 box=322,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard=Signal in=82 layout=49.32,28.8 limit=652,386,512,191 paints=#9C9CA0FF role=text size=24 text=Signal
                        102 Slider at=652,516,498,50 by=scroll drawn focus heard=Signal in=82 limit=652,386,512,191 op paints=#333337FF,#4C9AE8FF,#EEEEF0FF role=slider sideways
                          103 RepeatButton at=652,531,478,20 drawn in=82 name=PART_DecreaseButton paints=#4C9AE8FF part role=button
                  104 ContentPresenter at=652,574,498,94 by=scroll in=82 limit=652,386,512,191 role=none seen=652,574,498,3
                    105 SliderRow at=652,574,498,90 by=scroll in=82 limit=652,386,512,191 name=SNES.Parameter.standard paints=#333337FF,#4C9AE8FF,#55555AFF,#9C9CA0FF,#EEEEF0FF role=none seen=652,574,498,3
                  106 RepeatButton at=1164,386,16,16 drawn heard="Line up" name=PART_LineUpButton paints=#FFFFFFFF part role=button
                    107 ViewboxContainer at=1168,392,8,4.4 by=clip limit=1168,392,8,4.4 role=none
                  108 RepeatButton at=1164,442,16,119 drawn heard="Page down" name=PART_PageDownButton part role=button
                  109 RepeatButton at=1164,561,16,16 drawn heard="Line down" name=PART_LineDownButton paints=#FFFFFFFF part role=button
                    110 ViewboxContainer at=1168,567,8,4.4 by=clip limit=1168,567,8,4.4 role=none
        111 ButtonBar at=1102,136,90,44 by=panel limit=64,124,1152,465 paints=#9C9CA0FF role=toolbar
          112 Button at=1102,136,90,44 by=clip drawn focus heard=CLOSE limit=1102,136,90,44 op paints=#55555AFF,#9C9CA0FF role=button text=Close
            113 TextBlock at=1119,142,56,32 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=55.51,31.2 limit=1102,136,90,44 owner=112 paints=#EEEEF0FF role=text size=26 text=CLOSE
focus 112
edge 8 up same
edge 8 down 20
edge 20 up 8
edge 20 down 28
edge 23 up 8
edge 23 down 28
edge 23 left 20
edge 23 right 71
edge 28 up 23
edge 28 down 39
edge 28 left same
edge 28 right 91
edge 39 up 28
edge 39 down 45
edge 39 left same
edge 39 right 78
edge 45 up 39
edge 45 down 51
edge 45 left same
edge 45 right 87
edge 51 up 45
edge 51 down 57
edge 51 left same
edge 51 right 78
edge 57 up 51
edge 57 down 64
edge 57 left same
edge 57 right 98
edge 64 up 57
edge 64 down "item:81/2/0/0/1/Television standard"
edge 64 left same
edge 64 right 102
edge 71 up 112
edge 71 down 78
edge 71 left 23
edge 71 right 73
edge 73 up 112
edge 73 down 78
edge 73 left 71
edge 73 right same
edge 78 up 71
edge 78 down 87
edge 78 left 39
edge 78 right same
edge 87 up 78
edge 87 down 91
edge 87 left 45
edge 87 right same
edge 91 up 87
edge 91 down 98
edge 98 up 91
edge 98 down 102
edge 98 left 57
edge 98 right same
edge 102 up 98
edge 102 down "item:81/2/0/0/0/0/Reset Television standard"
edge 112 up same
edge 112 down 73
edge 112 left 8
edge 112 right same
edge "item:81/2/0/0/0/0/Reset Television standard" up 102
edge "item:81/2/0/0/0/0/Reset Television standard" down "item:81/2/0/0/1/Television standard"
edge "item:81/2/0/0/0/0/Reset Television standard" left 64
edge "item:81/2/0/0/0/0/Reset Television standard" right same
press pageup focus="Dropdown PART_Facet (Category)" tab=GB title=Shaders top=ShaderSettingsWindow
press pagedown focus="Dropdown PART_Facet (Category)" tab=SNES title=Shaders top=ShaderSettingsWindow
press first focus="Dropdown PART_Facet (Category)" tab=SNES title=Shaders top=ShaderSettingsWindow
press last focus="Dropdown PART_Facet (Category)" tab=SNES title=Shaders top=ShaderSettingsWindow
press back top=none
