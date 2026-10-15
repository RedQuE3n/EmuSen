# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state BigPicture-System-1280x800
size 1280 800
kind scene
root Panel
unit 1
slack 1.5
floor 0
frame 0,0,1280,800
case System
palette #000000FF #007ACCFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #6E6E6EFF #808080FF #9CDCFEFF #CD5C5CFF #D08770FF #D4D4D4FF #DAA520FF #DCDCDCFF #FF4500FF #FFD700FF #FFFFFFFF
view system nes ""
tree
  0 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
    1 Panel at=0,0,1280,800 by=clip limit=0,0,1280,800 role=none
      2 NormalizedCanvas at=0,0,1280,800 by=clip limit=0,0,1280,800 role=group
        3 ImageCarousel at=0,120,1280,480 by=clip element=carousel:systemcarousel limit=0,0,1280,800 role=list
          4 FittedImage at=182.33,298,62,124 by=clip drawn limit=0,120,1280,480 role=image
          5 FittedImage at=1035.67,298,62,124 by=clip drawn limit=0,120,1280,480 role=image
          6 FittedImage at=602.8,285.6,74.4,148.8 by=clip drawn limit=0,120,1280,480 role=image
        7 HintBar at=38.4,736,465.51,24 by=clip element=helpsystem:help heard="D-pad left and right System, A Select, X Screensaver, Menu Menu" limit=0,0,1280,800 role=group
        8 DeviceStatusBar at=1228.96,12.8,28,28 by=clip element=systemstatus:(default) heard="Bluetooth on" limit=0,0,1280,800 role=statusbar
      9 ScrollLetterOverlay at=0,0,1280,800 by=clip limit=0,0,1280,800 nohit role=text
focus none
press right game="" sheet=none system=gb view=system
press left game="" sheet=none system=nes view=system
press pagedown game="" sheet=none system=nes view=system
press pageup game="" sheet=none system=nes view=system
press accept game="Fable of Tiles (Synthetic)" sheet=none system=nes view=gamelist
press back game="" sheet=none system=nes view=system
