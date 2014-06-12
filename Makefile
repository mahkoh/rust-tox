FLAGS = -O

TOX = $(shell rustc --crate-file-name lib.rs)

all: $(TOX)

-include .tox.d

$(TOX):
	rustc $(FLAGS) lib.rs

test: $(TOX) test.rs
	rustc $(FLAGS) -L. test.rs

version:
	rustc --no-trans --dep-info .tox.d lib.rs

clean:
	rm -f $(TOX) test

.PHONY: version clean
