# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ThemeBrowser-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 144,71,1632,1007
title "ES-DE Themes"
case ThemeBrowser
entered down
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=144,221,1632,828 by=panel limit=144,221,1632,827 paints=#D4D4D4FF role=none seen=144,221,1632,827
    1 DockPanel at=180,239,1560,792 by=panel limit=144,221,1632,827 role=none
      2 StackPanel at=180,239,1560,118.5 by=panel limit=144,221,1632,827 role=none
        3 DockPanel at=180,239,1560,66 by=panel limit=144,221,1632,827 role=none
          4 Button at=1576.5,239,163.5,66 by=panel drawn focus heard=REFRESH limit=144,221,1632,827 name=ThemeBrowserRefresh op paints=#707070FF,#D4D4D4FF role=button text=Refresh
            5 TextBlock at=1602,248,112.5,48 box=75,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REFRESH layout=74.7,31.2 limit=1576.5,239,163.5,66 owner=4 paints=#EEEEF0FF role=text size=26 text=REFRESH
          6 TextBlock name=ThemeBrowserFetched paints=#D4D4D4FF role=text volatile="says when the theme list was fetched, by the wall clock"
        7 TextBlock at=180,314,1560,43.5 by=panel drawn limit=144,221,1632,827 name=ThemeBrowserStatus paints=#D4D4D4FF role=text
      8 StackPanel at=801,965,318,66 by=panel limit=144,221,1632,827 role=none
        9 Button at=801,965,175.5,66 by=panel drawn focus heard="DETAILS…" limit=144,221,1632,827 name=ThemeBrowserDetails op paints=#707070FF,#D4D4D4FF role=button text="Details…"
          10 TextBlock at=826.5,974,124.5,48 box=83,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="DETAILS…" layout=82.08,31.2 limit=801,965,175.5,66 owner=9 paints=#D4D4D4FF role=text size=26 text="DETAILS…"
        11 Button at=988.5,965,130.5,66 by=panel drawn focus heard=CLOSE limit=144,221,1632,827 op paints=#707070FF,#D4D4D4FF role=button text=Close
          12 TextBlock at=1014,974,79.5,48 box=53,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=52.75,31.2 limit=988.5,965,130.5,66 owner=11 paints=#D4D4D4FF role=text size=26 text=CLOSE
      13 Grid at=180,375.5,1560,571.5 by=panel limit=144,221,1632,827 role=none
        14 LunaList at=180,375.5,609,571.5 by=panel limit=144,221,1632,827 name=ThemeBrowserList paints=#FFFFFFFF role=list
          15 ScrollViewer at=180,375.5,609,571.5 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=180,375.5,609,571.5
            16 ListBoxItem at=180,375.5,609,63 by=fixed-scroll drawn focus heard="Synthetic Book" in=15 limit=180,375.5,609,571.5 op paints=#333337FF,#D4D4D4FF role=listitem text="Synthetic Book"
              17 TextBlock at=195,384.5,579,43.5 box=386,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Synthetic Book" in=15 layout=114.55,28.8 limit=180,375.5,609,63 owner=16 paints=#EEEEF0FF role=text size=24 text="Synthetic Book"
            18 ListBoxItem at=180,438.5,609,63 by=fixed-scroll drawn focus heard="Plain Shelf   ·   Installed" in=15 limit=180,375.5,609,571.5 op paints=#333337FF,#D4D4D4FF role=listitem text="Plain Shelf   ·   Installed"
              19 TextBlock at=195,447.5,579,43.5 box=386,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Plain Shelf   ·   Installed" in=15 layout=179.4,28.8 limit=180,438.5,609,63 owner=18 paints=#D4D4D4FF role=text size=24 text="Plain Shelf   ·   Installed"
            20 ListBoxItem at=180,501.5,609,63 by=fixed-scroll drawn focus heard="Lab Wheel   ·   New" in=15 limit=180,375.5,609,571.5 op paints=#333337FF,#D4D4D4FF role=listitem text="Lab Wheel   ·   New"
              21 TextBlock at=195,510.5,579,43.5 box=386,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Lab Wheel   ·   New" in=15 layout=147.22,28.8 limit=180,501.5,609,63 owner=20 paints=#D4D4D4FF role=text size=24 text="Lab Wheel   ·   New"
        22 ScrollViewer at=825,375.5,915,571.5 by=panel limit=144,221,1632,827 name=ThemeBrowserPreview overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=825,375.5,915,571.5
          23 StackPanel at=825,375.5,915,571.5 by=fixed-scroll in=22 limit=825,375.5,915,571.5 role=none
            24 FittedImage at=825,375.5,915,285 by=fixed-scroll drawn in=22 limit=825,375.5,915,571.5 name=ThemeBrowserScreenshot role=image
            25 TextBlock at=825,672.5,915,43.5 box=610,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Synthetic Book, screenshot 1" in=22 layout=220.46,28.8 limit=825,375.5,915,571.5 name=ThemeBrowserCaption paints=#D4D4D4FF role=text size=24 text="Synthetic Book, screenshot 1"
            26 TextBlock at=825,728,915,51 box=610,34 by=fixed-scroll caps=19.5986 drawn face="Barlow Condensed" heard="Synthetic Book" in=22 layout=133.64,33.6 limit=825,375.5,915,571.5 name=ThemeBrowserName paints=#D4D4D4FF role=text size=28 text="Synthetic Book"
            27 TextBlock at=825,791,915,43.5 box=610,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="by Ada Example   ·   github.com/ada/synthetic-book-es-de" in=22 layout=455.23,28.8 limit=825,375.5,915,571.5 name=ThemeBrowserAuthor paints=#D4D4D4FF role=text size=24 text="by Ada Example   ·   github.com/ada/synthetic-book-es-de"
            28 TextBlock at=825,846.5,915,43.5 box=610,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="2 variants · 3 colour schemes · 3 aspect ratios" in=22 layout=357.36,28.8 limit=825,375.5,915,571.5 name=ThemeBrowserSupports paints=#D4D4D4FF role=text size=24 text="2 variants · 3 colour schemes · 3 aspect ratios"
            29 TextBlock at=825,902,915,43.5 box=610,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Not installed" in=22 layout=96.19,28.8 limit=825,375.5,915,571.5 name=ThemeBrowserState paints=#D4D4D4FF role=text size=24 text="Not installed"
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
press back focus="Dropdown TransitionsDropdown" tab=Options title="Theme Settings" top=ThemeSettingsWindow
