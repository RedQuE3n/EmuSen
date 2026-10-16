# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ShaderSettings-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 64,37,1152,655
title Shaders
footer "Drawn over the picture as it is shown, per console. A choice and every slider are saved at once and reach a running game of that console while its shader is in use. Not downloaded. libretro's slang pack, about 54 MB from buildbot.libretro.com, is the file RetroArch's updater fetches: some 2,600 presets. EmuSen ships none of it, and each shader keeps its authors' licence."
case ShaderSettings
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=64,137,1152,452 by=panel limit=64,137,1152,452 paints=#9C9CA0FF role=none
    1 DockPanel at=88,149,1104,428 by=panel limit=64,137,1152,452 role=none
      2 Grid at=88,149,1104,428 by=panel limit=64,137,1152,452 role=none
        3 Tabs at=88,149,1104,428 by=panel limit=64,137,1152,452 name=ConsoleTabs paints=#9C9CA0FF role=tab
          4 TabItem at=88,149,65,48 by=clip drawn focus heard=NES limit=88,149,1104,428 paints=#86868AFF role=tabitem sideways text=NES
            5 TextBlock at=100,155,41,36 box=41,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=NES layout=40.14,36 limit=88,149,65,48 owner=4 paints=#86868AFF role=text size=30 text=NES
          6 TabItem at=153,149,51,48 by=clip drawn focus heard=GB limit=88,149,1104,428 paints=#86868AFF role=tabitem sideways text=GB
            7 TextBlock at=165,155,27,36 box=27,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GB layout=27,36 limit=153,149,51,48 owner=6 paints=#86868AFF role=text size=30 text=GB
          8 TabItem at=204,149,77,48 by=clip drawn focus heard=SNES limit=88,149,1104,428 op paints=#EEEEF0FF role=tabitem sideways text=SNES
            9 TextBlock at=216,155,53,36 box=53,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=SNES layout=52.41,36 limit=204,149,77,48 owner=8 paints=#EEEEF0FF role=text size=30 text=SNES
          10 TabItem at=281,149,64,48 by=clip drawn focus heard=N64 limit=88,149,1104,428 paints=#86868AFF role=tabitem sideways text=N64
            11 TextBlock at=293,155,40,36 box=40,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=N64 layout=39.54,36 limit=281,149,64,48 owner=10 paints=#86868AFF role=text size=30 text=N64
          12 TabItem at=345,149,110,48 by=clip drawn focus heard=GENESIS limit=88,149,1104,428 paints=#86868AFF role=tabitem sideways text=Genesis
            13 TextBlock at=357,155,86,36 box=86,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GENESIS layout=85.47,36 limit=345,149,110,48 owner=12 paints=#86868AFF role=text size=30 text=GENESIS
          14 TabItem at=455,149,109,48 by=clip drawn focus heard="SEGA CD" limit=88,149,1104,428 paints=#86868AFF role=tabitem sideways text="Sega CD"
            15 TextBlock at=467,155,85,36 box=85,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard="SEGA CD" layout=84.18,36 limit=455,149,109,48 owner=14 paints=#86868AFF role=text size=30 text="SEGA CD"
          16 TabItem at=564,149,156,48 by=clip drawn focus heard=PLAYSTATION limit=88,149,1104,428 paints=#86868AFF role=tabitem sideways text=PlayStation
            17 TextBlock at=576,155,132,36 box=132,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=PLAYSTATION layout=131.91,36 limit=564,149,156,48 owner=16 paints=#86868AFF role=text size=30 text=PLAYSTATION
          18 ShaderPanel at=100,207,1080,370 by=clip limit=88,199,1104,378 role=none
            19 Grid at=100,207,528,370 by=clip limit=88,199,1104,378 role=none
              20 FilterBar at=100,207,528,46 by=clip limit=88,199,1104,378 name=SNES.ShaderSearch paints=#9C9CA0FF role=group
                21 TextBlock at=100,215,73,29 box=73,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Category layout=72.82,28.8 limit=100,207,528,46 paints=#86868AFF part role=text size=24 text=Category
                22 Dropdown at=181,207,180,44 drawn focus heard=Category name=PART_Facet op paints=#0E0E10FF,#55555AFF,#9C9CA0FF part role=combobox sideways
                  23 TextBlock at=193,212,136,32 box=136,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=All layout=20.07,31.2 limit=193,212,136,32 owner=22 paints=#9C9CA0FF role=text size=26 text=All
                  24 ViewboxContainer at=339,225.73,12,6.54 by=clip limit=339,225.73,12,6.54 nohit role=none
                25 TextBox at=369,207,259,46 drawn focus heard=Search name=PART_Search op paints=#0E0E10FF,#55555AFF,#D2D2D6FF part role=edit
                  26 ScrollViewer allow="a text box scrolls its own text" at=380,214,237,32 name=PART_ScrollViewer paints=#D2D2D6FF part role=scrollviewer
                    27 TextBlock allow="a text box scrolls its own text" at=380,214,237,32 box=237,32 caps=18.1987 drawn face="Barlow Condensed" heard=Search in=26 layout=61,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text=Search
              28 GroupedList at=100,261,528,262 by=clip limit=88,199,1104,378 name=SNES.ShaderList paints=#1B1B1EFF,#FFFFFFFF role=list
                29 ScrollViewer at=100,261,528,262 name=PART_ScrollViewer overflow=0 paints=#1E1E1EFF,#FFFFFFFF part role=scrollviewer viewport=100,261,528,262
                  30 ListBoxItem at=100,261,528,86 by=fixed-scroll drawn focus heard="Built into EmuSen: None" in=29 limit=100,261,528,262 op paints=#333337FF,#9C9CA0FF role=listitem
                    31 GroupedListRow at=100,261,528,85 by=clip in=29 limit=100,261,528,86 role=none
                      32 DockPanel at=110,275,508,24 by=clip in=29 limit=100,261,528,86 role=none
                        33 TextBlock at=609,275,9,24 box=9,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=5 in=29 layout=8.18,24 limit=100,261,528,86 owner=30 paints=#86868AFF role=text size=20 text=5
                        34 TextBlock at=110,275,491,24 box=491,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="BUILT INTO EMUSEN" in=29 layout=132.18,24 limit=100,261,528,86 owner=30 paints=#86868AFF role=text size=20 text="BUILT INTO EMUSEN"
                      35 Border at=104,304,520,41 by=clip in=29 limit=100,261,528,86 paints=#4C9AE8FF role=none
                        36 DockPanel at=114,310,500,29 by=clip in=29 limit=100,261,528,86 role=none
                          37 Border at=557,312,57,26 by=clip in=29 limit=100,261,528,86 paints=#FFFFFFFF role=none
                            38 TextBlock at=565,313,41,24 box=41,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="In use" in=29 layout=40.88,24 limit=100,261,528,86 owner=30 paints=#4C9AE8FF role=text size=20 text="In use"
                          39 StackPanel at=114,310,435,29 by=clip in=29 limit=100,261,528,86 role=none
                            40 TextBlock at=114,310,435,29 box=435,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=None in=29 layout=41.74,28.8 limit=100,261,528,86 owner=30 paints=#FFFFFFFF role=text size=24 text=None
                  41 ListBoxItem at=100,347,528,44 by=fixed-scroll drawn focus heard=CRT in=29 limit=100,261,528,262 op paints=#333337FF,#9C9CA0FF role=listitem
                    42 GroupedListRow at=100,347,528,43 by=clip in=29 limit=100,347,528,44 role=none
                      43 Border at=104,348,520,41 by=clip in=29 limit=100,347,528,44 role=none
                        44 DockPanel at=114,354,500,29 by=clip in=29 limit=100,347,528,44 role=none
                          45 StackPanel at=114,354,500,29 by=clip in=29 limit=100,347,528,44 role=none
                            46 TextBlock at=114,354,500,29 box=500,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=CRT in=29 layout=31.49,28.8 limit=100,347,528,44 owner=41 paints=#9C9CA0FF role=text size=24 text=CRT
                  47 ListBoxItem at=100,391,528,44 by=fixed-scroll drawn focus heard="CRT (Lottes)" in=29 limit=100,261,528,262 op paints=#333337FF,#9C9CA0FF role=listitem
                    48 GroupedListRow at=100,391,528,43 by=clip in=29 limit=100,391,528,44 role=none
                      49 Border at=104,392,520,41 by=clip in=29 limit=100,391,528,44 role=none
                        50 DockPanel at=114,398,500,29 by=clip in=29 limit=100,391,528,44 role=none
                          51 StackPanel at=114,398,500,29 by=clip in=29 limit=100,391,528,44 role=none
                            52 TextBlock at=114,398,500,29 box=500,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="CRT (Lottes)" in=29 layout=97.92,28.8 limit=100,391,528,44 owner=47 paints=#9C9CA0FF role=text size=24 text="CRT (Lottes)"
                  53 ListBoxItem at=100,435,528,44 by=fixed-scroll drawn focus heard=Scanlines in=29 limit=100,261,528,262 op paints=#333337FF,#9C9CA0FF role=listitem
                    54 GroupedListRow at=100,435,528,43 by=clip in=29 limit=100,435,528,44 role=none
                      55 Border at=104,436,520,41 by=clip in=29 limit=100,435,528,44 role=none
                        56 DockPanel at=114,442,500,29 by=clip in=29 limit=100,435,528,44 role=none
                          57 StackPanel at=114,442,500,29 by=clip in=29 limit=100,435,528,44 role=none
                            58 TextBlock at=114,442,500,29 box=500,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Scanlines in=29 layout=77.93,28.8 limit=100,435,528,44 owner=53 paints=#9C9CA0FF role=text size=24 text=Scanlines
                  59 ListBoxItem at=100,479,528,44 by=fixed-scroll drawn focus heard="Simple CRT" in=29 limit=100,261,528,262 op paints=#333337FF,#9C9CA0FF role=listitem
                    60 GroupedListRow at=100,479,528,43 by=clip in=29 limit=100,479,528,44 role=none
                      61 Border at=104,480,520,41 by=clip in=29 limit=100,479,528,44 role=none
                        62 DockPanel at=114,486,500,29 by=clip in=29 limit=100,479,528,44 role=none
                          63 StackPanel at=114,486,500,29 by=clip in=29 limit=100,479,528,44 role=none
                            64 TextBlock at=114,486,500,29 box=500,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Simple CRT" in=29 layout=91.2,28.8 limit=100,479,528,44 owner=59 paints=#9C9CA0FF role=text size=24 text="Simple CRT"
              65 Border at=100,533,528,44 by=clip limit=88,199,1104,378 role=none
                66 Button at=100,533,182,44 by=clip drawn focus heard="DOWNLOAD PACK" limit=88,199,1104,378 name=SNES.DownloadPack op paints=#55555AFF,#9C9CA0FF role=button text="Download Pack"
                  67 TextBlock at=117,539,148,32 box=148,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="DOWNLOAD PACK" layout=147.39,31.2 limit=100,533,182,44 owner=66 paints=#9C9CA0FF role=text size=26 text="DOWNLOAD PACK"
            68 DockPanel at=652,207,528,370 by=clip limit=88,199,1104,378 role=none
              69 StackPanel at=652,207,528,105 by=clip limit=88,199,1104,378 role=none
                70 Grid at=652,207,528,44 by=clip limit=88,199,1104,378 role=none
                  71 TextBlock at=652,207,290,44 box=290,44 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=None layout=41.74,28.8 limit=88,199,1104,378 name=SNES.ShaderName paints=#9C9CA0FF role=text size=24 text=None
                  72 StackPanel at=954,207,226,44 by=clip limit=88,199,1104,378 role=none
                    73 Button at=954,207,126,44 by=clip drawn heard="RESET ALL" limit=88,199,1104,378 name=SNES.ResetShader off paints=#55555AFF,#9C9CA0FF role=button text="Reset All"
                      74 TextBlock at=971,213,92,32 box=92,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="RESET ALL" layout=91.91,31.2 limit=954,207,126,44 off owner=73 paints=#55555AFF role=text size=26 text="RESET ALL"
                    75 Button at=1088,207,92,44 by=clip drawn heard="IN USE" limit=88,199,1104,378 name=SNES.UseShader off paints=#55555AFF,#9C9CA0FF role=button text="In Use"
                      76 TextBlock at=1105,213,58,32 box=58,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="IN USE" layout=57.77,31.2 limit=1088,207,92,44 off owner=75 paints=#55555AFF role=text size=26 text="IN USE"
                77 MonoText at=652,257,528,25 box=528,25 by=clip caps=12.854 drawn face="Noto Sans" heard="Built into EmuSen" layout=150.25,24.52 limit=88,199,1104,378 name=SNES.ShaderPath paints=#9C9CA0FF role=text size=18 text="Built into EmuSen"
                78 HintText at=652,288,528,24 box=528,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="This shader has nothing to adjust." layout=230.42,24 limit=88,199,1104,378 paints=#86868AFF role=text size=20 text="This shader has nothing to adjust."
              79 SliderList at=652,320,528,257 by=clip limit=88,199,1104,378 name=SNES.ShaderParameters paints=#9C9CA0FF role=list
                80 ScrollViewer at=652,320,528,257 name=PART_ScrollViewer overflow=0 paints=#1E1E1EFF,#9C9CA0FF part role=scrollviewer viewport=652,320,528,257
        81 ButtonBar at=1102,149,90,44 by=panel limit=64,137,1152,452 paints=#9C9CA0FF role=toolbar
          82 Button at=1102,149,90,44 by=clip drawn focus heard=CLOSE limit=1102,149,90,44 op paints=#55555AFF,#9C9CA0FF role=button text=Close
            83 TextBlock at=1119,155,56,32 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=55.51,31.2 limit=1102,149,90,44 owner=82 paints=#EEEEF0FF role=text size=26 text=CLOSE
focus 82
edge 8 up same
edge 8 down 22
edge 22 up 8
edge 22 down 30
edge 25 up 8
edge 25 down 30
edge 25 left 22
edge 25 right 82
edge 30 up 25
edge 30 down 41
edge 30 left same
edge 30 right 82
edge 41 up 30
edge 41 down 47
edge 41 left same
edge 41 right 75
edge 47 up 41
edge 47 down 53
edge 47 left same
edge 47 right 75
edge 53 up 47
edge 53 down 59
edge 53 left same
edge 53 right 75
edge 59 up 53
edge 59 down 66
edge 59 left same
edge 59 right 75
edge 66 up 59
edge 66 down same
edge 66 left same
edge 66 right 25
edge 75 up 82
edge 75 down path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/1/1/0/1/0/3/0/2
edge 75 left 25
edge 75 right same
edge 82 up same
edge 82 down 25
edge 82 left 8
edge 82 right same
edge "item:79/0/0/0/0/0/Reset Quality" up path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/1/1/0/1/0/3/0/2
edge "item:79/0/0/0/0/0/Reset Quality" down item:79/0/0/0/1/Quality
edge "item:79/0/0/0/0/0/Reset Quality" left 47
edge "item:79/0/0/0/0/0/Reset Quality" right same
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/1/1/0/1/0/3/0/2 up 75
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/1/1/0/1/0/3/0/2 down "item:79/0/0/0/0/0/Reset Quality"
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/1/1/0/1/0/3/0/2 left 41
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/1/1/0/1/0/3/0/2 right same
press pageup focus="Dropdown PART_Facet (Category)" tab=GB title=Shaders top=ShaderSettingsWindow
press pagedown focus="Dropdown PART_Facet (Category)" tab=SNES title=Shaders top=ShaderSettingsWindow
press first focus="Dropdown PART_Facet (Category)" tab=SNES title=Shaders top=ShaderSettingsWindow
press last focus="Dropdown PART_Facet (Category)" tab=SNES title=Shaders top=ShaderSettingsWindow
press back top=none
