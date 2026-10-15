# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ResumeLongTitle-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 350.48,126.5,1219.05,908
title "Continue Playing"
case ResumeLongTitle
palette #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=350,276,1221,729 by=panel limit=350.48,276,1219.05,728 paints=#9C9CA0FF role=none seen=350.48,276,1219.05,728
    1 StackPanel at=410,318,1101,663 by=panel limit=350.48,276,1219.05,728 role=none
      2 Image at=755,318,411.43,360 by=panel drawn limit=350.48,276,1219.05,728 role=image
      3 TextBlock at=410,696,1101,43.5 box=734,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Would you like to continue your last game?" layout=346.8,28.8 limit=350.48,276,1219.05,728 paints=#9C9CA0FF role=text size=24 text="Would you like to continue your last game?"
      4 HintText at=410,757.5,1101,72 box=734,48 by=panel caps=13.999 drawn face="Barlow Condensed" heard="A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta) was left on Thursday 15 October, 09:30. Resume where you left off, or start it again." layout=718.44,48 limit=350.48,276,1219.05,728 paints=#86868AFF role=text size=20 text="A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta) was left on Thursday 15 October, 09:30. Resume where you left off, or start it again."
      5 LunaSwitch at=410,847.5,291,48 by=panel drawn focus heard="Do not ask again" limit=350.48,276,1219.05,728 name=RememberResumeSwitch op paints=#86868AFF,#9C9CA0FF role=button
        6 TextBlock at=488,850.5,204,43.5 box=136,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Do not ask again" layout=135.58,28.8 limit=410,847.5,291,48 owner=5 paints=#9C9CA0FF role=text size=24 text="Do not ask again"
      7 ButtonBar at=1175,913.5,336,66 by=panel limit=350.48,276,1219.05,728 paints=#9C9CA0FF role=toolbar
        8 Button at=1175,913.5,157.5,66 by=clip drawn focus heard=RESUME limit=1175,913.5,336,66 name=ResumeButton op paints=#55555AFF,#9C9CA0FF role=button text=Resume
          9 TextBlock at=1200.5,922.5,106.5,48 box=71,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=RESUME layout=70.56,31.2 limit=1175,913.5,157.5,66 owner=8 paints=#EEEEF0FF role=text size=26 text=RESUME
        10 Button at=1344.5,913.5,166.5,66 by=clip drawn focus heard=RESTART limit=1175,913.5,336,66 name=RestartButton op paints=#55555AFF,#9C9CA0FF role=button text=Restart
          11 TextBlock at=1370,922.5,115.5,48 box=77,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=RESTART layout=76.41,31.2 limit=1344.5,913.5,166.5,66 owner=10 paints=#9C9CA0FF role=text size=26 text=RESTART
focus 8
edge 5 up same
edge 5 down 8
edge 5 left same
edge 5 right 8
edge 8 up 5
edge 8 down same
edge 8 left 5
edge 8 right 10
edge 10 up 5
edge 10 down same
edge 10 left 8
edge 10 right same
press pageup focus="Button 'Resume'" title="Continue Playing" top=ResumeWindow
press pagedown focus="Button 'Resume'" title="Continue Playing" top=ResumeWindow
press first focus="Button 'Resume'" title="Continue Playing" top=ResumeWindow
press last focus="Button 'Resume'" title="Continue Playing" top=ResumeWindow
press back top=none
