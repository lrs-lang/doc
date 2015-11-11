.PHONY: all clean

-include config.mk

all: lrs_doc

-include lrs_doc.d

lrs_doc:
	lrsc $(ops) --emit=link,dep-info src/main.rs

clean:
	rm -f lrs_doc
