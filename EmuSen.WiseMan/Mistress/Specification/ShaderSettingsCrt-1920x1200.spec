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
palette #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
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
          14 TabItem at=682.5,205,234,72 by=clip drawn focus heard=PLAYSTATION limit=132,205,1656,663 paints=#86868AFF role=tabitem sideways text=PlayStation
            15 TextBlock at=700.5,214,198,54 box=132,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=PLAYSTATION layout=131.91,36 limit=682.5,205,234,72 owner=14 paints=#86868AFF role=text size=30 text=PLAYSTATION
          16 ShaderPanel at=150,292,1620,576 by=clip limit=132,280,1656,588 role=none
            17 Grid at=150,292,792,576 by=clip limit=132,280,1656,588 role=none
              18 FilterBar at=150,292,792,69 by=clip limit=132,280,1656,588 name=SNES.ShaderSearch paints=#9C9CA0FF role=group
                19 TextBlock at=150,304,109.5,43.5 box=73,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Category layout=72.82,28.8 limit=150,292,792,69 paints=#86868AFF part role=text size=24 text=Category
                20 Dropdown at=271.5,292,270,66 drawn focus heard=Category name=PART_Facet op paints=#0E0E10FF,#55555AFF,#9C9CA0FF part role=combobox sideways
                  21 TextBlock at=289.5,299.5,204,48 box=136,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=All layout=20.07,31.2 limit=289.5,299.5,204,48 owner=20 paints=#9C9CA0FF role=text size=26 text=All
                  22 ViewboxContainer at=508.5,320.1,18,9.81 by=clip limit=508.5,320.1,18,9.81 nohit role=none
                23 TextBox at=553.5,292,388.5,69 drawn focus heard=Search name=PART_Search op paints=#0E0E10FF,#55555AFF,#D2D2D6FF part role=edit
                  24 ScrollViewer allow="a text box scrolls its own text" at=570,302.5,355.5,48 name=PART_ScrollViewer paints=#D2D2D6FF part role=scrollviewer
                    25 TextBlock allow="a text box scrolls its own text" at=570,302.5,355.5,48 box=237,32 caps=18.1987 drawn face="Barlow Condensed" heard=Search in=24 layout=61,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text=Search
              26 GroupedList at=150,373,792,393 by=clip limit=132,280,1656,588 name=SNES.ShaderList paints=#1B1B1EFF,#FFFFFFFF role=list
                27 ScrollViewer at=150,373,792,393 name=PART_ScrollViewer overflow=0 paints=#1E1E1EFF,#FFFFFFFF part role=scrollviewer viewport=150,373,792,393
                  28 ListBoxItem at=150,373,792,129 by=fixed-scroll drawn focus heard="Built into EmuSen: None" in=27 limit=150,373,792,393 op paints=#333337FF,#9C9CA0FF role=listitem
                    29 GroupedListRow at=150,373,792,127.5 by=clip in=27 limit=150,373,792,129 role=none
                      30 DockPanel at=165,394,762,36 by=clip in=27 limit=150,373,792,129 role=none
                        31 TextBlock at=913.5,394,13.5,36 box=9,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=5 in=27 layout=8.18,24 limit=150,373,792,129 owner=28 paints=#86868AFF role=text size=20 text=5
                        32 TextBlock at=165,394,736.5,36 box=491,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="BUILT INTO EMUSEN" in=27 layout=132.18,24 limit=150,373,792,129 owner=28 paints=#86868AFF role=text size=20 text="BUILT INTO EMUSEN"
                      33 Border at=156,437.5,780,61.5 by=clip in=27 limit=150,373,792,129 role=none
                        34 DockPanel at=171,446.5,750,43.5 by=clip in=27 limit=150,373,792,129 role=none
                          35 Border at=835.5,449.5,85.5,39 by=clip in=27 limit=150,373,792,129 paints=#0E0E10FF role=none
                            36 TextBlock at=847.5,451,61.5,36 box=41,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="In use" in=27 layout=40.88,24 limit=150,373,792,129 owner=28 paints=#9C9CA0FF role=text size=20 text="In use"
                          37 StackPanel at=171,446.5,652.5,43.5 by=clip in=27 limit=150,373,792,129 role=none
                            38 TextBlock at=171,446.5,652.5,43.5 box=435,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=None in=27 layout=41.74,28.8 limit=150,373,792,129 owner=28 paints=#9C9CA0FF role=text size=24 text=None
                  39 ListBoxItem at=150,502,792,66 by=fixed-scroll drawn focus heard=CRT in=27 limit=150,373,792,393 op paints=#333337FF,#9C9CA0FF role=listitem
                    40 GroupedListRow at=150,502,792,64.5 by=clip in=27 limit=150,502,792,66 role=none
                      41 Border at=156,503.5,780,61.5 by=clip in=27 limit=150,502,792,66 paints=#4C9AE8FF role=none
                        42 DockPanel at=171,512.5,750,43.5 by=clip in=27 limit=150,502,792,66 role=none
                          43 StackPanel at=171,512.5,750,43.5 by=clip in=27 limit=150,502,792,66 role=none
                            44 TextBlock at=171,512.5,750,43.5 box=500,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=CRT in=27 layout=31.49,28.8 limit=150,502,792,66 owner=39 paints=#FFFFFFFF role=text size=24 text=CRT
                  45 ListBoxItem at=150,568,792,66 by=fixed-scroll drawn focus heard="CRT (Lottes)" in=27 limit=150,373,792,393 op paints=#333337FF,#9C9CA0FF role=listitem
                    46 GroupedListRow at=150,568,792,64.5 by=clip in=27 limit=150,568,792,66 role=none
                      47 Border at=156,569.5,780,61.5 by=clip in=27 limit=150,568,792,66 role=none
                        48 DockPanel at=171,578.5,750,43.5 by=clip in=27 limit=150,568,792,66 role=none
                          49 StackPanel at=171,578.5,750,43.5 by=clip in=27 limit=150,568,792,66 role=none
                            50 TextBlock at=171,578.5,750,43.5 box=500,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="CRT (Lottes)" in=27 layout=97.92,28.8 limit=150,568,792,66 owner=45 paints=#9C9CA0FF role=text size=24 text="CRT (Lottes)"
                  51 ListBoxItem at=150,634,792,66 by=fixed-scroll drawn focus heard=Scanlines in=27 limit=150,373,792,393 op paints=#333337FF,#9C9CA0FF role=listitem
                    52 GroupedListRow at=150,634,792,64.5 by=clip in=27 limit=150,634,792,66 role=none
                      53 Border at=156,635.5,780,61.5 by=clip in=27 limit=150,634,792,66 role=none
                        54 DockPanel at=171,644.5,750,43.5 by=clip in=27 limit=150,634,792,66 role=none
                          55 StackPanel at=171,644.5,750,43.5 by=clip in=27 limit=150,634,792,66 role=none
                            56 TextBlock at=171,644.5,750,43.5 box=500,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Scanlines in=27 layout=77.93,28.8 limit=150,634,792,66 owner=51 paints=#9C9CA0FF role=text size=24 text=Scanlines
                  57 ListBoxItem at=150,700,792,66 by=fixed-scroll drawn focus heard="Simple CRT" in=27 limit=150,373,792,393 op paints=#333337FF,#9C9CA0FF role=listitem
                    58 GroupedListRow at=150,700,792,64.5 by=clip in=27 limit=150,700,792,66 role=none
                      59 Border at=156,701.5,780,61.5 by=clip in=27 limit=150,700,792,66 role=none
                        60 DockPanel at=171,710.5,750,43.5 by=clip in=27 limit=150,700,792,66 role=none
                          61 StackPanel at=171,710.5,750,43.5 by=clip in=27 limit=150,700,792,66 role=none
                            62 TextBlock at=171,710.5,750,43.5 box=500,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Simple CRT" in=27 layout=91.2,28.8 limit=150,700,792,66 owner=57 paints=#9C9CA0FF role=text size=24 text="Simple CRT"
              63 Border at=150,781,792,87 by=clip limit=132,280,1656,588 role=none
                64 Button at=150,781,273,66 by=clip drawn focus heard="DOWNLOAD PACK" limit=132,280,1656,588 name=SNES.DownloadPack op paints=#55555AFF,#9C9CA0FF role=button text="Download Pack"
                  65 TextBlock at=175.5,790,222,48 box=148,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="DOWNLOAD PACK" layout=147.39,31.2 limit=150,781,273,66 owner=64 paints=#9C9CA0FF role=text size=26 text="DOWNLOAD PACK"
            66 DockPanel at=978,292,792,576 by=clip limit=132,280,1656,588 role=none
              67 StackPanel at=978,292,792,276 by=clip limit=132,280,1656,588 role=none
                68 Grid at=978,292,792,66 by=clip limit=132,280,1656,588 role=none
                  69 TextBlock at=978,292,292.5,66 box=195,44 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=CRT layout=31.49,28.8 limit=132,280,1656,588 name=SNES.ShaderName paints=#9C9CA0FF role=text size=24 text=CRT
                  70 StackPanel at=1288.5,292,481.5,66 by=clip limit=132,280,1656,588 role=none
                    71 Button at=1288.5,292,189,66 by=clip drawn focus heard="RESET ALL" limit=132,280,1656,588 name=SNES.ResetShader op paints=#55555AFF,#9C9CA0FF role=button text="Reset All"
                      72 TextBlock at=1314,301,138,48 box=92,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="RESET ALL" layout=91.91,31.2 limit=1288.5,292,189,66 owner=71 paints=#9C9CA0FF role=text size=26 text="RESET ALL"
                    73 Button at=1489.5,292,280.5,66 by=clip drawn focus heard="USE THIS SHADER" limit=132,280,1656,588 name=SNES.UseShader op paints=#55555AFF,#9C9CA0FF role=button text="Use This Shader"
                      74 TextBlock at=1515,301,229.5,48 box=153,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="USE THIS SHADER" layout=152.57,31.2 limit=1489.5,292,280.5,66 owner=73 paints=#9C9CA0FF role=text size=26 text="USE THIS SHADER"
                75 MonoText at=978,367,792,75 box=528,50 by=clip caps=12.854 drawn face="Noto Sans" heard="Built into EmuSen. EmuSen's own, from the physics and measurements cited in EmuSen_CRT.md" layout=467.8,49.03 limit=132,280,1656,588 name=SNES.ShaderPath paints=#9C9CA0FF role=text size=18 text="Built into EmuSen. EmuSen's own, from the physics and measurements cited in EmuSen_CRT.md"
                76 HintText at=978,451,792,36 box=528,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="28 parameters." layout=102.98,24 limit=132,280,1656,588 paints=#86868AFF role=text size=20 text="28 parameters."
                77 FilterBar at=978,499,771,69 by=clip limit=132,280,1656,588 name=SNES.ParameterSearch paints=#9C9CA0FF role=group
                  78 TextBox at=978,499,771,69 drawn focus heard="Search parameters" name=PART_Search op paints=#0E0E10FF,#55555AFF,#D2D2D6FF part role=edit
                    79 ScrollViewer allow="a text box scrolls its own text" at=994.5,509.5,738,48 name=PART_ScrollViewer paints=#D2D2D6FF part role=scrollviewer
                      80 TextBlock allow="a text box scrolls its own text" at=994.5,509.5,738,48 box=492,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search parameters" in=79 layout=168.51,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text="Search parameters"
              81 SliderList at=978,580,792,288 by=clip limit=132,280,1656,588 name=SNES.ShaderParameters paints=#9C9CA0FF role=list
                82 ScrollViewer at=978,580,792,288 fades=b name=PART_ScrollViewer overflow=3660 paints=#1E1E1EFF,#333337FF,#4C9AE8FF,#55555AFF,#86868AFF,#9C9CA0FF,#EEEEF0FF part role=scrollviewer viewport=978,580,768,288
                  83 ContentPresenter at=978,580,747,141 by=scroll in=82 limit=978,580,768,288 role=none
                    84 SliderRow at=978,580,747,135 by=scroll in=82 limit=978,580,768,288 name=SNES.Parameter.quality role=none
                      85 StackPanel at=978,586,747,123 by=scroll in=82 limit=978,580,768,288 role=none
                        86 DockPanel at=978,586,747,45 by=scroll in=82 limit=978,580,768,288 role=none
                          87 Button at=1627.5,586,97.5,45 by=scroll drawn focus heard="Reset Quality" in=82 limit=978,580,768,288 op paints=#55555AFF,#9C9CA0FF role=button text=Reset
                            88 TextBlock at=1644,590.5,64.5,36 box=43,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=RESET in=82 layout=42.66,24 limit=1627.5,586,97.5,45 owner=87 paints=#9C9CA0FF role=text size=20 text=RESET
                          89 TextBlock at=1497,586,112.5,43.5 box=75,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard=Balanced in=82 layout=74.69,28.8 limit=978,580,768,288 paints=#9C9CA0FF role=text size=24 text=Balanced
                          90 TextBlock at=978,586,501,43.5 box=334,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard=Quality in=82 layout=55.3,28.8 limit=978,580,768,288 paints=#9C9CA0FF role=text size=24 text=Quality
                        91 Slider at=978,634,747,75 by=scroll drawn focus heard=Quality in=82 limit=978,580,768,288 op paints=#333337FF,#4C9AE8FF,#EEEEF0FF role=slider sideways
                          92 RepeatButton at=978,656.5,358.5,30 drawn in=82 name=PART_DecreaseButton paints=#4C9AE8FF part role=button
                          93 RepeatButton at=1366.5,656.5,358.5,30 drawn in=82 name=PART_IncreaseButton paints=#333337FF,#4C9AE8FF part role=button
                  94 ContentPresenter at=978,721,747,141 by=scroll in=82 limit=978,580,768,288 role=none
                    95 SliderRow at=978,721,747,135 by=scroll in=82 limit=978,580,768,288 name=SNES.Parameter.signal role=none
                      96 StackPanel at=978,727,747,123 by=scroll in=82 limit=978,580,768,288 role=none
                        97 DockPanel at=978,727,747,45 by=scroll in=82 limit=978,580,768,288 role=none
                          98 Button at=1627.5,727,97.5,45 by=scroll drawn focus heard="Reset Signal" in=82 limit=978,580,768,288 op paints=#55555AFF,#9C9CA0FF role=button text=Reset
                            99 TextBlock at=1644,731.5,64.5,36 box=43,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=RESET in=82 layout=42.66,24 limit=1627.5,727,97.5,45 owner=98 paints=#9C9CA0FF role=text size=20 text=RESET
                          100 TextBlock at=1479,727,130.5,43.5 box=87,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard=Composite in=82 layout=86.3,28.8 limit=978,580,768,288 paints=#9C9CA0FF role=text size=24 text=Composite
                          101 TextBlock at=978,727,483,43.5 box=322,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard=Signal in=82 layout=49.32,28.8 limit=978,580,768,288 paints=#9C9CA0FF role=text size=24 text=Signal
                        102 Slider at=978,775,747,75 by=scroll drawn focus heard=Signal in=82 limit=978,580,768,288 op paints=#333337FF,#4C9AE8FF,#EEEEF0FF role=slider sideways
                          103 RepeatButton at=978,797.5,717,30 drawn in=82 name=PART_DecreaseButton paints=#4C9AE8FF part role=button
                  104 ContentPresenter at=978,862,747,141 by=scroll in=82 limit=978,580,768,288 role=none seen=978,862,747,6
                    105 SliderRow at=978,862,747,135 by=scroll in=82 limit=978,580,768,288 name=SNES.Parameter.standard paints=#333337FF,#4C9AE8FF,#55555AFF,#9C9CA0FF,#EEEEF0FF role=none seen=978,862,747,6
                  106 RepeatButton at=1746,580,24,24 drawn heard="Line up" name=PART_LineUpButton paints=#FFFFFFFF part role=button
                    107 ViewboxContainer at=1752,589,12,6.6 by=clip limit=1752,589,12,6.6 role=none
                  108 RepeatButton at=1746,664,24,180 drawn heard="Page down" name=PART_PageDownButton part role=button
                  109 RepeatButton at=1746,844,24,24 drawn heard="Line down" name=PART_LineDownButton paints=#FFFFFFFF part role=button
                    110 ViewboxContainer at=1752,853,12,6.6 by=clip limit=1752,853,12,6.6 role=none
        111 ButtonBar at=1653,205,135,66 by=panel limit=96,187,1728,698 paints=#9C9CA0FF role=toolbar
          112 Button at=1653,205,135,66 by=clip drawn focus heard=CLOSE limit=1653,205,135,66 op paints=#55555AFF,#9C9CA0FF role=button text=Close
            113 TextBlock at=1678.5,214,84,48 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=55.51,31.2 limit=1653,205,135,66 owner=112 paints=#EEEEF0FF role=text size=26 text=CLOSE
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
edge 78 right 87
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
