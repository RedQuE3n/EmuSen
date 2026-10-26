# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ShaderSettings-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 96,56.5,1728,982.5
title Shaders
footer "Drawn over the picture as it is shown, per console. A choice and every slider are saved at once and reach a running game of that console while its shader is in use. Not downloaded. libretro's slang pack, about 54 MB from buildbot.libretro.com, is the file RetroArch's updater fetches: some 2,600 presets. EmuSen ships none of it, and each shader keeps its authors' licence."
case ShaderSettings
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=96,206,1728,678 by=panel limit=96,206,1728,678 paints=#9C9CA0FF role=none
    1 DockPanel at=132,224,1656,642 by=panel limit=96,206,1728,678 role=none
      2 Grid at=132,224,1656,642 by=panel limit=96,206,1728,678 role=none
        3 Tabs at=132,224,1656,642 by=panel limit=96,206,1728,678 name=ConsoleTabs paints=#9C9CA0FF role=tab
          4 TabItem at=132,224,90,72 by=clip drawn focus heard=NES limit=132,224,1656,642 paints=#86868AFF role=tabitem sideways text=NES
            5 TextBlock at=150,236,54,48 box=36,32 by=clip caps=18.5487 drawn face="Barlow Condensed" heard=NES layout=35.46,31.8 limit=132,224,90,72 owner=4 paints=#86868AFF role=text size=26.5 text=NES
          6 TabItem at=222,224,72,72 by=clip drawn focus heard=GB limit=132,224,1656,642 paints=#86868AFF role=tabitem sideways text=GB
            7 TextBlock at=240,236,36,48 box=24,32 by=clip caps=18.5487 drawn face="Barlow Condensed" heard=GB layout=23.85,31.8 limit=222,224,72,72 owner=6 paints=#86868AFF role=text size=26.5 text=GB
          8 TabItem at=294,224,106.5,72 by=clip drawn focus heard=SNES limit=132,224,1656,642 op paints=#EEEEF0FF role=tabitem sideways text=SNES
            9 TextBlock at=312,236,70.5,48 box=47,32 by=clip caps=18.5487 drawn face="Barlow Condensed" heard=SNES layout=46.3,31.8 limit=294,224,106.5,72 owner=8 paints=#EEEEF0FF role=text size=26.5 text=SNES
          10 TabItem at=400.5,224,88.5,72 by=clip drawn focus heard=N64 limit=132,224,1656,642 paints=#86868AFF role=tabitem sideways text=N64
            11 TextBlock at=418.5,236,52.5,48 box=35,32 by=clip caps=18.5487 drawn face="Barlow Condensed" heard=N64 layout=34.93,31.8 limit=400.5,224,88.5,72 owner=10 paints=#86868AFF role=text size=26.5 text=N64
          12 TabItem at=489,224,147,72 by=clip drawn focus heard=SG-1000 limit=132,224,1656,642 paints=#86868AFF role=tabitem sideways text=SG-1000
            13 TextBlock at=507,236,111,48 box=74,32 by=clip caps=18.5487 drawn face="Barlow Condensed" heard=SG-1000 layout=73.03,31.8 limit=489,224,147,72 owner=12 paints=#86868AFF role=text size=26.5 text=SG-1000
          14 TabItem at=636,224,250.5,72 by=clip drawn focus heard="MASTER SYSTEM" limit=132,224,1656,642 paints=#86868AFF role=tabitem sideways text="Master System"
            15 TextBlock at=654,236,214.5,48 box=143,32 by=clip caps=18.5487 drawn face="Barlow Condensed" heard="MASTER SYSTEM" layout=142.91,31.8 limit=636,224,250.5,72 owner=14 paints=#86868AFF role=text size=26.5 text="MASTER SYSTEM"
          16 TabItem at=886.5,224,150,72 by=clip drawn focus heard=GENESIS limit=132,224,1656,642 paints=#86868AFF role=tabitem sideways text=Genesis
            17 TextBlock at=904.5,236,114,48 box=76,32 by=clip caps=18.5487 drawn face="Barlow Condensed" heard=GENESIS layout=75.5,31.8 limit=886.5,224,150,72 owner=16 paints=#86868AFF role=text size=26.5 text=GENESIS
          18 TabItem at=1036.5,224,148.5,72 by=clip drawn focus heard="SEGA CD" limit=132,224,1656,642 paints=#86868AFF role=tabitem sideways text="Sega CD"
            19 TextBlock at=1054.5,236,112.5,48 box=75,32 by=clip caps=18.5487 drawn face="Barlow Condensed" heard="SEGA CD" layout=74.36,31.8 limit=1036.5,224,148.5,72 owner=18 paints=#86868AFF role=text size=26.5 text="SEGA CD"
          20 TabItem at=1185,224,211.5,72 by=clip drawn focus heard=PLAYSTATION limit=132,224,1656,642 paints=#86868AFF role=tabitem sideways text=PlayStation
            21 TextBlock at=1203,236,175.5,48 box=117,32 by=clip caps=18.5487 drawn face="Barlow Condensed" heard=PLAYSTATION layout=116.52,31.8 limit=1185,224,211.5,72 owner=20 paints=#86868AFF role=text size=26.5 text=PLAYSTATION
          22 TabItem at=1396.5,224,235.5,72 by=clip drawn focus heard="PLAYSTATION 2" limit=132,224,1656,642 paints=#86868AFF role=tabitem sideways text="PlayStation 2"
            23 TextBlock at=1414.5,236,199.5,48 box=133,32 by=clip caps=18.5487 drawn face="Barlow Condensed" heard="PLAYSTATION 2" layout=132.47,31.8 limit=1396.5,224,235.5,72 owner=22 paints=#86868AFF role=text size=26.5 text="PLAYSTATION 2"
          24 ShaderPanel at=150,311,1620,555 by=clip limit=132,299,1656,567 role=none
            25 Grid at=150,311,792,555 by=clip limit=132,299,1656,567 role=none
              26 FilterBar at=150,311,792,69 by=clip limit=132,299,1656,567 name=SNES.ShaderSearch paints=#9C9CA0FF role=group
                27 TextBlock at=150,323,109.5,43.5 box=73,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Category layout=72.82,28.8 limit=150,311,792,69 paints=#86868AFF part role=text size=24 text=Category
                28 Dropdown at=271.5,311,270,66 drawn focus heard=Category name=PART_Facet op paints=#0E0E10FF,#55555AFF,#9C9CA0FF part role=combobox sideways
                  29 TextBlock at=289.5,318.5,204,48 box=136,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=All layout=20.07,31.2 limit=289.5,318.5,204,48 owner=28 paints=#9C9CA0FF role=text size=26 text=All
                  30 ViewboxContainer at=508.5,339.1,18,9.81 by=clip limit=508.5,339.1,18,9.81 nohit role=none
                31 TextBox at=553.5,311,388.5,69 drawn focus heard=Search name=PART_Search op paints=#0E0E10FF,#55555AFF,#D2D2D6FF part role=edit
                  32 ScrollViewer allow="a text box scrolls its own text" at=570,321.5,355.5,48 name=PART_ScrollViewer paints=#D2D2D6FF part role=scrollviewer
                    33 TextBlock allow="a text box scrolls its own text" at=570,321.5,355.5,48 box=237,32 caps=18.1987 drawn face="Barlow Condensed" heard=Search in=32 layout=61,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text=Search
              34 GroupedList at=150,392,792,393 by=clip limit=132,299,1656,567 name=SNES.ShaderList paints=#1B1B1EFF,#FFFFFFFF role=list
                35 ScrollViewer at=150,392,792,393 name=PART_ScrollViewer overflow=0 paints=#1E1E1EFF,#FFFFFFFF part role=scrollviewer viewport=150,392,792,393
                  36 ListBoxItem at=150,392,792,129 by=fixed-scroll drawn focus heard="Built into EmuSen: None" in=35 limit=150,392,792,393 op paints=#333337FF,#9C9CA0FF role=listitem
                    37 GroupedListRow at=150,392,792,127.5 by=clip in=35 limit=150,392,792,129 role=none
                      38 DockPanel at=165,413,762,36 by=clip in=35 limit=150,392,792,129 role=none
                        39 TextBlock at=913.5,413,13.5,36 box=9,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=5 in=35 layout=8.18,24 limit=150,392,792,129 owner=36 paints=#86868AFF role=text size=20 text=5
                        40 TextBlock at=165,413,736.5,36 box=491,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="BUILT INTO EMUSEN" in=35 layout=132.18,24 limit=150,392,792,129 owner=36 paints=#86868AFF role=text size=20 text="BUILT INTO EMUSEN"
                      41 Border at=156,456.5,780,61.5 by=clip in=35 limit=150,392,792,129 paints=#4C9AE8FF role=none
                        42 DockPanel at=171,465.5,750,43.5 by=clip in=35 limit=150,392,792,129 role=none
                          43 Border at=835.5,468.5,85.5,39 by=clip in=35 limit=150,392,792,129 paints=#FFFFFFFF role=none
                            44 TextBlock at=847.5,470,61.5,36 box=41,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="In use" in=35 layout=40.88,24 limit=150,392,792,129 owner=36 paints=#4C9AE8FF role=text size=20 text="In use"
                          45 StackPanel at=171,465.5,652.5,43.5 by=clip in=35 limit=150,392,792,129 role=none
                            46 TextBlock at=171,465.5,652.5,43.5 box=435,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=None in=35 layout=41.74,28.8 limit=150,392,792,129 owner=36 paints=#FFFFFFFF role=text size=24 text=None
                  47 ListBoxItem at=150,521,792,66 by=fixed-scroll drawn focus heard=CRT in=35 limit=150,392,792,393 op paints=#333337FF,#9C9CA0FF role=listitem
                    48 GroupedListRow at=150,521,792,64.5 by=clip in=35 limit=150,521,792,66 role=none
                      49 Border at=156,522.5,780,61.5 by=clip in=35 limit=150,521,792,66 role=none
                        50 DockPanel at=171,531.5,750,43.5 by=clip in=35 limit=150,521,792,66 role=none
                          51 StackPanel at=171,531.5,750,43.5 by=clip in=35 limit=150,521,792,66 role=none
                            52 TextBlock at=171,531.5,750,43.5 box=500,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=CRT in=35 layout=31.49,28.8 limit=150,521,792,66 owner=47 paints=#9C9CA0FF role=text size=24 text=CRT
                  53 ListBoxItem at=150,587,792,66 by=fixed-scroll drawn focus heard="CRT (Lottes)" in=35 limit=150,392,792,393 op paints=#333337FF,#9C9CA0FF role=listitem
                    54 GroupedListRow at=150,587,792,64.5 by=clip in=35 limit=150,587,792,66 role=none
                      55 Border at=156,588.5,780,61.5 by=clip in=35 limit=150,587,792,66 role=none
                        56 DockPanel at=171,597.5,750,43.5 by=clip in=35 limit=150,587,792,66 role=none
                          57 StackPanel at=171,597.5,750,43.5 by=clip in=35 limit=150,587,792,66 role=none
                            58 TextBlock at=171,597.5,750,43.5 box=500,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="CRT (Lottes)" in=35 layout=97.92,28.8 limit=150,587,792,66 owner=53 paints=#9C9CA0FF role=text size=24 text="CRT (Lottes)"
                  59 ListBoxItem at=150,653,792,66 by=fixed-scroll drawn focus heard=Scanlines in=35 limit=150,392,792,393 op paints=#333337FF,#9C9CA0FF role=listitem
                    60 GroupedListRow at=150,653,792,64.5 by=clip in=35 limit=150,653,792,66 role=none
                      61 Border at=156,654.5,780,61.5 by=clip in=35 limit=150,653,792,66 role=none
                        62 DockPanel at=171,663.5,750,43.5 by=clip in=35 limit=150,653,792,66 role=none
                          63 StackPanel at=171,663.5,750,43.5 by=clip in=35 limit=150,653,792,66 role=none
                            64 TextBlock at=171,663.5,750,43.5 box=500,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Scanlines in=35 layout=77.93,28.8 limit=150,653,792,66 owner=59 paints=#9C9CA0FF role=text size=24 text=Scanlines
                  65 ListBoxItem at=150,719,792,66 by=fixed-scroll drawn focus heard="Simple CRT" in=35 limit=150,392,792,393 op paints=#333337FF,#9C9CA0FF role=listitem
                    66 GroupedListRow at=150,719,792,64.5 by=clip in=35 limit=150,719,792,66 role=none
                      67 Border at=156,720.5,780,61.5 by=clip in=35 limit=150,719,792,66 role=none
                        68 DockPanel at=171,729.5,750,43.5 by=clip in=35 limit=150,719,792,66 role=none
                          69 StackPanel at=171,729.5,750,43.5 by=clip in=35 limit=150,719,792,66 role=none
                            70 TextBlock at=171,729.5,750,43.5 box=500,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Simple CRT" in=35 layout=91.2,28.8 limit=150,719,792,66 owner=65 paints=#9C9CA0FF role=text size=24 text="Simple CRT"
              71 Border at=150,800,792,66 by=clip limit=132,299,1656,567 role=none
                72 Button at=150,800,273,66 by=clip drawn focus heard="DOWNLOAD PACK" limit=132,299,1656,567 name=SNES.DownloadPack op paints=#55555AFF,#9C9CA0FF role=button text="Download Pack"
                  73 TextBlock at=175.5,809,222,48 box=148,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="DOWNLOAD PACK" layout=147.39,31.2 limit=150,800,273,66 owner=72 paints=#9C9CA0FF role=text size=26 text="DOWNLOAD PACK"
            74 DockPanel at=978,311,792,555 by=clip limit=132,299,1656,567 role=none
              75 StackPanel at=978,311,792,157.5 by=clip limit=132,299,1656,567 role=none
                76 Grid at=978,311,792,66 by=clip limit=132,299,1656,567 role=none
                  77 TextBlock at=978,311,435,66 box=290,44 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=None layout=41.74,28.8 limit=132,299,1656,567 name=SNES.ShaderName paints=#9C9CA0FF role=text size=24 text=None
                  78 StackPanel at=1431,311,339,66 by=clip limit=132,299,1656,567 role=none
                    79 Button at=1431,311,189,66 by=clip drawn heard="RESET ALL" limit=132,299,1656,567 name=SNES.ResetShader off paints=#55555AFF,#9C9CA0FF role=button text="Reset All"
                      80 TextBlock at=1456.5,320,138,48 box=92,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="RESET ALL" layout=91.91,31.2 limit=1431,311,189,66 off owner=79 paints=#55555AFF role=text size=26 text="RESET ALL"
                    81 Button at=1632,311,138,66 by=clip drawn heard="IN USE" limit=132,299,1656,567 name=SNES.UseShader off paints=#55555AFF,#9C9CA0FF role=button text="In Use"
                      82 TextBlock at=1657.5,320,87,48 box=58,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="IN USE" layout=57.77,31.2 limit=1632,311,138,66 off owner=81 paints=#55555AFF role=text size=26 text="IN USE"
                83 MonoText at=978,386,792,37.5 box=528,25 by=clip caps=12.854 drawn face="Noto Sans" heard="Built into EmuSen" layout=150.25,24.52 limit=132,299,1656,567 name=SNES.ShaderPath paints=#9C9CA0FF role=text size=18 text="Built into EmuSen"
                84 HintText at=978,432.5,792,36 box=528,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="This shader has nothing to adjust." layout=230.42,24 limit=132,299,1656,567 paints=#86868AFF role=text size=20 text="This shader has nothing to adjust."
              85 SliderList at=978,480.5,792,385.5 by=clip limit=132,299,1656,567 name=SNES.ShaderParameters paints=#9C9CA0FF role=list
                86 ScrollViewer at=978,480.5,792,385.5 name=PART_ScrollViewer overflow=0 paints=#1E1E1EFF,#9C9CA0FF part role=scrollviewer viewport=978,480.5,792,385.5
        87 ButtonBar at=1653,224,135,66 by=panel limit=96,206,1728,678 paints=#9C9CA0FF role=toolbar
          88 Button at=1653,224,135,66 by=clip drawn focus heard=CLOSE limit=1653,224,135,66 op paints=#55555AFF,#9C9CA0FF role=button text=Close
            89 TextBlock at=1678.5,233,84,48 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=55.51,31.2 limit=1653,224,135,66 owner=88 paints=#EEEEF0FF role=text size=26 text=CLOSE
focus 88
edge 8 up 88
edge 8 down 28
edge 28 up 8
edge 28 down 36
edge 31 up 8
edge 31 down 36
edge 31 left 28
edge 31 right 88
edge 36 up 31
edge 36 down 47
edge 36 left same
edge 36 right 88
edge 47 up 36
edge 47 down 53
edge 47 left same
edge 47 right 81
edge 53 up 47
edge 53 down 59
edge 53 left same
edge 53 right 81
edge 59 up 53
edge 59 down 65
edge 59 left same
edge 59 right 81
edge 65 up 59
edge 65 down 72
edge 65 left same
edge 65 right 81
edge 72 up 65
edge 72 down same
edge 72 left same
edge 72 right 31
edge 81 up 8
edge 81 down path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/1/1/0/1/0/3/0/2
edge 81 left 31
edge 81 right same
edge 88 up same
edge 88 down 31
edge 88 left 8
edge 88 right same
edge "item:85/0/0/0/0/0/Reset Quality" up path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/1/1/0/1/0/3/0/2
edge "item:85/0/0/0/0/0/Reset Quality" down item:85/0/0/0/1/Quality
edge "item:85/0/0/0/0/0/Reset Quality" left 53
edge "item:85/0/0/0/0/0/Reset Quality" right same
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/1/1/0/1/0/3/0/2 up 81
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/1/1/0/1/0/3/0/2 down "item:85/0/0/0/0/0/Reset Quality"
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/1/1/0/1/0/3/0/2 left 47
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/1/1/0/1/0/3/0/2 right "item:85/0/0/0/0/0/Reset Quality"
press pageup focus="Dropdown PART_Facet (Category)" tab=GB title=Shaders top=ShaderSettingsWindow
press pagedown focus="Dropdown PART_Facet (Category)" tab=SNES title=Shaders top=ShaderSettingsWindow
press first focus="Dropdown PART_Facet (Category)" tab=SNES title=Shaders top=ShaderSettingsWindow
press last focus="Dropdown PART_Facet (Category)" tab=SNES title=Shaders top=ShaderSettingsWindow
press back top=none
