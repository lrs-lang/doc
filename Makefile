musl := "/home/julian/c/musl"

.PHONY: all clean lib

all: lrs_doc

srcs := $(shell find -name "*.rs")

lrs_doc: main.rs $(srcs) ../lib/obj/liblrs.rlib Makefile
	@# rustc -O -C lto -C no-stack-check -Z no-landing-pads -L ../lib/obj -C link-args="-nostdlib $(musl)/lib/crt1.o -L $(musl)/lib -static -l c -l pthread" "$<" -o "$@"
	@# rustc -O -C lto -C no-stack-check -Z no-landing-pads -L ../lib/obj "$<" -o "$@"
	rustc -g -C no-stack-check -Z no-landing-pads -L ../lib/obj "$<" -o "$@"

clean:
	rm bin/*