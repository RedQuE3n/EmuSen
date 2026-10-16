# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ShaderSettingsPlayStation-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 64,81,1152,611
title Shaders
footer "Drawn over the picture as it is shown, per console. A choice and every slider are saved at once and reach a running game of that console while its shader is in use. Not downloaded. libretro's slang pack, about 54 MB from buildbot.libretro.com, is the file RetroArch's updater fetches: some 2,600 presets. EmuSen ships none of it, and each shader keeps its authors' licence."
case ShaderSettingsPlayStation
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=64,181,1152,408 by=panel limit=64,181,1152,408 paints=#9C9CA0FF role=none
    1 DockPanel at=88,193,1104,384 by=panel limit=64,181,1152,408 role=none
      2 Grid at=88,193,1104,384 by=panel limit=64,181,1152,408 role=none
        3 Tabs at=88,193,1104,384 by=panel limit=64,181,1152,408 name=ConsoleTabs paints=#9C9CA0FF role=tab
          4 TabItem at=88,193,65,48 by=clip drawn focus heard=NES limit=88,193,1104,384 paints=#86868AFF role=tabitem sideways text=NES
            5 TextBlock at=100,199,41,36 box=41,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=NES layout=40.14,36 limit=88,193,65,48 owner=4 paints=#86868AFF role=text size=30 text=NES
          6 TabItem at=153,193,51,48 by=clip drawn focus heard=GB limit=88,193,1104,384 paints=#86868AFF role=tabitem sideways text=GB
            7 TextBlock at=165,199,27,36 box=27,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GB layout=27,36 limit=153,193,51,48 owner=6 paints=#86868AFF role=text size=30 text=GB
          8 TabItem at=204,193,77,48 by=clip drawn focus heard=SNES limit=88,193,1104,384 paints=#86868AFF role=tabitem sideways text=SNES
            9 TextBlock at=216,199,53,36 box=53,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=SNES layout=52.41,36 limit=204,193,77,48 owner=8 paints=#86868AFF role=text size=30 text=SNES
          10 TabItem at=281,193,64,48 by=clip drawn focus heard=N64 limit=88,193,1104,384 paints=#86868AFF role=tabitem sideways text=N64
            11 TextBlock at=293,199,40,36 box=40,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=N64 layout=39.54,36 limit=281,193,64,48 owner=10 paints=#86868AFF role=text size=30 text=N64
          12 TabItem at=345,193,110,48 by=clip drawn focus heard=GENESIS limit=88,193,1104,384 paints=#86868AFF role=tabitem sideways text=Genesis
            13 TextBlock at=357,199,86,36 box=86,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GENESIS layout=85.47,36 limit=345,193,110,48 owner=12 paints=#86868AFF role=text size=30 text=GENESIS
          14 TabItem at=455,193,109,48 by=clip drawn focus heard="SEGA CD" limit=88,193,1104,384 paints=#86868AFF role=tabitem sideways text="Sega CD"
            15 TextBlock at=467,199,85,36 box=85,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard="SEGA CD" layout=84.18,36 limit=455,193,109,48 owner=14 paints=#86868AFF role=text size=30 text="SEGA CD"
          16 TabItem at=564,193,156,48 by=clip drawn focus heard=PLAYSTATION limit=88,193,1104,384 op paints=#EEEEF0FF role=tabitem sideways text=PlayStation
            17 TextBlock at=576,199,132,36 box=132,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=PLAYSTATION layout=131.91,36 limit=564,193,156,48 owner=16 paints=#EEEEF0FF role=text size=30 text=PLAYSTATION
          18 ShaderPanel at=100,251,1080,326 by=clip limit=88,243,1104,334 role=none
            19 Grid at=100,251,528,326 by=clip limit=88,243,1104,334 role=none
              20 FilterBar at=100,251,528,46 by=clip limit=88,243,1104,334 name=PlayStation.ShaderSearch paints=#9C9CA0FF role=group
                21 TextBlock at=100,259,73,29 box=73,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Category layout=72.82,28.8 limit=100,251,528,46 paints=#86868AFF part role=text size=24 text=Category
                22 Dropdown at=181,251,180,44 drawn focus heard=Category name=PART_Facet op paints=#0E0E10FF,#55555AFF,#9C9CA0FF part role=combobox sideways
                  23 TextBlock at=193,256,136,32 box=136,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=All layout=20.07,31.2 limit=193,256,136,32 owner=22 paints=#9C9CA0FF role=text size=26 text=All
                  24 ViewboxContainer at=339,269.73,12,6.54 by=clip limit=339,269.73,12,6.54 nohit role=none
                25 TextBox at=369,251,259,46 drawn focus heard=Search name=PART_Search op paints=#0E0E10FF,#55555AFF,#D2D2D6FF part role=edit
                  26 ScrollViewer allow="a text box scrolls its own text" at=380,258,237,32 name=PART_ScrollViewer paints=#D2D2D6FF part role=scrollviewer
                    27 TextBlock allow="a text box scrolls its own text" at=380,258,237,32 box=237,32 caps=18.1987 drawn face="Barlow Condensed" heard=Search in=26 layout=61,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text=Search
              28 GroupedList at=100,305,528,218 by=clip limit=88,243,1104,334 name=PlayStation.ShaderList paints=#1B1B1EFF,#FFFFFFFF role=list
                29 ScrollViewer at=100,305,528,218 name=PART_ScrollViewer overflow=0 paints=#1E1E1EFF,#FFFFFFFF part role=scrollviewer viewport=100,305,528,218
                  30 ListBoxItem at=100,305,528,86 by=fixed-scroll drawn focus heard="Built into EmuSen: None" in=29 limit=100,305,528,218 op paints=#333337FF,#9C9CA0FF role=listitem
                    31 GroupedListRow at=100,305,528,85 by=clip in=29 limit=100,305,528,86 role=none
                      32 DockPanel at=110,319,508,24 by=clip in=29 limit=100,305,528,86 role=none
                        33 TextBlock at=609,319,9,24 box=9,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard=4 in=29 layout=8.3,24 limit=100,305,528,86 owner=30 paints=#86868AFF role=text size=20 text=4
                        34 TextBlock at=110,319,491,24 box=491,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="BUILT INTO EMUSEN" in=29 layout=132.18,24 limit=100,305,528,86 owner=30 paints=#86868AFF role=text size=20 text="BUILT INTO EMUSEN"
                      35 Border at=104,348,520,41 by=clip in=29 limit=100,305,528,86 paints=#4C9AE8FF role=none
                        36 DockPanel at=114,354,500,29 by=clip in=29 limit=100,305,528,86 role=none
                          37 Border at=557,356,57,26 by=clip in=29 limit=100,305,528,86 paints=#FFFFFFFF role=none
                            38 TextBlock at=565,357,41,24 box=41,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="In use" in=29 layout=40.88,24 limit=100,305,528,86 owner=30 paints=#4C9AE8FF role=text size=20 text="In use"
                          39 StackPanel at=114,354,435,29 by=clip in=29 limit=100,305,528,86 role=none
                            40 TextBlock at=114,354,435,29 box=435,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=None in=29 layout=41.74,28.8 limit=100,305,528,86 owner=30 paints=#FFFFFFFF role=text size=24 text=None
                  41 ListBoxItem at=100,391,528,44 by=fixed-scroll drawn focus heard=CRT in=29 limit=100,305,528,218 op paints=#333337FF,#9C9CA0FF role=listitem
                    42 GroupedListRow at=100,391,528,43 by=clip in=29 limit=100,391,528,44 role=none
                      43 Border at=104,392,520,41 by=clip in=29 limit=100,391,528,44 role=none
                        44 DockPanel at=114,398,500,29 by=clip in=29 limit=100,391,528,44 role=none
                          45 StackPanel at=114,398,500,29 by=clip in=29 limit=100,391,528,44 role=none
                            46 TextBlock at=114,398,500,29 box=500,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=CRT in=29 layout=31.49,28.8 limit=100,391,528,44 owner=41 paints=#9C9CA0FF role=text size=24 text=CRT
                  47 ListBoxItem at=100,435,528,44 by=fixed-scroll drawn focus heard=Scanlines in=29 limit=100,305,528,218 op paints=#333337FF,#9C9CA0FF role=listitem
                    48 GroupedListRow at=100,435,528,43 by=clip in=29 limit=100,435,528,44 role=none
                      49 Border at=104,436,520,41 by=clip in=29 limit=100,435,528,44 role=none
                        50 DockPanel at=114,442,500,29 by=clip in=29 limit=100,435,528,44 role=none
                          51 StackPanel at=114,442,500,29 by=clip in=29 limit=100,435,528,44 role=none
                            52 TextBlock at=114,442,500,29 box=500,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Scanlines in=29 layout=77.93,28.8 limit=100,435,528,44 owner=47 paints=#9C9CA0FF role=text size=24 text=Scanlines
                  53 ListBoxItem at=100,479,528,44 by=fixed-scroll drawn focus heard="Simple CRT" in=29 limit=100,305,528,218 op paints=#333337FF,#9C9CA0FF role=listitem
                    54 GroupedListRow at=100,479,528,43 by=clip in=29 limit=100,479,528,44 role=none
                      55 Border at=104,480,520,41 by=clip in=29 limit=100,479,528,44 role=none
                        56 DockPanel at=114,486,500,29 by=clip in=29 limit=100,479,528,44 role=none
                          57 StackPanel at=114,486,500,29 by=clip in=29 limit=100,479,528,44 role=none
                            58 TextBlock at=114,486,500,29 box=500,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Simple CRT" in=29 layout=91.2,28.8 limit=100,479,528,44 owner=53 paints=#9C9CA0FF role=text size=24 text="Simple CRT"
              59 Border at=100,533,528,44 by=clip limit=88,243,1104,334 role=none
                60 Button at=100,533,182,44 by=clip drawn focus heard="DOWNLOAD PACK" limit=88,243,1104,334 name=PlayStation.DownloadPack op paints=#55555AFF,#9C9CA0FF role=button text="Download Pack"
                  61 TextBlock at=117,539,148,32 box=148,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="DOWNLOAD PACK" layout=147.39,31.2 limit=100,533,182,44 owner=60 paints=#9C9CA0FF role=text size=26 text="DOWNLOAD PACK"
            62 DockPanel at=652,251,528,326 by=clip limit=88,243,1104,334 role=none
              63 StackPanel at=652,251,528,105 by=clip limit=88,243,1104,334 role=none
                64 Grid at=652,251,528,44 by=clip limit=88,243,1104,334 role=none
                  65 TextBlock at=652,251,290,44 box=290,44 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=None layout=41.74,28.8 limit=88,243,1104,334 name=PlayStation.ShaderName paints=#9C9CA0FF role=text size=24 text=None
                  66 StackPanel at=954,251,226,44 by=clip limit=88,243,1104,334 role=none
                    67 Button at=954,251,126,44 by=clip drawn heard="RESET ALL" limit=88,243,1104,334 name=PlayStation.ResetShader off paints=#55555AFF,#9C9CA0FF role=button text="Reset All"
                      68 TextBlock at=971,257,92,32 box=92,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="RESET ALL" layout=91.91,31.2 limit=954,251,126,44 off owner=67 paints=#55555AFF role=text size=26 text="RESET ALL"
                    69 Button at=1088,251,92,44 by=clip drawn heard="IN USE" limit=88,243,1104,334 name=PlayStation.UseShader off paints=#55555AFF,#9C9CA0FF role=button text="In Use"
                      70 TextBlock at=1105,257,58,32 box=58,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="IN USE" layout=57.77,31.2 limit=1088,251,92,44 off owner=69 paints=#55555AFF role=text size=26 text="IN USE"
                71 MonoText at=652,301,528,25 box=528,25 by=clip caps=12.854 drawn face="Noto Sans" heard="Built into EmuSen" layout=150.25,24.52 limit=88,243,1104,334 name=PlayStation.ShaderPath paints=#9C9CA0FF role=text size=18 text="Built into EmuSen"
                72 HintText at=652,332,528,24 box=528,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="This shader has nothing to adjust." layout=230.42,24 limit=88,243,1104,334 paints=#86868AFF role=text size=20 text="This shader has nothing to adjust."
              73 SliderList at=652,364,528,213 by=clip limit=88,243,1104,334 name=PlayStation.ShaderParameters paints=#9C9CA0FF role=list
                74 ScrollViewer at=652,364,528,213 name=PART_ScrollViewer overflow=0 paints=#1E1E1EFF,#9C9CA0FF part role=scrollviewer viewport=652,364,528,213
        75 ButtonBar at=1102,193,90,44 by=panel limit=64,181,1152,408 paints=#9C9CA0FF role=toolbar
          76 Button at=1102,193,90,44 by=clip drawn focus heard=CLOSE limit=1102,193,90,44 op paints=#55555AFF,#9C9CA0FF role=button text=Close
            77 TextBlock at=1119,199,56,32 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=55.51,31.2 limit=1102,193,90,44 owner=76 paints=#EEEEF0FF role=text size=26 text=CLOSE
focus 76
edge 16 up same
edge 16 down 22
edge 22 up 16
edge 22 down 30
edge 25 up 16
edge 25 down 30
edge 25 left 22
edge 25 right 76
edge 30 up 25
edge 30 down 41
edge 30 left same
edge 30 right 76
edge 41 up 30
edge 41 down 47
edge 41 left same
edge 41 right 69
edge 47 up 41
edge 47 down 53
edge 47 left same
edge 47 right 69
edge 53 up 47
edge 53 down 60
edge 53 left same
edge 53 right 69
edge 60 up 53
edge 60 down same
edge 60 left same
edge 60 right 25
edge 69 up 76
edge 69 down path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/1/1/0/1/0/3/0/2
edge 69 left 25
edge 69 right same
edge 76 up same
edge 76 down 25
edge 76 left 16
edge 76 right same
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/1/1/0/1/0/3/0/2 up 69
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/1/1/0/1/0/3/0/2 down "item:73/0/0/0/0/0/Reset Quality"
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/1/1/0/1/0/3/0/2 left 41
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/1/1/0/1/0/3/0/2 right same
press pageup focus="Dropdown PART_Facet (Category)" tab="Sega CD" title=Shaders top=ShaderSettingsWindow
press pagedown focus="Dropdown PART_Facet (Category)" tab=PlayStation title=Shaders top=ShaderSettingsWindow
press first focus="Dropdown PART_Facet (Category)" tab=PlayStation title=Shaders top=ShaderSettingsWindow
press last focus="Dropdown PART_Facet (Category)" tab=PlayStation title=Shaders top=ShaderSettingsWindow
press back top=none
