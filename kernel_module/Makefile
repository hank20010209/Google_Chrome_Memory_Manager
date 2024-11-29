obj-m += get_mem.o
PWD := $(CURDIR) 

all: 
	make -C /lib/modules/$(shell uname -r)/build M=$(PWD) modules 
	sudo insmod get_mem.ko || echo "Kernel module get_mem could not be inserted"
clean: 
	make -C /lib/modules/$(shell uname -r)/build M=$(PWD) clean
	sudo rmmod get_mem || echo "Failed to remove kernel module get_mem"
