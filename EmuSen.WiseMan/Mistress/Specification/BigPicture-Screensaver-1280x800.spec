# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state BigPicture-Screensaver-1280x800
size 1280 800
kind overlay
root Screensaver#Screensaver
unit 1
slack 1.5
floor 0
frame 0,0,1280,800
case Screensaver
palette #000000FF #007ACCFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #6E6E6EFF #808080FF #9CDCFEFF #CD5C5CFF #D08770FF #D4D4D4FF #DAA520FF #DCDCDCFF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the pad moves this view's own selection, not the focus
tree
  0 Border at=0,0,1280,800 by=panel limit=0,0,1280,800 paints=#000000FF role=none
  1 CrossFadeImage at=0,0,1280,800 by=panel limit=0,0,1280,800 nohit role=none
    2 FittedImage at=0,0,1280,800 by=panel drawn limit=0,0,1280,800 nohit role=image
  3 Canvas at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
    4 Border at=17,16,262,84 by=panel limit=0,0,1280,800 paints=#000000AA role=none
    5 FontText at=31,23,231,28 by=panel heard="Dune Relay (Synthetic)" limit=0,0,1280,800 role=text
    6 FontText at=31,65,160,28 by=panel heard="Super Nintendo" limit=0,0,1280,800 role=text
focus none
