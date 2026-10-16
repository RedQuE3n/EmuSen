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
              28 GroupedList at=100,248,528,262 by=clip limit=88,186,1104,391 name=SNES.ShaderList paints=#1B1B1EFF,#FFFFFFFF role=list
                29 ScrollViewer at=100,248,528,262 name=PART_ScrollViewer overflow=0 paints=#1E1E1EFF,#FFFFFFFF part role=scrollviewer viewport=100,248,528,262
                  30 ListBoxItem at=100,248,528,86 by=fixed-scroll drawn focus heard="Built into EmuSen: None" in=29 limit=100,248,528,262 op paints=#333337FF,#9C9CA0FF role=listitem
                    31 GroupedListRow at=100,248,528,85 by=clip in=29 limit=100,248,528,86 role=none
                      32 DockPanel at=110,262,508,24 by=clip in=29 limit=100,248,528,86 role=none
                        33 TextBlock at=609,262,9,24 box=9,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=5 in=29 layout=8.18,24 limit=100,248,528,86 owner=30 paints=#86868AFF role=text size=20 text=5
                        34 TextBlock at=110,262,491,24 box=491,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="BUILT INTO EMUSEN" in=29 layout=132.18,24 limit=100,248,528,86 owner=30 paints=#86868AFF role=text size=20 text="BUILT INTO EMUSEN"
                      35 Border at=104,291,520,41 by=clip in=29 limit=100,248,528,86 role=none
                        36 DockPanel at=114,297,500,29 by=clip in=29 limit=100,248,528,86 role=none
                          37 Border at=557,299,57,26 by=clip in=29 limit=100,248,528,86 paints=#0E0E10FF role=none
                            38 TextBlock at=565,300,41,24 box=41,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="In use" in=29 layout=40.88,24 limit=100,248,528,86 owner=30 paints=#9C9CA0FF role=text size=20 text="In use"
                          39 StackPanel at=114,297,435,29 by=clip in=29 limit=100,248,528,86 role=none
                            40 TextBlock at=114,297,435,29 box=435,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=None in=29 layout=41.74,28.8 limit=100,248,528,86 owner=30 paints=#9C9CA0FF role=text size=24 text=None
                  41 ListBoxItem at=100,334,528,44 by=fixed-scroll drawn focus heard=CRT in=29 limit=100,248,528,262 op paints=#333337FF,#9C9CA0FF role=listitem
                    42 GroupedListRow at=100,334,528,43 by=clip in=29 limit=100,334,528,44 role=none
                      43 Border at=104,335,520,41 by=clip in=29 limit=100,334,528,44 paints=#4C9AE8FF role=none
                        44 DockPanel at=114,341,500,29 by=clip in=29 limit=100,334,528,44 role=none
                          45 StackPanel at=114,341,500,29 by=clip in=29 limit=100,334,528,44 role=none
                            46 TextBlock at=114,341,500,29 box=500,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=CRT in=29 layout=31.49,28.8 limit=100,334,528,44 owner=41 paints=#FFFFFFFF role=text size=24 text=CRT
                  47 ListBoxItem at=100,378,528,44 by=fixed-scroll drawn focus heard="CRT (Lottes)" in=29 limit=100,248,528,262 op paints=#333337FF,#9C9CA0FF role=listitem
                    48 GroupedListRow at=100,378,528,43 by=clip in=29 limit=100,378,528,44 role=none
                      49 Border at=104,379,520,41 by=clip in=29 limit=100,378,528,44 role=none
                        50 DockPanel at=114,385,500,29 by=clip in=29 limit=100,378,528,44 role=none
                          51 StackPanel at=114,385,500,29 by=clip in=29 limit=100,378,528,44 role=none
                            52 TextBlock at=114,385,500,29 box=500,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="CRT (Lottes)" in=29 layout=97.92,28.8 limit=100,378,528,44 owner=47 paints=#9C9CA0FF role=text size=24 text="CRT (Lottes)"
                  53 ListBoxItem at=100,422,528,44 by=fixed-scroll drawn focus heard=Scanlines in=29 limit=100,248,528,262 op paints=#333337FF,#9C9CA0FF role=listitem
                    54 GroupedListRow at=100,422,528,43 by=clip in=29 limit=100,422,528,44 role=none
                      55 Border at=104,423,520,41 by=clip in=29 limit=100,422,528,44 role=none
                        56 DockPanel at=114,429,500,29 by=clip in=29 limit=100,422,528,44 role=none
                          57 StackPanel at=114,429,500,29 by=clip in=29 limit=100,422,528,44 role=none
                            58 TextBlock at=114,429,500,29 box=500,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Scanlines in=29 layout=77.93,28.8 limit=100,422,528,44 owner=53 paints=#9C9CA0FF role=text size=24 text=Scanlines
                  59 ListBoxItem at=100,466,528,44 by=fixed-scroll drawn focus heard="Simple CRT" in=29 limit=100,248,528,262 op paints=#333337FF,#9C9CA0FF role=listitem
                    60 GroupedListRow at=100,466,528,43 by=clip in=29 limit=100,466,528,44 role=none
                      61 Border at=104,467,520,41 by=clip in=29 limit=100,466,528,44 role=none
                        62 DockPanel at=114,473,500,29 by=clip in=29 limit=100,466,528,44 role=none
                          63 StackPanel at=114,473,500,29 by=clip in=29 limit=100,466,528,44 role=none
                            64 TextBlock at=114,473,500,29 box=500,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Simple CRT" in=29 layout=91.2,28.8 limit=100,466,528,44 owner=59 paints=#9C9CA0FF role=text size=24 text="Simple CRT"
              65 Border at=100,520,528,57 by=clip limit=88,186,1104,391 role=none
                66 Button at=100,520,182,44 by=clip drawn focus heard="DOWNLOAD PACK" limit=88,186,1104,391 name=SNES.DownloadPack op paints=#55555AFF,#9C9CA0FF role=button text="Download Pack"
                  67 TextBlock at=117,526,148,32 box=148,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="DOWNLOAD PACK" layout=147.39,31.2 limit=100,520,182,44 owner=66 paints=#9C9CA0FF role=text size=26 text="DOWNLOAD PACK"
            68 DockPanel at=652,194,528,383 by=clip limit=88,186,1104,391 role=none
              69 StackPanel at=652,194,528,184 by=clip limit=88,186,1104,391 role=none
                70 Grid at=652,194,528,44 by=clip limit=88,186,1104,391 role=none
                  71 TextBlock at=652,194,195,44 box=195,44 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=CRT layout=31.49,28.8 limit=88,186,1104,391 name=SNES.ShaderName paints=#9C9CA0FF role=text size=24 text=CRT
                  72 StackPanel at=859,194,321,44 by=clip limit=88,186,1104,391 role=none
                    73 Button at=859,194,126,44 by=clip drawn focus heard="RESET ALL" limit=88,186,1104,391 name=SNES.ResetShader op paints=#55555AFF,#9C9CA0FF role=button text="Reset All"
                      74 TextBlock at=876,200,92,32 box=92,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="RESET ALL" layout=91.91,31.2 limit=859,194,126,44 owner=73 paints=#9C9CA0FF role=text size=26 text="RESET ALL"
                    75 Button at=993,194,187,44 by=clip drawn focus heard="USE THIS SHADER" limit=88,186,1104,391 name=SNES.UseShader op paints=#55555AFF,#9C9CA0FF role=button text="Use This Shader"
                      76 TextBlock at=1010,200,153,32 box=153,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="USE THIS SHADER" layout=152.57,31.2 limit=993,194,187,44 owner=75 paints=#9C9CA0FF role=text size=26 text="USE THIS SHADER"
                77 MonoText at=652,244,528,50 box=528,50 by=clip caps=12.854 drawn face="Noto Sans" heard="Built into EmuSen. EmuSen's own, from the physics and measurements cited in EmuSen_CRT.md" layout=467.8,49.03 limit=88,186,1104,391 name=SNES.ShaderPath paints=#9C9CA0FF role=text size=18 text="Built into EmuSen. EmuSen's own, from the physics and measurements cited in EmuSen_CRT.md"
                78 HintText at=652,300,528,24 box=528,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="28 parameters." layout=102.98,24 limit=88,186,1104,391 paints=#86868AFF role=text size=20 text="28 parameters."
                79 FilterBar at=652,332,514,46 by=clip limit=88,186,1104,391 name=SNES.ParameterSearch paints=#9C9CA0FF role=group
                  80 TextBox at=652,332,514,46 drawn focus heard="Search parameters" name=PART_Search op paints=#0E0E10FF,#55555AFF,#D2D2D6FF part role=edit
                    81 ScrollViewer allow="a text box scrolls its own text" at=663,339,492,32 name=PART_ScrollViewer paints=#D2D2D6FF part role=scrollviewer
                      82 TextBlock allow="a text box scrolls its own text" at=663,339,492,32 box=492,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search parameters" in=81 layout=168.51,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text="Search parameters"
              83 SliderList at=652,386,528,191 by=clip limit=88,186,1104,391 name=SNES.ShaderParameters paints=#9C9CA0FF role=list
                84 ScrollViewer at=652,386,528,191 fades=b name=PART_ScrollViewer overflow=2441 paints=#1E1E1EFF,#333337FF,#4C9AE8FF,#55555AFF,#86868AFF,#9C9CA0FF,#EEEEF0FF part role=scrollviewer viewport=652,386,512,191
                  85 ContentPresenter at=652,386,498,94 by=scroll in=84 limit=652,386,512,191 role=none
                    86 SliderRow at=652,386,498,90 by=scroll in=84 limit=652,386,512,191 name=SNES.Parameter.quality role=none
                      87 StackPanel at=652,390,498,82 by=scroll in=84 limit=652,386,512,191 role=none
                        88 DockPanel at=652,390,498,30 by=scroll in=84 limit=652,386,512,191 role=none
                          89 Button at=1085,390,65,30 by=scroll drawn focus heard="Reset Quality" in=84 limit=652,386,512,191 op paints=#55555AFF,#9C9CA0FF role=button text=Reset
                            90 TextBlock at=1096,393,43,24 box=43,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=RESET in=84 layout=42.66,24 limit=1085,390,65,30 owner=89 paints=#9C9CA0FF role=text size=20 text=RESET
                          91 TextBlock at=998,390,75,29 box=75,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard=Balanced in=84 layout=74.69,28.8 limit=652,386,512,191 paints=#9C9CA0FF role=text size=24 text=Balanced
                          92 TextBlock at=652,390,334,29 box=334,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard=Quality in=84 layout=55.3,28.8 limit=652,386,512,191 paints=#9C9CA0FF role=text size=24 text=Quality
                        93 Slider at=652,422,498,50 by=scroll drawn focus heard=Quality in=84 limit=652,386,512,191 op paints=#333337FF,#4C9AE8FF,#EEEEF0FF role=slider sideways
                          94 RepeatButton at=652,437,239,20 drawn in=84 name=PART_DecreaseButton paints=#4C9AE8FF part role=button
                          95 RepeatButton at=911,437,239,20 drawn in=84 name=PART_IncreaseButton paints=#333337FF,#4C9AE8FF part role=button
                  96 ContentPresenter at=652,480,498,94 by=scroll in=84 limit=652,386,512,191 role=none
                    97 SliderRow at=652,480,498,90 by=scroll in=84 limit=652,386,512,191 name=SNES.Parameter.signal role=none
                      98 StackPanel at=652,484,498,82 by=scroll in=84 limit=652,386,512,191 role=none
                        99 DockPanel at=652,484,498,30 by=scroll in=84 limit=652,386,512,191 role=none
                          100 Button at=1085,484,65,30 by=scroll drawn focus heard="Reset Signal" in=84 limit=652,386,512,191 op paints=#55555AFF,#9C9CA0FF role=button text=Reset
                            101 TextBlock at=1096,487,43,24 box=43,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=RESET in=84 layout=42.66,24 limit=1085,484,65,30 owner=100 paints=#9C9CA0FF role=text size=20 text=RESET
                          102 TextBlock at=986,484,87,29 box=87,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard=Composite in=84 layout=86.3,28.8 limit=652,386,512,191 paints=#9C9CA0FF role=text size=24 text=Composite
                          103 TextBlock at=652,484,322,29 box=322,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard=Signal in=84 layout=49.32,28.8 limit=652,386,512,191 paints=#9C9CA0FF role=text size=24 text=Signal
                        104 Slider at=652,516,498,50 by=scroll drawn focus heard=Signal in=84 limit=652,386,512,191 op paints=#333337FF,#4C9AE8FF,#EEEEF0FF role=slider sideways
                          105 RepeatButton at=652,531,478,20 drawn in=84 name=PART_DecreaseButton paints=#4C9AE8FF part role=button
                  106 ContentPresenter at=652,574,498,94 by=scroll in=84 limit=652,386,512,191 role=none seen=652,574,498,3
                    107 SliderRow at=652,574,498,90 by=scroll in=84 limit=652,386,512,191 name=SNES.Parameter.standard paints=#333337FF,#4C9AE8FF,#55555AFF,#9C9CA0FF,#EEEEF0FF role=none seen=652,574,498,3
                  108 RepeatButton at=1164,386,16,16 drawn heard="Line up" name=PART_LineUpButton paints=#FFFFFFFF part role=button
                    109 ViewboxContainer at=1168,392,8,4.4 by=clip limit=1168,392,8,4.4 role=none
                  110 RepeatButton at=1164,442,16,119 drawn heard="Page down" name=PART_PageDownButton part role=button
                  111 RepeatButton at=1164,561,16,16 drawn heard="Line down" name=PART_LineDownButton paints=#FFFFFFFF part role=button
                    112 ViewboxContainer at=1168,567,8,4.4 by=clip limit=1168,567,8,4.4 role=none
        113 ButtonBar at=1102,136,90,44 by=panel limit=64,124,1152,465 paints=#9C9CA0FF role=toolbar
          114 Button at=1102,136,90,44 by=clip drawn focus heard=CLOSE limit=1102,136,90,44 op paints=#55555AFF,#9C9CA0FF role=button text=Close
            115 TextBlock at=1119,142,56,32 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=55.51,31.2 limit=1102,136,90,44 owner=114 paints=#EEEEF0FF role=text size=26 text=CLOSE
focus 114
edge 8 up same
edge 8 down 22
edge 22 up 8
edge 22 down 30
edge 25 up 8
edge 25 down 30
edge 25 left 22
edge 25 right 73
edge 30 up 25
edge 30 down 41
edge 30 left same
edge 30 right 93
edge 41 up 30
edge 41 down 47
edge 41 left same
edge 41 right 80
edge 47 up 41
edge 47 down 53
edge 47 left same
edge 47 right 89
edge 53 up 47
edge 53 down 59
edge 53 left same
edge 53 right 80
edge 59 up 53
edge 59 down 66
edge 59 left same
edge 59 right 100
edge 66 up 59
edge 66 down "item:83/2/0/0/1/Television standard"
edge 66 left same
edge 66 right 104
edge 73 up 8
edge 73 down 80
edge 73 left 25
edge 73 right 75
edge 75 up 114
edge 75 down 80
edge 75 left 73
edge 75 right same
edge 80 up 73
edge 80 down 89
edge 80 left 41
edge 80 right same
edge 89 up 80
edge 89 down 93
edge 89 left 47
edge 89 right same
edge 93 up 89
edge 93 down 100
edge 100 up 93
edge 100 down 104
edge 100 left 59
edge 100 right same
edge 104 up 100
edge 104 down "item:83/2/0/0/0/0/Reset Television standard"
edge 114 up same
edge 114 down 75
edge 114 left 8
edge 114 right same
edge "item:83/2/0/0/0/0/Reset Television standard" up 104
edge "item:83/2/0/0/0/0/Reset Television standard" down "item:83/2/0/0/1/Television standard"
edge "item:83/2/0/0/0/0/Reset Television standard" left 8
edge "item:83/2/0/0/0/0/Reset Television standard" right same
press pageup focus="Dropdown PART_Facet (Category)" tab=GB title=Shaders top=ShaderSettingsWindow
press pagedown focus="Dropdown PART_Facet (Category)" tab=SNES title=Shaders top=ShaderSettingsWindow
press first focus="Dropdown PART_Facet (Category)" tab=SNES title=Shaders top=ShaderSettingsWindow
press last focus="Dropdown PART_Facet (Category)" tab=SNES title=Shaders top=ShaderSettingsWindow
press back top=none
