# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ThemeBrowser-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 96,47,1088,671
title "ES-DE Themes"
case ThemeBrowser
entered down
palette #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=96,147,1088,551 by=panel limit=96,147,1088,551 paints=#9C9CA0FF role=none
    1 DockPanel at=120,159,1040,527 by=panel limit=96,147,1088,551 role=none
      2 StackPanel at=120,159,1040,79 by=panel limit=96,147,1088,551 role=none
        3 DockPanel at=120,159,1040,44 by=panel limit=96,147,1088,551 role=none
          4 Button at=1047,159,113,44 by=panel drawn focus heard=REFRESH limit=96,147,1088,551 name=ThemeBrowserRefresh op paints=#55555AFF,#9C9CA0FF role=button text=Refresh
            5 TextBlock at=1064,165,79,32 box=79,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REFRESH layout=78.94,31.2 limit=1047,159,113,44 owner=4 paints=#EEEEF0FF role=text size=26 text=REFRESH
          6 TextBlock name=ThemeBrowserFetched paints=#9C9CA0FF role=text volatile="says when the theme list was fetched, by the wall clock"
        7 TextBlock at=120,209,1040,29 by=panel drawn limit=96,147,1088,551 name=ThemeBrowserStatus paints=#9C9CA0FF role=text
      8 StackPanel at=530,642,219,44 by=panel limit=96,147,1088,551 role=none
        9 Button at=530,642,121,44 by=panel drawn focus heard="DETAILS…" limit=96,147,1088,551 name=ThemeBrowserDetails op paints=#55555AFF,#9C9CA0FF role=button text="Details…"
          10 TextBlock at=547,648,87,32 box=87,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="DETAILS…" layout=86.63,31.2 limit=530,642,121,44 owner=9 paints=#9C9CA0FF role=text size=26 text="DETAILS…"
        11 Button at=659,642,90,44 by=panel drawn focus heard=CLOSE limit=96,147,1088,551 op paints=#55555AFF,#9C9CA0FF role=button text=Close
          12 TextBlock at=676,648,56,32 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=55.51,31.2 limit=659,642,90,44 owner=11 paints=#9C9CA0FF role=text size=26 text=CLOSE
      13 Grid at=120,250,1040,380 by=panel limit=96,147,1088,551 role=none
        14 LunaList at=120,250,406,380 by=panel limit=96,147,1088,551 name=ThemeBrowserList paints=#FFFFFFFF role=list
          15 ScrollViewer at=120,250,406,380 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=120,250,406,380
            16 ListBoxItem at=120,250,406,42 by=fixed-scroll drawn focus heard="Synthetic Book" in=15 limit=120,250,406,380 op paints=#333337FF,#9C9CA0FF role=listitem text="Synthetic Book"
              17 TextBlock at=130,256,386,29 box=386,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Synthetic Book" in=15 layout=121.44,28.8 limit=120,250,406,42 owner=16 paints=#EEEEF0FF role=text size=24 text="Synthetic Book"
            18 ListBoxItem at=120,292,406,42 by=fixed-scroll drawn focus heard="Plain Shelf   ·   Installed" in=15 limit=120,250,406,380 op paints=#333337FF,#9C9CA0FF role=listitem text="Plain Shelf   ·   Installed"
              19 TextBlock at=130,298,386,29 box=386,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Plain Shelf   ·   Installed" in=15 layout=188.5,28.8 limit=120,292,406,42 owner=18 paints=#9C9CA0FF role=text size=24 text="Plain Shelf   ·   Installed"
            20 ListBoxItem at=120,334,406,42 by=fixed-scroll drawn focus heard="Lab Wheel   ·   New" in=15 limit=120,250,406,380 op paints=#333337FF,#9C9CA0FF role=listitem text="Lab Wheel   ·   New"
              21 TextBlock at=130,340,386,29 box=386,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Lab Wheel   ·   New" in=15 layout=151.63,28.8 limit=120,334,406,42 owner=20 paints=#9C9CA0FF role=text size=24 text="Lab Wheel   ·   New"
        22 ScrollViewer at=550,250,610,380 by=panel limit=96,147,1088,551 name=ThemeBrowserPreview overflow=0 paints=#9C9CA0FF role=scrollviewer viewport=550,250,610,380
          23 StackPanel at=550,250,610,380 by=fixed-scroll in=22 limit=550,250,610,380 role=none
            24 FittedImage at=550,250,610,190 by=fixed-scroll drawn in=22 limit=550,250,610,380 name=ThemeBrowserScreenshot role=image
            25 TextBlock at=550,448,610,29 box=610,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Synthetic Book, screenshot 1" in=22 layout=232.2,28.8 limit=550,250,610,380 name=ThemeBrowserCaption paints=#9C9CA0FF role=text size=24 text="Synthetic Book, screenshot 1"
            26 TextBlock at=550,485,610,34 box=610,34 by=fixed-scroll caps=19.5986 drawn face="Barlow Condensed" heard="Synthetic Book" in=22 layout=141.68,33.6 limit=550,250,610,380 name=ThemeBrowserName paints=#9C9CA0FF role=text size=28 text="Synthetic Book"
            27 TextBlock at=550,527,610,29 box=610,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="by Ada Example   ·   github.com/ada/synthetic-book-es-de" in=22 layout=472.58,28.8 limit=550,250,610,380 name=ThemeBrowserAuthor paints=#9C9CA0FF role=text size=24 text="by Ada Example   ·   github.com/ada/synthetic-book-es-de"
            28 TextBlock at=550,564,610,29 box=610,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="2 variants · 3 colour schemes · 3 aspect ratios" in=22 layout=374.21,28.8 limit=550,250,610,380 name=ThemeBrowserSupports paints=#9C9CA0FF role=text size=24 text="2 variants · 3 colour schemes · 3 aspect ratios"
            29 TextBlock at=550,601,610,29 box=610,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Not installed" in=22 layout=102.07,28.8 limit=550,250,610,380 name=ThemeBrowserState paints=#9C9CA0FF role=text size=24 text="Not installed"
focus 4
edge 4 up same
edge 4 down 16
edge 4 left 9
edge 4 right same
edge 9 up 20
edge 9 down same
edge 9 left 20
edge 9 right 11
edge 11 up 20
edge 11 down same
edge 11 left 9
edge 11 right 4
edge 16 up 4
edge 16 down 18
edge 16 left same
edge 16 right 4
edge 18 up 16
edge 18 down 20
edge 18 left same
edge 18 right 4
edge 20 up 18
edge 20 down 9
edge 20 left same
edge 20 right 4
press pageup focus="Button 'Refresh'" title="ES-DE Themes" top=ThemeBrowserWindow
press pagedown focus="Button 'Refresh'" title="ES-DE Themes" top=ThemeBrowserWindow
press first focus="Button 'Refresh'" title="ES-DE Themes" top=ThemeBrowserWindow
press last focus="Button 'Refresh'" title="ES-DE Themes" top=ThemeBrowserWindow
press back focus="Dropdown TransitionsDropdown" tab=Options top=ThemeSettingsWindow
