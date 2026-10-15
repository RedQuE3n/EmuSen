# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ThemeBrowserLongName-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 144,37,1632,1041
title "ES-DE Themes"
case ThemeBrowserLongName
palette #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=144,187,1632,861 by=panel limit=144,187,1632,861 paints=#9C9CA0FF role=none
    1 DockPanel at=180,205,1560,825 by=panel limit=144,187,1632,861 role=none
      2 StackPanel at=180,205,1560,118.5 by=panel limit=144,187,1632,861 role=none
        3 DockPanel at=180,205,1560,66 by=panel limit=144,187,1632,861 role=none
          4 Button at=1570.5,205,169.5,66 by=panel drawn focus heard=REFRESH limit=144,187,1632,861 name=ThemeBrowserRefresh op paints=#55555AFF,#9C9CA0FF role=button text=Refresh
            5 TextBlock at=1596,214,118.5,48 box=79,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REFRESH layout=78.94,31.2 limit=1570.5,205,169.5,66 owner=4 paints=#9C9CA0FF role=text size=26 text=REFRESH
          6 TextBlock name=ThemeBrowserFetched paints=#9C9CA0FF role=text volatile="says when the theme list was fetched, by the wall clock"
        7 TextBlock at=180,280,1560,43.5 by=panel drawn limit=144,187,1632,861 name=ThemeBrowserStatus paints=#9C9CA0FF role=text
      8 StackPanel at=795,964,328.5,66 by=panel limit=144,187,1632,861 role=none
        9 Button at=795,964,181.5,66 by=panel drawn focus heard="DETAILS…" limit=144,187,1632,861 name=ThemeBrowserDetails op paints=#55555AFF,#9C9CA0FF role=button text="Details…"
          10 TextBlock at=820.5,973,130.5,48 box=87,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="DETAILS…" layout=86.63,31.2 limit=795,964,181.5,66 owner=9 paints=#9C9CA0FF role=text size=26 text="DETAILS…"
        11 Button at=988.5,964,135,66 by=panel drawn focus heard=CLOSE limit=144,187,1632,861 op paints=#55555AFF,#9C9CA0FF role=button text=Close
          12 TextBlock at=1014,973,84,48 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=55.51,31.2 limit=988.5,964,135,66 owner=11 paints=#9C9CA0FF role=text size=26 text=CLOSE
      13 Grid at=180,341.5,1560,604.5 by=panel limit=144,187,1632,861 role=none
        14 LunaList at=180,341.5,609,604.5 by=panel limit=144,187,1632,861 name=ThemeBrowserList paints=#FFFFFFFF role=list
          15 ScrollViewer at=180,341.5,609,604.5 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=180,341.5,609,604.5
            16 ListBoxItem at=180,341.5,609,63 by=fixed-scroll drawn focus heard="Synthetic Book" in=15 limit=180,341.5,609,604.5 op paints=#333337FF,#9C9CA0FF role=listitem text="Synthetic Book"
              17 TextBlock at=195,350.5,579,43.5 box=386,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Synthetic Book" in=15 layout=121.44,28.8 limit=180,341.5,609,63 owner=16 paints=#9C9CA0FF role=text size=24 text="Synthetic Book"
            18 ListBoxItem at=180,404.5,609,63 by=fixed-scroll drawn focus heard="Plain Shelf" in=15 limit=180,341.5,609,604.5 op paints=#333337FF,#9C9CA0FF role=listitem text="Plain Shelf"
              19 TextBlock at=195,413.5,579,43.5 box=386,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Plain Shelf" in=15 layout=85.75,28.8 limit=180,404.5,609,63 owner=18 paints=#9C9CA0FF role=text size=24 text="Plain Shelf"
            20 ListBoxItem at=180,467.5,609,63 by=fixed-scroll drawn focus heard="Lab Wheel   ·   New" in=15 limit=180,341.5,609,604.5 op paints=#333337FF,#9C9CA0FF role=listitem text="Lab Wheel   ·   New"
              21 TextBlock at=195,476.5,579,43.5 box=386,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Lab Wheel   ·   New" in=15 layout=151.63,28.8 limit=180,467.5,609,63 owner=20 paints=#9C9CA0FF role=text size=24 text="Lab Wheel   ·   New"
            22 ListBoxItem at=180,530.5,609,144 by=fixed-scroll drawn focus heard="The Complete Illustrated Library of Every Console, Anniversary Deluxe Edition   ·   Installed" in=15 limit=180,341.5,609,604.5 op paints=#333337FF,#9C9CA0FF role=listitem text="The Complete Illustrated Library of Every Console, Anniversary Deluxe Edition   ·   Installed"
              23 TextBlock at=195,536.5,579,130.5 box=386,87 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="The Complete Illustrated Library of Every Console, Anniversary Deluxe Edition   ·   Installed" in=15 layout=332.52,86.4 limit=180,530.5,609,144 owner=22 paints=#EEEEF0FF role=text size=24 text="The Complete Illustrated Library of Every Console, Anniversary Deluxe Edition   ·   Installed"
        24 ScrollViewer at=825,341.5,915,604.5 by=panel fades=b limit=144,187,1632,861 name=ThemeBrowserPreview overflow=147 paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=scrollviewer viewport=825,341.5,915,604.5
          25 StackPanel at=825,341.5,915,751.5 by=scroll in=24 limit=825,341.5,915,604.5 paints=#9C9CA0FF role=none seen=825,341.5,915,604.5
            26 FittedImage at=825,341.5,915,285 by=scroll drawn in=24 limit=825,341.5,915,604.5 name=ThemeBrowserScreenshot role=image
            27 TextBlock at=825,638.5,915,87 box=610,58 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="The Complete Illustrated Library of Every Console, Anniversary Deluxe Edition, screenshot 1" in=24 layout=564.55,57.6 limit=825,341.5,915,604.5 name=ThemeBrowserCaption paints=#9C9CA0FF role=text size=24 text="The Complete Illustrated Library of Every Console, Anniversary Deluxe Edition, screenshot 1"
            28 TextBlock at=825,737.5,915,102 box=610,68 by=scroll caps=19.5986 drawn face="Barlow Condensed" heard="The Complete Illustrated Library of Every Console, Anniversary Deluxe Edition" in=24 layout=590.02,67.2 limit=825,341.5,915,604.5 name=ThemeBrowserName paints=#9C9CA0FF role=text size=28 text="The Complete Illustrated Library of Every Console, Anniversary Deluxe Edition"
            29 TextBlock at=825,851.5,915,130.5 box=610,87 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="by Alexandra Margaretha van der Westhuizen-Oyelaran, with the Community Artwork Collective   ·   github.com/community-artwork-collective/complete-illustrated-library-anniversary-deluxe-es-de" in=24 layout=543.96,86.4 limit=825,341.5,915,604.5 name=ThemeBrowserAuthor paints=#9C9CA0FF role=text seen=825,851.5,915,94.5 size=24 text="by Alexandra Margaretha van der Westhuizen-Oyelaran, with the Community Artwork Collective   ·   github.com/community-artwork-collective/complete-illustrated-library-anniversary-deluxe-es-de"
focus 22
edge 4 up same
edge 4 down 16
edge 4 left 9
edge 4 right same
edge 9 up 22
edge 9 down same
edge 9 left 22
edge 9 right 11
edge 11 up 22
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
edge 20 down 22
edge 20 left same
edge 20 right 4
edge 22 up 20
edge 22 down 9
edge 22 left same
edge 22 right 11
press pageup focus="ListBoxItem 'Synthetic Book'" title="ES-DE Themes" top=ThemeBrowserWindow
press pagedown focus="ListBoxItem 'The Complete Illustrated Library of Every Console, Anniversary Deluxe Edition   ·   Installed'" title="ES-DE Themes" top=ThemeBrowserWindow
press first focus="ListBoxItem 'The Complete Illustrated Library of Every Console, Anniversary Deluxe Edition   ·   Installed'" title="ES-DE Themes" top=ThemeBrowserWindow
press last focus="ListBoxItem 'The Complete Illustrated Library of Every Console, Anniversary Deluxe Edition   ·   Installed'" title="ES-DE Themes" top=ThemeBrowserWindow
press back focus="Dropdown TransitionsDropdown" tab=Options top=ThemeSettingsWindow
