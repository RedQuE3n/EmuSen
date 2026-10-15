# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state CoverPickerLongTitle-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 330,310.5,1260,579
title "Use Another Game's Cover"
case CoverPickerLongTitle
palette #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=330,460,1260,399 by=panel limit=330,460,1260,399 paints=#9C9CA0FF role=none
    1 StackPanel at=390,502,1140,333 by=panel limit=330,460,1260,399 role=none
      2 StackPanel at=390,502,1140,195 by=panel limit=330,460,1260,399 role=none
        3 TextBlock at=390,502,1140,36 box=760,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard="A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta)" layout=687.74,24 limit=330,460,1260,399 name=CoverPickerTitle paints=#9C9CA0FF role=text size=20 text="A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta)"
        4 HintText at=390,547,1140,72 box=760,48 by=panel caps=13.999 drawn face="Barlow Condensed" heard="Choose a game of your library; this game then shows its cover. Nothing is copied or sent, and Use Its Own Cover undoes it." layout=756.5,48 limit=330,460,1260,399 name=CoverPickerHint paints=#86868AFF role=text size=20 text="Choose a game of your library; this game then shows its cover. Nothing is copied or sent, and Use Its Own Cover undoes it."
        5 TextBox at=390,628,1140,69 by=panel drawn focus limit=330,460,1260,399 name=CoverPickerSearch op paints=#050507FF,#0E0E10FF,#55555AFF,#EEEEF0FF role=edit
          6 ScrollViewer allow="a text box scrolls its own text" at=406.5,638.5,1107,48 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
            7 TextBlock allow="a text box scrolls its own text" at=406.5,638.5,1107,48 box=738,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search the library" in=6 layout=156.83,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text="Search the library"
      8 ScrollViewer at=390,715,1140,36 by=panel limit=330,460,1260,399 name=CoverPickerScroll overflow=0 paints=#9C9CA0FF role=scrollviewer viewport=390,715,1140,36
        9 StackPanel at=390,715,1140,36 by=fixed-scroll in=8 limit=390,715,1140,36 role=none
          10 HintText at=390,715,1140,36 box=760,24 by=fixed-scroll caps=13.999 drawn face="Barlow Condensed" heard="No other game in the library has a cover yet." in=8 layout=300.62,24 limit=390,715,1140,36 name=CoverPickerEmpty paints=#86868AFF role=text size=20 text="No other game in the library has a cover yet."
      11 ButtonBar at=1375.5,769,154.5,66 by=panel limit=330,460,1260,399 paints=#9C9CA0FF role=toolbar
        12 Button at=1375.5,769,154.5,66 by=clip drawn focus heard=CANCEL limit=1375.5,769,154.5,66 name=CoverPickerCancel op paints=#55555AFF,#9C9CA0FF role=button text=Cancel
          13 TextBlock at=1401,778,103.5,48 box=69,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CANCEL layout=68.02,31.2 limit=1375.5,769,154.5,66 owner=12 paints=#9C9CA0FF role=text size=26 text=CANCEL
focus 5
edge 5 up same
edge 5 down 12
edge 5 left same
edge 5 right 12
edge 12 up 5
edge 12 down same
edge 12 left 5
edge 12 right same
press pageup focus="TextBox CoverPickerSearch" title="Use Another Game's Cover" top=CoverPickerWindow
press pagedown focus="TextBox CoverPickerSearch" title="Use Another Game's Cover" top=CoverPickerWindow
press first focus="TextBox CoverPickerSearch" title="Use Another Game's Cover" top=CoverPickerWindow
press last focus="TextBox CoverPickerSearch" title="Use Another Game's Cover" top=CoverPickerWindow
press back top=none
