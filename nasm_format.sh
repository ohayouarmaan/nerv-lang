#!/bin/sh
UNAME_S="$(uname -s)"
UNAME_M="$(uname -m)"

if [ "$UNAME_S" = "Darwin" ]; then
    echo "macho64"
elif [ "$UNAME_S" = "Linux" ]; then
    if [ "$UNAME_M" = "x86_64" ]; then
        echo "elf64"
    else
        echo "elf32"
    fi
else
    echo "elf64" # default fallback
fi
