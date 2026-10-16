# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ShaderSettingsCrt-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 96,37,1728,1002
title Shaders
footer "Drawn over the picture as it is shown, per console. A choice and every slider are saved at once and reach a running game of that console while its shader is in use. Not downloaded. libretro's slang pack, about 54 MB from buildbot.libretro.com, is the file RetroArch's updater fetches: some 2,600 presets. EmuSen ships none of it, and each shader keeps its authors' licence."
case ShaderSettingsCrt
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=96,187,1728,699 by=panel limit=96,187,1728,698 paints=#9C9CA0FF role=none seen=96,187,1728,698
    1 DockPanel at=132,205,1656,663 by=panel limit=96,187,1728,698 role=none
      2 Grid at=132,205,1656,663 by=panel limit=96,187,1728,698 role=none
        3 Tabs at=132,205,1656,663 by=panel limit=96,187,1728,698 name=ConsoleTabs paints=#9C9CA0FF role=tab
          4 TabItem at=132,205,97.5,72 by=clip drawn focus heard=NES limit=132,205,1656,663 paints=#86868AFF role=tabitem sideways text=NES
            5 TextBlock at=150,214,61.5,54 box=41,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=NES layout=40.14,36 limit=132,205,97.5,72 owner=4 paints=#86868AFF role=text size=30 text=NES
          6 TabItem at=229.5,205,76.5,72 by=clip drawn focus heard=GB limit=132,205,1656,663 paints=#86868AFF role=tabitem sideways text=GB
            7 TextBlock at=247.5,214,40.5,54 box=27,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GB layout=27,36 limit=229.5,205,76.5,72 owner=6 paints=#86868AFF role=text size=30 text=GB
          8 TabItem at=306,205,115.5,72 by=clip drawn focus heard=SNES limit=132,205,1656,663 op paints=#EEEEF0FF role=tabitem sideways text=SNES
            9 TextBlock at=324,214,79.5,54 box=53,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=SNES layout=52.41,36 limit=306,205,115.5,72 owner=8 paints=#EEEEF0FF role=text size=30 text=SNES
          10 TabItem at=421.5,205,96,72 by=clip drawn focus heard=N64 limit=132,205,1656,663 paints=#86868AFF role=tabitem sideways text=N64
            11 TextBlock at=439.5,214,60,54 box=40,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=N64 layout=39.54,36 limit=421.5,205,96,72 owner=10 paints=#86868AFF role=text size=30 text=N64
          12 TabItem at=517.5,205,165,72 by=clip drawn focus heard=GENESIS limit=132,205,1656,663 paints=#86868AFF role=tabitem sideways text=Genesis
            13 TextBlock at=535.5,214,129,54 box=86,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GENESIS layout=85.47,36 limit=517.5,205,165,72 owner=12 paints=#86868AFF role=text size=30 text=GENESIS
          14 TabItem at=682.5,205,163.5,72 by=clip drawn focus heard="SEGA CD" limit=132,205,1656,663 paints=#86868AFF role=tabitem sideways text="Sega CD"
            15 TextBlock at=700.5,214,127.5,54 box=85,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard="SEGA CD" layout=84.18,36 limit=682.5,205,163.5,72 owner=14 paints=#86868AFF role=text size=30 text="SEGA CD"
          16 TabItem at=846,205,234,72 by=clip drawn focus heard=PLAYSTATION limit=132,205,1656,663 paints=#86868AFF role=tabitem sideways text=PlayStation
            17 TextBlock at=864,214,198,54 box=132,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=PLAYSTATION layout=131.91,36 limit=846,205,234,72 owner=16 paints=#86868AFF role=text size=30 text=PLAYSTATION
          18 ShaderPanel at=150,292,1620,576 by=clip limit=132,280,1656,588 role=none
            19 Grid at=150,292,792,576 by=clip limit=132,280,1656,588 role=none
              20 FilterBar at=150,292,792,69 by=clip limit=132,280,1656,588 name=SNES.ShaderSearch paints=#9C9CA0FF role=group
                21 TextBlock at=150,304,109.5,43.5 box=73,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Category layout=72.82,28.8 limit=150,292,792,69 paints=#86868AFF part role=text size=24 text=Category
                22 Dropdown at=271.5,292,270,66 drawn focus heard=Category name=PART_Facet op paints=#0E0E10FF,#55555AFF,#9C9CA0FF part role=combobox sideways
                  23 TextBlock at=289.5,299.5,204,48 box=136,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=All layout=20.07,31.2 limit=289.5,299.5,204,48 owner=22 paints=#9C9CA0FF role=text size=26 text=All
                  24 ViewboxContainer at=508.5,320.1,18,9.81 by=clip limit=508.5,320.1,18,9.81 nohit role=none
                25 TextBox at=553.5,292,388.5,69 drawn focus heard=Search name=PART_Search op paints=#0E0E10FF,#55555AFF,#D2D2D6FF part role=edit
                  26 ScrollViewer allow="a text box scrolls its own text" at=570,302.5,355.5,48 name=PART_ScrollViewer paints=#D2D2D6FF part role=scrollviewer
                    27 TextBlock allow="a text box scrolls its own text" at=570,302.5,355.5,48 box=237,32 caps=18.1987 drawn face="Barlow Condensed" heard=Search in=26 layout=61,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text=Search
              28 GroupedList at=150,373,792,393 by=clip limit=132,280,1656,588 name=SNES.ShaderList paints=#1B1B1EFF,#FFFFFFFF role=list
                29 ScrollViewer at=150,373,792,393 name=PART_ScrollViewer overflow=0 paints=#1E1E1EFF,#FFFFFFFF part role=scrollviewer viewport=150,373,792,393
                  30 ListBoxItem at=150,373,792,129 by=fixed-scroll drawn focus heard="Built into EmuSen: None" in=29 limit=150,373,792,393 op paints=#333337FF,#9C9CA0FF role=listitem
                    31 GroupedListRow at=150,373,792,127.5 by=clip in=29 limit=150,373,792,129 role=none
                      32 DockPanel at=165,394,762,36 by=clip in=29 limit=150,373,792,129 role=none
                        33 TextBlock at=913.5,394,13.5,36 box=9,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=5 in=29 layout=8.18,24 limit=150,373,792,129 owner=30 paints=#86868AFF role=text size=20 text=5
                        34 TextBlock at=165,394,736.5,36 box=491,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="BUILT INTO EMUSEN" in=29 layout=132.18,24 limit=150,373,792,129 owner=30 paints=#86868AFF role=text size=20 text="BUILT INTO EMUSEN"
                      35 Border at=156,437.5,780,61.5 by=clip in=29 limit=150,373,792,129 role=none
                        36 DockPanel at=171,446.5,750,43.5 by=clip in=29 limit=150,373,792,129 role=none
                          37 Border at=835.5,449.5,85.5,39 by=clip in=29 limit=150,373,792,129 paints=#0E0E10FF role=none
                            38 TextBlock at=847.5,451,61.5,36 box=41,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="In use" in=29 layout=40.88,24 limit=150,373,792,129 owner=30 paints=#9C9CA0FF role=text size=20 text="In use"
                          39 StackPanel at=171,446.5,652.5,43.5 by=clip in=29 limit=150,373,792,129 role=none
                            40 TextBlock at=171,446.5,652.5,43.5 box=435,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=None in=29 layout=41.74,28.8 limit=150,373,792,129 owner=30 paints=#9C9CA0FF role=text size=24 text=None
                  41 ListBoxItem at=150,502,792,66 by=fixed-scroll drawn focus heard=CRT in=29 limit=150,373,792,393 op paints=#333337FF,#9C9CA0FF role=listitem
                    42 GroupedListRow at=150,502,792,64.5 by=clip in=29 limit=150,502,792,66 role=none
                      43 Border at=156,503.5,780,61.5 by=clip in=29 limit=150,502,792,66 paints=#4C9AE8FF role=none
                        44 DockPanel at=171,512.5,750,43.5 by=clip in=29 limit=150,502,792,66 role=none
                          45 StackPanel at=171,512.5,750,43.5 by=clip in=29 limit=150,502,792,66 role=none
                            46 TextBlock at=171,512.5,750,43.5 box=500,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=CRT in=29 layout=31.49,28.8 limit=150,502,792,66 owner=41 paints=#FFFFFFFF role=text size=24 text=CRT
                  47 ListBoxItem at=150,568,792,66 by=fixed-scroll drawn focus heard="CRT (Lottes)" in=29 limit=150,373,792,393 op paints=#333337FF,#9C9CA0FF role=listitem
                    48 GroupedListRow at=150,568,792,64.5 by=clip in=29 limit=150,568,792,66 role=none
                      49 Border at=156,569.5,780,61.5 by=clip in=29 limit=150,568,792,66 role=none
                        50 DockPanel at=171,578.5,750,43.5 by=clip in=29 limit=150,568,792,66 role=none
                          51 StackPanel at=171,578.5,750,43.5 by=clip in=29 limit=150,568,792,66 role=none
                            52 TextBlock at=171,578.5,750,43.5 box=500,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="CRT (Lottes)" in=29 layout=97.92,28.8 limit=150,568,792,66 owner=47 paints=#9C9CA0FF role=text size=24 text="CRT (Lottes)"
                  53 ListBoxItem at=150,634,792,66 by=fixed-scroll drawn focus heard=Scanlines in=29 limit=150,373,792,393 op paints=#333337FF,#9C9CA0FF role=listitem
                    54 GroupedListRow at=150,634,792,64.5 by=clip in=29 limit=150,634,792,66 role=none
                      55 Border at=156,635.5,780,61.5 by=clip in=29 limit=150,634,792,66 role=none
                        56 DockPanel at=171,644.5,750,43.5 by=clip in=29 limit=150,634,792,66 role=none
                          57 StackPanel at=171,644.5,750,43.5 by=clip in=29 limit=150,634,792,66 role=none
                            58 TextBlock at=171,644.5,750,43.5 box=500,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Scanlines in=29 layout=77.93,28.8 limit=150,634,792,66 owner=53 paints=#9C9CA0FF role=text size=24 text=Scanlines
                  59 ListBoxItem at=150,700,792,66 by=fixed-scroll drawn focus heard="Simple CRT" in=29 limit=150,373,792,393 op paints=#333337FF,#9C9CA0FF role=listitem
                    60 GroupedListRow at=150,700,792,64.5 by=clip in=29 limit=150,700,792,66 role=none
                      61 Border at=156,701.5,780,61.5 by=clip in=29 limit=150,700,792,66 role=none
                        62 DockPanel at=171,710.5,750,43.5 by=clip in=29 limit=150,700,792,66 role=none
                          63 StackPanel at=171,710.5,750,43.5 by=clip in=29 limit=150,700,792,66 role=none
                            64 TextBlock at=171,710.5,750,43.5 box=500,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Simple CRT" in=29 layout=91.2,28.8 limit=150,700,792,66 owner=59 paints=#9C9CA0FF role=text size=24 text="Simple CRT"
              65 Border at=150,781,792,87 by=clip limit=132,280,1656,588 role=none
                66 Button at=150,781,273,66 by=clip drawn focus heard="DOWNLOAD PACK" limit=132,280,1656,588 name=SNES.DownloadPack op paints=#55555AFF,#9C9CA0FF role=button text="Download Pack"
                  67 TextBlock at=175.5,790,222,48 box=148,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="DOWNLOAD PACK" layout=147.39,31.2 limit=150,781,273,66 owner=66 paints=#9C9CA0FF role=text size=26 text="DOWNLOAD PACK"
            68 DockPanel at=978,292,792,576 by=clip limit=132,280,1656,588 role=none
              69 StackPanel at=978,292,792,276 by=clip limit=132,280,1656,588 role=none
                70 Grid at=978,292,792,66 by=clip limit=132,280,1656,588 role=none
                  71 TextBlock at=978,292,292.5,66 box=195,44 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=CRT layout=31.49,28.8 limit=132,280,1656,588 name=SNES.ShaderName paints=#9C9CA0FF role=text size=24 text=CRT
                  72 StackPanel at=1288.5,292,481.5,66 by=clip limit=132,280,1656,588 role=none
                    73 Button at=1288.5,292,189,66 by=clip drawn focus heard="RESET ALL" limit=132,280,1656,588 name=SNES.ResetShader op paints=#55555AFF,#9C9CA0FF role=button text="Reset All"
                      74 TextBlock at=1314,301,138,48 box=92,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="RESET ALL" layout=91.91,31.2 limit=1288.5,292,189,66 owner=73 paints=#9C9CA0FF role=text size=26 text="RESET ALL"
                    75 Button at=1489.5,292,280.5,66 by=clip drawn focus heard="USE THIS SHADER" limit=132,280,1656,588 name=SNES.UseShader op paints=#55555AFF,#9C9CA0FF role=button text="Use This Shader"
                      76 TextBlock at=1515,301,229.5,48 box=153,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="USE THIS SHADER" layout=152.57,31.2 limit=1489.5,292,280.5,66 owner=75 paints=#9C9CA0FF role=text size=26 text="USE THIS SHADER"
                77 MonoText at=978,367,792,75 box=528,50 by=clip caps=12.854 drawn face="Noto Sans" heard="Built into EmuSen. EmuSen's own, from the physics and measurements cited in EmuSen_CRT.md" layout=467.8,49.03 limit=132,280,1656,588 name=SNES.ShaderPath paints=#9C9CA0FF role=text size=18 text="Built into EmuSen. EmuSen's own, from the physics and measurements cited in EmuSen_CRT.md"
                78 HintText at=978,451,792,36 box=528,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="28 parameters." layout=102.98,24 limit=132,280,1656,588 paints=#86868AFF role=text size=20 text="28 parameters."
                79 FilterBar at=978,499,771,69 by=clip limit=132,280,1656,588 name=SNES.ParameterSearch paints=#9C9CA0FF role=group
                  80 TextBox at=978,499,771,69 drawn focus heard="Search parameters" name=PART_Search op paints=#0E0E10FF,#55555AFF,#D2D2D6FF part role=edit
                    81 ScrollViewer allow="a text box scrolls its own text" at=994.5,509.5,738,48 name=PART_ScrollViewer paints=#D2D2D6FF part role=scrollviewer
                      82 TextBlock allow="a text box scrolls its own text" at=994.5,509.5,738,48 box=492,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search parameters" in=81 layout=168.51,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text="Search parameters"
              83 SliderList at=978,580,792,288 by=clip limit=132,280,1656,588 name=SNES.ShaderParameters paints=#9C9CA0FF role=list
                84 ScrollViewer at=978,580,792,288 fades=b name=PART_ScrollViewer overflow=3660 paints=#1E1E1EFF,#333337FF,#4C9AE8FF,#55555AFF,#86868AFF,#9C9CA0FF,#EEEEF0FF part role=scrollviewer viewport=978,580,768,288
                  85 ContentPresenter at=978,580,747,141 by=scroll in=84 limit=978,580,768,288 role=none
                    86 SliderRow at=978,580,747,135 by=scroll in=84 limit=978,580,768,288 name=SNES.Parameter.quality role=none
                      87 StackPanel at=978,586,747,123 by=scroll in=84 limit=978,580,768,288 role=none
                        88 DockPanel at=978,586,747,45 by=scroll in=84 limit=978,580,768,288 role=none
                          89 Button at=1627.5,586,97.5,45 by=scroll drawn focus heard="Reset Quality" in=84 limit=978,580,768,288 op paints=#55555AFF,#9C9CA0FF role=button text=Reset
                            90 TextBlock at=1644,590.5,64.5,36 box=43,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=RESET in=84 layout=42.66,24 limit=1627.5,586,97.5,45 owner=89 paints=#9C9CA0FF role=text size=20 text=RESET
                          91 TextBlock at=1497,586,112.5,43.5 box=75,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard=Balanced in=84 layout=74.69,28.8 limit=978,580,768,288 paints=#9C9CA0FF role=text size=24 text=Balanced
                          92 TextBlock at=978,586,501,43.5 box=334,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard=Quality in=84 layout=55.3,28.8 limit=978,580,768,288 paints=#9C9CA0FF role=text size=24 text=Quality
                        93 Slider at=978,634,747,75 by=scroll drawn focus heard=Quality in=84 limit=978,580,768,288 op paints=#333337FF,#4C9AE8FF,#EEEEF0FF role=slider sideways
                          94 RepeatButton at=978,656.5,358.5,30 drawn in=84 name=PART_DecreaseButton paints=#4C9AE8FF part role=button
                          95 RepeatButton at=1366.5,656.5,358.5,30 drawn in=84 name=PART_IncreaseButton paints=#333337FF,#4C9AE8FF part role=button
                  96 ContentPresenter at=978,721,747,141 by=scroll in=84 limit=978,580,768,288 role=none
                    97 SliderRow at=978,721,747,135 by=scroll in=84 limit=978,580,768,288 name=SNES.Parameter.signal role=none
                      98 StackPanel at=978,727,747,123 by=scroll in=84 limit=978,580,768,288 role=none
                        99 DockPanel at=978,727,747,45 by=scroll in=84 limit=978,580,768,288 role=none
                          100 Button at=1627.5,727,97.5,45 by=scroll drawn focus heard="Reset Signal" in=84 limit=978,580,768,288 op paints=#55555AFF,#9C9CA0FF role=button text=Reset
                            101 TextBlock at=1644,731.5,64.5,36 box=43,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=RESET in=84 layout=42.66,24 limit=1627.5,727,97.5,45 owner=100 paints=#9C9CA0FF role=text size=20 text=RESET
                          102 TextBlock at=1479,727,130.5,43.5 box=87,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard=Composite in=84 layout=86.3,28.8 limit=978,580,768,288 paints=#9C9CA0FF role=text size=24 text=Composite
                          103 TextBlock at=978,727,483,43.5 box=322,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard=Signal in=84 layout=49.32,28.8 limit=978,580,768,288 paints=#9C9CA0FF role=text size=24 text=Signal
                        104 Slider at=978,775,747,75 by=scroll drawn focus heard=Signal in=84 limit=978,580,768,288 op paints=#333337FF,#4C9AE8FF,#EEEEF0FF role=slider sideways
                          105 RepeatButton at=978,797.5,717,30 drawn in=84 name=PART_DecreaseButton paints=#4C9AE8FF part role=button
                  106 ContentPresenter at=978,862,747,141 by=scroll in=84 limit=978,580,768,288 role=none seen=978,862,747,6
                    107 SliderRow at=978,862,747,135 by=scroll in=84 limit=978,580,768,288 name=SNES.Parameter.standard paints=#333337FF,#4C9AE8FF,#55555AFF,#9C9CA0FF,#EEEEF0FF role=none seen=978,862,747,6
                  108 RepeatButton at=1746,580,24,24 drawn heard="Line up" name=PART_LineUpButton paints=#FFFFFFFF part role=button
                    109 ViewboxContainer at=1752,589,12,6.6 by=clip limit=1752,589,12,6.6 role=none
                  110 RepeatButton at=1746,664,24,180 drawn heard="Page down" name=PART_PageDownButton part role=button
                  111 RepeatButton at=1746,844,24,24 drawn heard="Line down" name=PART_LineDownButton paints=#FFFFFFFF part role=button
                    112 ViewboxContainer at=1752,853,12,6.6 by=clip limit=1752,853,12,6.6 role=none
        113 ButtonBar at=1653,205,135,66 by=panel limit=96,187,1728,698 paints=#9C9CA0FF role=toolbar
          114 Button at=1653,205,135,66 by=clip drawn focus heard=CLOSE limit=1653,205,135,66 op paints=#55555AFF,#9C9CA0FF role=button text=Close
            115 TextBlock at=1678.5,214,84,48 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=55.51,31.2 limit=1653,205,135,66 owner=114 paints=#EEEEF0FF role=text size=26 text=CLOSE
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
edge 80 right 89
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
