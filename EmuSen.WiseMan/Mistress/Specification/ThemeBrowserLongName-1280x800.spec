# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ThemeBrowserLongName-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 96,24,1088,694
title "ES-DE Themes"
case ThemeBrowserLongName
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=96,124,1088,574 by=panel limit=96,124,1088,574 paints=#D4D4D4FF role=none
    1 DockPanel at=120,136,1040,550 by=panel limit=96,124,1088,574 role=none
      2 StackPanel at=120,136,1040,79 by=panel limit=96,124,1088,574 role=none
        3 DockPanel at=120,136,1040,44 by=panel limit=96,124,1088,574 role=none
          4 Button at=1051,136,109,44 by=panel drawn focus heard=REFRESH limit=96,124,1088,574 name=ThemeBrowserRefresh op paints=#707070FF,#D4D4D4FF role=button text=Refresh
            5 TextBlock at=1068,142,75,32 box=75,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REFRESH layout=74.7,31.2 limit=1051,136,109,44 owner=4 paints=#D4D4D4FF role=text size=26 text=REFRESH
          6 TextBlock name=ThemeBrowserFetched paints=#D4D4D4FF role=text volatile="says when the theme list was fetched, by the wall clock"
        7 TextBlock at=120,186,1040,29 by=panel drawn limit=96,124,1088,574 name=ThemeBrowserStatus paints=#D4D4D4FF role=text
      8 StackPanel at=534,642,212,44 by=panel limit=96,124,1088,574 role=none
        9 Button at=534,642,117,44 by=panel drawn focus heard="DETAILS…" limit=96,124,1088,574 name=ThemeBrowserDetails op paints=#707070FF,#D4D4D4FF role=button text="Details…"
          10 TextBlock at=551,648,83,32 box=83,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="DETAILS…" layout=82.08,31.2 limit=534,642,117,44 owner=9 paints=#D4D4D4FF role=text size=26 text="DETAILS…"
        11 Button at=659,642,87,44 by=panel drawn focus heard=CLOSE limit=96,124,1088,574 op paints=#707070FF,#D4D4D4FF role=button text=Close
          12 TextBlock at=676,648,53,32 box=53,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=52.75,31.2 limit=659,642,87,44 owner=11 paints=#D4D4D4FF role=text size=26 text=CLOSE
      13 Grid at=120,227,1040,403 by=panel limit=96,124,1088,574 role=none
        14 LunaList at=120,227,406,403 by=panel limit=96,124,1088,574 name=ThemeBrowserList paints=#FFFFFFFF role=list
          15 ScrollViewer at=120,227,406,403 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=120,227,406,403
            16 ListBoxItem at=120,227,406,42 by=fixed-scroll drawn focus heard="Synthetic Book" in=15 limit=120,227,406,403 op paints=#333337FF,#D4D4D4FF role=listitem text="Synthetic Book"
              17 TextBlock at=130,233,386,29 box=386,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Synthetic Book" in=15 layout=114.55,28.8 limit=120,227,406,42 owner=16 paints=#D4D4D4FF role=text size=24 text="Synthetic Book"
            18 ListBoxItem at=120,269,406,42 by=fixed-scroll drawn focus heard="Plain Shelf" in=15 limit=120,227,406,403 op paints=#333337FF,#D4D4D4FF role=listitem text="Plain Shelf"
              19 TextBlock at=130,275,386,29 box=386,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Plain Shelf" in=15 layout=80.78,28.8 limit=120,269,406,42 owner=18 paints=#D4D4D4FF role=text size=24 text="Plain Shelf"
            20 ListBoxItem at=120,311,406,42 by=fixed-scroll drawn focus heard="Lab Wheel   ·   New" in=15 limit=120,227,406,403 op paints=#333337FF,#D4D4D4FF role=listitem text="Lab Wheel   ·   New"
              21 TextBlock at=130,317,386,29 box=386,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Lab Wheel   ·   New" in=15 layout=147.22,28.8 limit=120,311,406,42 owner=20 paints=#D4D4D4FF role=text size=24 text="Lab Wheel   ·   New"
            22 ListBoxItem at=120,353,406,67 by=fixed-scroll drawn focus heard="The Complete Illustrated Library of Every Console, Anniversary Deluxe Edition   ·   Installed" in=15 limit=120,227,406,403 op paints=#333337FF,#D4D4D4FF role=listitem text="The Complete Illustrated Library of Every Console, Anniversary Deluxe Edition   ·   Installed"
              23 TextBlock at=130,357,386,58 box=386,58 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="The Complete Illustrated Library of Every Console, Anniversary Deluxe Edition   ·   Installed" in=15 layout=383.93,57.6 limit=120,353,406,67 owner=22 paints=#EEEEF0FF role=text size=24 text="The Complete Illustrated Library of Every Console, Anniversary Deluxe Edition   ·   Installed"
        24 ScrollViewer at=550,227,610,403 by=panel fades=b limit=96,124,1088,574 name=ThemeBrowserPreview overflow=98 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=550,227,610,403
          25 StackPanel at=550,227,610,501 by=scroll in=24 limit=550,227,610,403 paints=#D4D4D4FF role=none seen=550,227,610,403
            26 FittedImage at=550,227,610,190 by=scroll drawn in=24 limit=550,227,610,403 name=ThemeBrowserScreenshot role=image
            27 TextBlock at=550,425,610,58 box=610,58 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="The Complete Illustrated Library of Every Console, Anniversary Deluxe Edition, screenshot 1" in=24 layout=597.98,57.6 limit=550,227,610,403 name=ThemeBrowserCaption paints=#D4D4D4FF role=text size=24 text="The Complete Illustrated Library of Every Console, Anniversary Deluxe Edition, screenshot 1"
            28 TextBlock at=550,491,610,68 box=610,68 by=scroll caps=19.5986 drawn face="Barlow Condensed" heard="The Complete Illustrated Library of Every Console, Anniversary Deluxe Edition" in=24 layout=560.67,67.2 limit=550,227,610,403 name=ThemeBrowserName paints=#D4D4D4FF role=text size=28 text="The Complete Illustrated Library of Every Console, Anniversary Deluxe Edition"
            29 TextBlock at=550,567,610,87 box=610,87 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="by Alexandra Margaretha van der Westhuizen-Oyelaran, with the Community Artwork Collective   ·   github.com/community-artwork-collective/complete-illustrated-library-anniversary-deluxe-es-de" in=24 layout=589.01,86.4 limit=550,227,610,403 name=ThemeBrowserAuthor paints=#D4D4D4FF role=text seen=550,567,610,63 size=24 text="by Alexandra Margaretha van der Westhuizen-Oyelaran, with the Community Artwork Collective   ·   github.com/community-artwork-collective/complete-illustrated-library-anniversary-deluxe-es-de"
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
edge 22 right 4
press pageup focus="ListBoxItem 'Synthetic Book'" title="ES-DE Themes" top=ThemeBrowserWindow
press pagedown focus="ListBoxItem 'The Complete Illustrated Library of Every Console, Anniversary Deluxe Edition   ·   Installed'" title="ES-DE Themes" top=ThemeBrowserWindow
press first focus="ListBoxItem 'The Complete Illustrated Library of Every Console, Anniversary Deluxe Edition   ·   Installed'" title="ES-DE Themes" top=ThemeBrowserWindow
press last focus="ListBoxItem 'The Complete Illustrated Library of Every Console, Anniversary Deluxe Edition   ·   Installed'" title="ES-DE Themes" top=ThemeBrowserWindow
press back focus="Dropdown TransitionsDropdown (Transitions)" tab=Options top=ThemeSettingsWindow
