NASM_FORMAT := $(shell ./nasm_format.sh)

.PHONY: all build run assemble link clean

all: build run assemble link

build:
	mkdir -p build

run:
	cargo run -- examples/test.nerv build/out.s

assemble:
	nasm -f $(NASM_FORMAT) build/out.s -o build/out.o

link:
	gcc build/out.o -o build/out
	rm build/out.o

clean:
	rm -f build/out build/out.o build/out.s
