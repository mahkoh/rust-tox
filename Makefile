
FLAGS = -O
DEPS_FILE = target/.tox.deps
TOX_SRC = src/tox.rs
TOX = target/$(shell rustc --print-file-name $(TOX_SRC))
TOX_DEPS = $(shell head -n1 $(DEPS_FILE) 2> /dev/null)
EXAMPLES_SRC = $(wildcard src/bin/*.rs)
EXAMPLES_BIN = $(EXAMPLES_SRC:src/bin/%.rs=target/%)

all: $(TOX) $(EXAMPLES_BIN)

lib: $(TOX)

$(TOX): $(TOX_DEPS)
	@mkdir -p target
	rustc $(FLAGS) --out-dir target $(TOX_SRC)
	@rustc --no-trans --dep-info $(DEPS_FILE) $(TOX_SRC)
	@sed -i 's/.*: //' $(DEPS_FILE)

$(EXAMPLES_BIN): target/%: src/bin/%.rs $(TOX)
	rustc --out-dir target -L target $<

docs:
	rm -rf doc
	rustdoc $(TOX_SRC)

clean:
	rm -rf doc
	rm -rf target

.PHONY: all clean docs lib
