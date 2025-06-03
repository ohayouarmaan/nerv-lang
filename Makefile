NASM_FORMAT := $(shell ./nasm_format.sh)

all: build/out

build/out.s:
	cargo run -- examples/test.nerv build/out.s

build/out.o: build/out.s
	nasm -f $(NASM_FORMAT) build/out.s -o build/out.o

build/out: build/out.o
	gcc build/out.o -o build/out
	rm build/out.o

clean:
	rm -f build/out build/out.o
