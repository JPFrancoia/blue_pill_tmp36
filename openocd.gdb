target remote :3333
set print asm-demangle on
set print pretty on
monitor tpiu config external uart off 8000000 2000000
monitor itm port 0 on
load
break DefaultHandler
break UserHardFault
break main
continue
