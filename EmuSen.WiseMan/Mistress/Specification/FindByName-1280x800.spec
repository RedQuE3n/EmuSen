# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state FindByName-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 220,168,840,464
title "Find by Name"
case FindByName
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=220,268,840,344 by=panel limit=220,268,840,344 paints=#9C9CA0FF role=none
    1 StackPanel at=260,296,760,300 by=panel limit=220,268,840,344 paints=#9C9CA0FF role=none
      2 StackPanel at=260,296,760,232 by=panel limit=220,268,840,344 role=none
        3 TextBlock at=260,296,760,24 box=760,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard="Cobalt Harbor (Synthetic)" layout=169.2,24 limit=220,268,840,344 name=FindByNameTitle paints=#9C9CA0FF role=text size=20 text="Cobalt Harbor (Synthetic)"
        4 HintText at=260,326,760,96 box=760,96 by=panel caps=13.999 drawn face="Barlow Condensed" heard="Searches ScreenScraper by name for this game's console, for a file its hashes did not find. Nothing is sent until you press Search; each search is one request, and one that finds nothing also counts against today's allowance of unrecognised games. Choosing a game keeps its text for this file and fetches its pictures, up to 4 more requests; the file itself is not renamed." layout=752.16,96 limit=220,268,840,344 name=FindByNameHint paints=#86868AFF role=text size=20 text="Searches ScreenScraper by name for this game's console, for a file its hashes did not find. Nothing is sent until you press Search; each search is one request, and one that finds nothing also counts against today's allowance of unrecognised games. Choosing a game keeps its text for this file and fetches its pictures, up to 4 more requests; the file itself is not renamed."
        5 Grid at=260,428,760,46 by=panel limit=220,268,840,344 role=none
          6 TextBox at=260,428,649,46 by=panel drawn focus heard="The game's name" limit=220,268,840,344 name=FindByNameText op paints=#050507FF,#0E0E10FF,#55555AFF,#EEEEF0FF role=edit text="Cobalt Harbor"
            7 ScrollViewer allow="a text box scrolls its own text" at=271,435,627,32 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
          8 Button at=917,429,103,44 by=panel drawn heard=SEARCH limit=220,268,840,344 name=FindByNameSearch off paints=#55555AFF,#9C9CA0FF role=button text=Search
            9 TextBlock at=934,435,69,32 box=69,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=SEARCH layout=68.46,31.2 limit=917,429,103,44 off owner=8 paints=#55555AFF role=text size=26 text=SEARCH
        10 HintText at=260,480,760,48 box=760,48 by=panel caps=13.999 drawn face="Barlow Condensed" heard="ScreenScraper cannot be asked: this build carries no developer credentials and EmuSen's developer file is not on this computer." layout=744.52,48 limit=220,268,840,344 name=FindByNameStatus paints=#86868AFF role=text size=20 text="ScreenScraper cannot be asked: this build carries no developer credentials and EmuSen's developer file is not on this computer."
      11 ButtonBar at=917,552,103,44 by=panel limit=220,268,840,344 paints=#9C9CA0FF role=toolbar
        12 Button at=917,552,103,44 by=clip drawn focus heard=CANCEL limit=917,552,103,44 name=FindByNameCancel op paints=#55555AFF,#9C9CA0FF role=button text=Cancel
          13 TextBlock at=934,558,69,32 box=69,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CANCEL layout=68.02,31.2 limit=917,552,103,44 owner=12 paints=#9C9CA0FF role=text size=26 text=CANCEL
focus 6
edge 6 up same
edge 6 down 12
edge 6 left same
edge 6 right 12
edge 12 up 6
edge 12 down same
edge 12 left 6
edge 12 right same
press pageup focus="TextBox FindByNameText (The game's name)" title="Find by Name" top=FindByNameWindow
press pagedown focus="TextBox FindByNameText (The game's name)" title="Find by Name" top=FindByNameWindow
press first focus="TextBox FindByNameText (The game's name)" title="Find by Name" top=FindByNameWindow
press last focus="TextBox FindByNameText (The game's name)" title="Find by Name" top=FindByNameWindow
press back top=none
