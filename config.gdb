target remote | openocd -c "source [find board/ek-tm4c123gxl.cfg]" -c "gdb_port pipe; log_output target/openocd.log"

# print demangled symbols
set print asm-demangle on

# detect unhandled exceptions, hard faults and panics
break DefaultHandler
break UserHardFault
break rust_begin_unwind

# *try* to stop at the user entry point (it might be gone due to inlining)
break main

monitor arm semihosting enable

load

# start the process
stepi
