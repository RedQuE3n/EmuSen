# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state CoverPicker-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 330,310.5,1260,579
title "Use Another Game's Cover"
case CoverPicker
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=330,460,1260,399 by=panel limit=330,460,1260,399 paints=#D4D4D4FF role=none
    1 StackPanel at=390,502,1140,333 by=panel limit=330,460,1260,399 role=none
      2 StackPanel at=390,502,1140,195 by=panel limit=330,460,1260,399 role=none
        3 TextBlock at=390,502,1140,36 box=760,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard="Cobalt Harbor (Synthetic)" layout=160.24,24 limit=330,460,1260,399 name=CoverPickerTitle paints=#D4D4D4FF role=text size=20 text="Cobalt Harbor (Synthetic)"
        4 HintText at=390,547,1140,72 box=760,48 by=panel caps=13.999 drawn face="Barlow Condensed" heard="Choose a game of your library; this game then shows its cover. Nothing is copied or sent, and Use Its Own Cover undoes it." layout=721.84,48 limit=330,460,1260,399 name=CoverPickerHint paints=#8D8D91FF role=text size=20 text="Choose a game of your library; this game then shows its cover. Nothing is copied or sent, and Use Its Own Cover undoes it."
        5 TextBox at=390,628,1140,69 by=panel drawn focus heard="Search the library" limit=330,460,1260,399 name=CoverPickerSearch op paints=#0E0E10FF,#26262BFF,#4C9AE8FF,#707070FF,#EEEEF0FF role=edit
          6 ScrollViewer allow="a text box scrolls its own text" at=406.5,638.5,1107,48 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
            7 TextBlock allow="a text box scrolls its own text" at=406.5,638.5,1107,48 box=738,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search the library" in=6 layout=148.93,31.2 name=PART_Placeholder paints=#8D8D91FF part role=text size=26 text="Search the library"
      8 ScrollViewer at=390,715,1140,36 by=panel limit=330,460,1260,399 name=CoverPickerScroll overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=390,715,1140,36
        9 StackPanel at=390,715,1140,36 by=fixed-scroll in=8 limit=390,715,1140,36 role=none
          10 HintText at=390,715,1140,36 box=760,24 by=fixed-scroll caps=13.999 drawn face="Barlow Condensed" heard="No other game in the library has a cover yet." in=8 layout=287.3,24 limit=390,715,1140,36 name=CoverPickerEmpty paints=#8D8D91FF role=text size=20 text="No other game in the library has a cover yet."
      11 ButtonBar at=1381.5,769,148.5,66 by=panel limit=330,460,1260,399 paints=#D4D4D4FF role=toolbar
        12 Button at=1381.5,769,148.5,66 by=clip drawn focus heard=CANCEL limit=1381.5,769,148.5,66 name=CoverPickerCancel op paints=#707070FF,#D4D4D4FF role=button text=Cancel
          13 TextBlock at=1407,778,97.5,48 box=65,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CANCEL layout=64.74,31.2 limit=1381.5,769,148.5,66 owner=12 paints=#D4D4D4FF role=text size=26 text=CANCEL
focus 5
edge 5 up same
edge 5 down 12
edge 5 left same
edge 5 right 12
edge 12 up 5
edge 12 down same
edge 12 left 5
edge 12 right same
press pageup focus="TextBox CoverPickerSearch (Search the library)" title="Use Another Game's Cover" top=CoverPickerWindow
press pagedown focus="TextBox CoverPickerSearch (Search the library)" title="Use Another Game's Cover" top=CoverPickerWindow
press first focus="TextBox CoverPickerSearch (Search the library)" title="Use Another Game's Cover" top=CoverPickerWindow
press last focus="TextBox CoverPickerSearch (Search the library)" title="Use Another Game's Cover" top=CoverPickerWindow
press back top=none
