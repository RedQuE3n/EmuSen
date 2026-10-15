# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state BigPicture-SearchKeyboard-1280x800
size 1280 800
kind overlay
root OnScreenKeyboard
unit 1
slack 1.5
floor 0
frame 0,0,543,345
case SearchKeyboard
palette #000000FF #007ACCFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #6E6E6EFF #808080FF #9CDCFEFF #CD5C5CFF #D08770FF #D4D4D4FF #DAA520FF #DCDCDCFF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 DockPanel at=17,17,509,312 by=panel limit=0,0,543,345 role=none
    1 TextBlock at=21,17,501,28 box=501,28 by=panel caps=14.2822 drawn face="Noto Sans" heard=| layout=11.02,27.24 limit=0,0,543,345 paints=#FFFFFFFF role=text size=20 text=|
    2 TextBlock at=21,309,501,20 box=501,20 by=panel caps=9.9976 drawn face="Noto Sans" heard="A  Type      B  Erase      Y  Space      Select  Shift      L1 R1  Layout      Start  Done" layout=500.07,19.07 limit=0,0,543,345 paints=#808080FF role=text size=14 text="A  Type      B  Erase      Y  Space      Select  Shift      L1 R1  Layout      Start  Done"
    3 StackPanel at=25,55,494,244 by=panel limit=0,0,543,345 role=none
      4 StackPanel at=25,55,494,44 by=panel limit=0,0,543,345 role=none
        5 Button at=25,55,44,44 by=panel drawn heard=1 limit=0,0,543,345 paints=#007ACCFF,#6E6E6EFF,#FFFFFFFF role=button text=1
          6 AccessText at=42,64,11,25 box=11,25 by=clip caps=12.854 drawn face="Noto Sans" layout=10.3,24.52 limit=25,55,44,44 owner=5 paints=#FFFFFFFF role=none size=18 text=1
        7 Button at=75,55,44,44 by=panel drawn heard=2 limit=0,0,543,345 paints=#252526FF,#6E6E6EFF,#D4D4D4FF role=button text=2
          8 AccessText at=92,64,11,25 box=11,25 by=clip caps=12.854 drawn face="Noto Sans" layout=10.3,24.52 limit=75,55,44,44 owner=7 paints=#D4D4D4FF role=none size=18 text=2
        9 Button at=125,55,44,44 by=panel drawn heard=3 limit=0,0,543,345 paints=#252526FF,#6E6E6EFF,#D4D4D4FF role=button text=3
          10 AccessText at=142,64,11,25 box=11,25 by=clip caps=12.854 drawn face="Noto Sans" layout=10.3,24.52 limit=125,55,44,44 owner=9 paints=#D4D4D4FF role=none size=18 text=3
        11 Button at=175,55,44,44 by=panel drawn heard=4 limit=0,0,543,345 paints=#252526FF,#6E6E6EFF,#D4D4D4FF role=button text=4
          12 AccessText at=192,64,11,25 box=11,25 by=clip caps=12.854 drawn face="Noto Sans" layout=10.3,24.52 limit=175,55,44,44 owner=11 paints=#D4D4D4FF role=none size=18 text=4
        13 Button at=225,55,44,44 by=panel drawn heard=5 limit=0,0,543,345 paints=#252526FF,#6E6E6EFF,#D4D4D4FF role=button text=5
          14 AccessText at=242,64,11,25 box=11,25 by=clip caps=12.854 drawn face="Noto Sans" layout=10.3,24.52 limit=225,55,44,44 owner=13 paints=#D4D4D4FF role=none size=18 text=5
        15 Button at=275,55,44,44 by=panel drawn heard=6 limit=0,0,543,345 paints=#252526FF,#6E6E6EFF,#D4D4D4FF role=button text=6
          16 AccessText at=292,64,11,25 box=11,25 by=clip caps=12.854 drawn face="Noto Sans" layout=10.3,24.52 limit=275,55,44,44 owner=15 paints=#D4D4D4FF role=none size=18 text=6
        17 Button at=325,55,44,44 by=panel drawn heard=7 limit=0,0,543,345 paints=#252526FF,#6E6E6EFF,#D4D4D4FF role=button text=7
          18 AccessText at=342,64,11,25 box=11,25 by=clip caps=12.854 drawn face="Noto Sans" layout=10.3,24.52 limit=325,55,44,44 owner=17 paints=#D4D4D4FF role=none size=18 text=7
        19 Button at=375,55,44,44 by=panel drawn heard=8 limit=0,0,543,345 paints=#252526FF,#6E6E6EFF,#D4D4D4FF role=button text=8
          20 AccessText at=392,64,11,25 box=11,25 by=clip caps=12.854 drawn face="Noto Sans" layout=10.3,24.52 limit=375,55,44,44 owner=19 paints=#D4D4D4FF role=none size=18 text=8
        21 Button at=425,55,44,44 by=panel drawn heard=9 limit=0,0,543,345 paints=#252526FF,#6E6E6EFF,#D4D4D4FF role=button text=9
          22 AccessText at=442,64,11,25 box=11,25 by=clip caps=12.854 drawn face="Noto Sans" layout=10.3,24.52 limit=425,55,44,44 owner=21 paints=#D4D4D4FF role=none size=18 text=9
        23 Button at=475,55,44,44 by=panel drawn heard=0 limit=0,0,543,345 paints=#252526FF,#6E6E6EFF,#D4D4D4FF role=button text=0
          24 AccessText at=492,64,11,25 box=11,25 by=clip caps=12.854 drawn face="Noto Sans" layout=10.3,24.52 limit=475,55,44,44 owner=23 paints=#D4D4D4FF role=none size=18 text=0
      25 StackPanel at=25,105,494,44 by=panel limit=0,0,543,345 role=none
        26 Button at=25,105,44,44 by=panel drawn heard=q limit=0,0,543,345 paints=#252526FF,#6E6E6EFF,#D4D4D4FF role=button text=q
          27 AccessText at=41,114,12,25 box=12,25 by=clip caps=12.854 drawn face="Noto Sans" layout=11.07,24.52 limit=25,105,44,44 owner=26 paints=#D4D4D4FF role=none size=18 text=q
        28 Button at=75,105,44,44 by=panel drawn heard=w limit=0,0,543,345 paints=#252526FF,#6E6E6EFF,#D4D4D4FF role=button text=w
          29 AccessText at=90,114,15,25 box=15,25 by=clip caps=12.854 drawn face="Noto Sans" layout=14.15,24.52 limit=75,105,44,44 owner=28 paints=#D4D4D4FF role=none size=18 text=w
        30 Button at=125,105,44,44 by=panel drawn heard=e limit=0,0,543,345 paints=#252526FF,#6E6E6EFF,#D4D4D4FF role=button text=e
          31 AccessText at=142,114,11,25 box=11,25 by=clip caps=12.854 drawn face="Noto Sans" layout=10.15,24.52 limit=125,105,44,44 owner=30 paints=#D4D4D4FF role=none size=18 text=e
        32 Button at=175,105,44,44 by=panel drawn heard=r limit=0,0,543,345 paints=#252526FF,#6E6E6EFF,#D4D4D4FF role=button text=r
          33 AccessText at=193,114,8,25 box=8,25 by=clip caps=12.854 drawn face="Noto Sans" layout=7.43,24.52 limit=175,105,44,44 owner=32 paints=#D4D4D4FF role=none size=18 text=r
        34 Button at=225,105,44,44 by=panel drawn heard=t limit=0,0,543,345 paints=#252526FF,#6E6E6EFF,#D4D4D4FF role=button text=t
          35 AccessText at=244,114,7,25 box=7,25 by=clip caps=12.854 drawn face="Noto Sans" layout=6.5,24.52 limit=225,105,44,44 owner=34 paints=#D4D4D4FF role=none size=18 text=t
        36 Button at=275,105,44,44 by=panel drawn heard=y limit=0,0,543,345 paints=#252526FF,#6E6E6EFF,#D4D4D4FF role=button text=y
          37 AccessText at=292,114,10,25 box=10,25 by=clip caps=12.854 drawn face="Noto Sans" layout=9.18,24.52 limit=275,105,44,44 owner=36 paints=#D4D4D4FF role=none size=18 text=y
        38 Button at=325,105,44,44 by=panel drawn heard=u limit=0,0,543,345 paints=#252526FF,#6E6E6EFF,#D4D4D4FF role=button text=u
          39 AccessText at=341,114,12,25 box=12,25 by=clip caps=12.854 drawn face="Noto Sans" layout=11.12,24.52 limit=325,105,44,44 owner=38 paints=#D4D4D4FF role=none size=18 text=u
        40 Button at=375,105,44,44 by=panel drawn heard=i limit=0,0,543,345 paints=#252526FF,#6E6E6EFF,#D4D4D4FF role=button text=i
          41 AccessText at=394,114,5,25 box=5,25 by=clip caps=12.854 drawn face="Noto Sans" layout=4.64,24.52 limit=375,105,44,44 owner=40 paints=#D4D4D4FF role=none size=18 text=i
        42 Button at=425,105,44,44 by=panel drawn heard=o limit=0,0,543,345 paints=#252526FF,#6E6E6EFF,#D4D4D4FF role=button text=o
          43 AccessText at=442,114,11,25 box=11,25 by=clip caps=12.854 drawn face="Noto Sans" layout=10.89,24.52 limit=425,105,44,44 owner=42 paints=#D4D4D4FF role=none size=18 text=o
        44 Button at=475,105,44,44 by=panel drawn heard=p limit=0,0,543,345 paints=#252526FF,#6E6E6EFF,#D4D4D4FF role=button text=p
          45 AccessText at=491,114,12,25 box=12,25 by=clip caps=12.854 drawn face="Noto Sans" layout=11.07,24.52 limit=475,105,44,44 owner=44 paints=#D4D4D4FF role=none size=18 text=p
      46 StackPanel at=25,155,494,44 by=panel limit=0,0,543,345 role=none
        47 Button at=25,155,44,44 by=panel drawn heard=a limit=0,0,543,345 paints=#252526FF,#6E6E6EFF,#D4D4D4FF role=button text=a
          48 AccessText at=42,164,11,25 box=11,25 by=clip caps=12.854 drawn face="Noto Sans" layout=10.1,24.52 limit=25,155,44,44 owner=47 paints=#D4D4D4FF role=none size=18 text=a
        49 Button at=75,155,44,44 by=panel drawn heard=s limit=0,0,543,345 paints=#252526FF,#6E6E6EFF,#D4D4D4FF role=button text=s
          50 AccessText at=92,164,9,25 box=9,25 by=clip caps=12.854 drawn face="Noto Sans" layout=8.62,24.52 limit=75,155,44,44 owner=49 paints=#D4D4D4FF role=none size=18 text=s
        51 Button at=125,155,44,44 by=panel drawn heard=d limit=0,0,543,345 paints=#252526FF,#6E6E6EFF,#D4D4D4FF role=button text=d
          52 AccessText at=141,164,12,25 box=12,25 by=clip caps=12.854 drawn face="Noto Sans" layout=11.07,24.52 limit=125,155,44,44 owner=51 paints=#D4D4D4FF role=none size=18 text=d
        53 Button at=175,155,44,44 by=panel drawn heard=f limit=0,0,543,345 paints=#252526FF,#6E6E6EFF,#D4D4D4FF role=button text=f
          54 AccessText at=194,164,7,25 box=7,25 by=clip caps=12.854 drawn face="Noto Sans" layout=6.19,24.52 limit=175,155,44,44 owner=53 paints=#D4D4D4FF role=none size=18 text=f
        55 Button at=225,155,44,44 by=panel drawn heard=g limit=0,0,543,345 paints=#252526FF,#6E6E6EFF,#D4D4D4FF role=button text=g
          56 AccessText at=241,164,12,25 box=12,25 by=clip caps=12.854 drawn face="Noto Sans" layout=11.07,24.52 limit=225,155,44,44 owner=55 paints=#D4D4D4FF role=none size=18 text=g
        57 Button at=275,155,44,44 by=panel drawn heard=h limit=0,0,543,345 paints=#252526FF,#6E6E6EFF,#D4D4D4FF role=button text=h
          58 AccessText at=291,164,12,25 box=12,25 by=clip caps=12.854 drawn face="Noto Sans" layout=11.12,24.52 limit=275,155,44,44 owner=57 paints=#D4D4D4FF role=none size=18 text=h
        59 Button at=325,155,44,44 by=panel drawn heard=j limit=0,0,543,345 paints=#252526FF,#6E6E6EFF,#D4D4D4FF role=button text=j
          60 AccessText at=344,164,5,25 box=5,25 by=clip caps=12.854 drawn face="Noto Sans" layout=4.64,24.52 limit=325,155,44,44 owner=59 paints=#D4D4D4FF role=none size=18 text=j
        61 Button at=375,155,44,44 by=panel drawn heard=k limit=0,0,543,345 paints=#252526FF,#6E6E6EFF,#D4D4D4FF role=button text=k
          62 AccessText at=392,164,10,25 box=10,25 by=clip caps=12.854 drawn face="Noto Sans" layout=9.61,24.52 limit=375,155,44,44 owner=61 paints=#D4D4D4FF role=none size=18 text=k
        63 Button at=425,155,44,44 by=panel drawn heard=l limit=0,0,543,345 paints=#252526FF,#6E6E6EFF,#D4D4D4FF role=button text=l
          64 AccessText at=444,164,5,25 box=5,25 by=clip caps=12.854 drawn face="Noto Sans" layout=4.64,24.52 limit=425,155,44,44 owner=63 paints=#D4D4D4FF role=none size=18 text=l
        65 Button at=475,155,44,44 by=panel drawn heard=' limit=0,0,543,345 paints=#252526FF,#6E6E6EFF,#D4D4D4FF role=button text='
          66 AccessText at=494,164,5,25 box=5,25 by=clip caps=12.854 drawn face="Noto Sans" layout=4.05,24.52 limit=475,155,44,44 owner=65 paints=#D4D4D4FF role=none size=18 text='
      67 StackPanel at=25,205,494,44 by=panel limit=0,0,543,345 role=none
        68 Button at=25,205,94,44 by=panel drawn heard=Shift limit=0,0,543,345 paints=#252526FF,#6E6E6EFF,#D4D4D4FF role=button text=Shift
          69 AccessText at=52,214,39,25 box=39,25 by=clip caps=12.854 drawn face="Noto Sans" layout=38.34,24.52 limit=25,205,94,44 owner=68 paints=#D4D4D4FF role=none size=18 text=Shift
        70 Button at=125,205,44,44 by=panel drawn heard=z limit=0,0,543,345 paints=#252526FF,#6E6E6EFF,#D4D4D4FF role=button text=z
          71 AccessText at=142,214,9,25 box=9,25 by=clip caps=12.854 drawn face="Noto Sans" layout=8.46,24.52 limit=125,205,44,44 owner=70 paints=#D4D4D4FF role=none size=18 text=z
        72 Button at=175,205,44,44 by=panel drawn heard=x limit=0,0,543,345 paints=#252526FF,#6E6E6EFF,#D4D4D4FF role=button text=x
          73 AccessText at=192,214,10,25 box=10,25 by=clip caps=12.854 drawn face="Noto Sans" layout=9.52,24.52 limit=175,205,44,44 owner=72 paints=#D4D4D4FF role=none size=18 text=x
        74 Button at=225,205,44,44 by=panel drawn heard=c limit=0,0,543,345 paints=#252526FF,#6E6E6EFF,#D4D4D4FF role=button text=c
          75 AccessText at=242,214,9,25 box=9,25 by=clip caps=12.854 drawn face="Noto Sans" layout=8.64,24.52 limit=225,205,44,44 owner=74 paints=#D4D4D4FF role=none size=18 text=c
        76 Button at=275,205,44,44 by=panel drawn heard=v limit=0,0,543,345 paints=#252526FF,#6E6E6EFF,#D4D4D4FF role=button text=v
          77 AccessText at=292,214,10,25 box=10,25 by=clip caps=12.854 drawn face="Noto Sans" layout=9.14,24.52 limit=275,205,44,44 owner=76 paints=#D4D4D4FF role=none size=18 text=v
        78 Button at=325,205,44,44 by=panel drawn heard=b limit=0,0,543,345 paints=#252526FF,#6E6E6EFF,#D4D4D4FF role=button text=b
          79 AccessText at=341,214,12,25 box=12,25 by=clip caps=12.854 drawn face="Noto Sans" layout=11.07,24.52 limit=325,205,44,44 owner=78 paints=#D4D4D4FF role=none size=18 text=b
        80 Button at=375,205,44,44 by=panel drawn heard=n limit=0,0,543,345 paints=#252526FF,#6E6E6EFF,#D4D4D4FF role=button text=n
          81 AccessText at=391,214,12,25 box=12,25 by=clip caps=12.854 drawn face="Noto Sans" layout=11.12,24.52 limit=375,205,44,44 owner=80 paints=#D4D4D4FF role=none size=18 text=n
        82 Button at=425,205,44,44 by=panel drawn heard=m limit=0,0,543,345 paints=#252526FF,#6E6E6EFF,#D4D4D4FF role=button text=m
          83 AccessText at=438,214,17,25 box=17,25 by=clip caps=12.854 drawn face="Noto Sans" layout=16.83,24.52 limit=425,205,44,44 owner=82 paints=#D4D4D4FF role=none size=18 text=m
        84 Button at=475,205,44,44 by=panel drawn heard=, limit=0,0,543,345 paints=#252526FF,#6E6E6EFF,#D4D4D4FF role=button text=,
          85 AccessText at=494,214,5,25 box=5,25 by=clip caps=12.854 drawn face="Noto Sans" layout=4.82,24.52 limit=475,205,44,44 owner=84 paints=#D4D4D4FF role=none size=18 text=,
      86 StackPanel at=25,255,494,44 by=panel limit=0,0,543,345 role=none
        87 Button at=25,255,94,44 by=panel drawn heard=Code limit=0,0,543,345 paints=#252526FF,#6E6E6EFF,#D4D4D4FF role=button text=Code
          88 AccessText at=50,264,44,25 box=44,25 by=clip caps=12.854 drawn face="Noto Sans" layout=43.49,24.52 limit=25,255,94,44 owner=87 paints=#D4D4D4FF role=none size=18 text=Code
        89 Button at=125,255,44,44 by=panel drawn heard=. limit=0,0,543,345 paints=#252526FF,#6E6E6EFF,#D4D4D4FF role=button text=.
          90 AccessText at=144,264,5,25 box=5,25 by=clip caps=12.854 drawn face="Noto Sans" layout=4.82,24.52 limit=125,255,44,44 owner=89 paints=#D4D4D4FF role=none size=18 text=.
        91 Button at=175,255,44,44 by=panel drawn heard=- limit=0,0,543,345 paints=#252526FF,#6E6E6EFF,#D4D4D4FF role=button text=-
          92 AccessText at=194,264,6,25 box=6,25 by=clip caps=12.854 drawn face="Noto Sans" layout=5.8,24.52 limit=175,255,44,44 owner=91 paints=#D4D4D4FF role=none size=18 text=-
        93 Button at=225,255,94,44 by=panel drawn heard=Space limit=0,0,543,345 paints=#252526FF,#6E6E6EFF,#D4D4D4FF role=button text=Space
          94 AccessText at=247,264,50,25 box=50,25 by=clip caps=12.854 drawn face="Noto Sans" layout=49.84,24.52 limit=225,255,94,44 owner=93 paints=#D4D4D4FF role=none size=18 text=Space
        95 Button at=325,255,94,44 by=panel drawn heard=Erase limit=0,0,543,345 paints=#252526FF,#6E6E6EFF,#D4D4D4FF role=button text=Erase
          96 AccessText at=349,264,46,25 box=46,25 by=clip caps=12.854 drawn face="Noto Sans" layout=45.95,24.52 limit=325,255,94,44 owner=95 paints=#D4D4D4FF role=none size=18 text=Erase
        97 Button at=425,255,94,44 by=panel drawn heard=Done limit=0,0,543,345 paints=#252526FF,#6E6E6EFF,#D4D4D4FF role=button text=Done
          98 AccessText at=449,264,46,25 box=46,25 by=clip caps=12.854 drawn face="Noto Sans" layout=45.31,24.52 limit=425,255,94,44 owner=97 paints=#D4D4D4FF role=none size=18 text=Done
focus path:0/3/1/0/0/0
press back top=none
