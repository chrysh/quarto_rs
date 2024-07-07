# SPDX-License-Identifier: GPL-2.0

# obj-$(CONFIG_SAMPLE_RUST_QUARTO)		+= rust_quarto.o
obj-m += rust_quarto.o
KERNEL_SRC := ~/code/kernel/net-next
SRC := $(shell pwd)

# Don't forget the following:
#export PATH="$(HOME)/.cargo/bin:$(PATH)"
#export LIBCLANG_PATH=/usr/lib/llvm-16/lib
#export LLVM=1

all:
	$(MAKE) -C $(KERNEL_SRC) M=$(SRC) modules

modules_install:
	$(MAKE) -C $(KERNEL_SRC) M=$(SRC) modules_install

