obj-m += get_mem.o
PWD := $(CURDIR) 

all: 
	make -C /lib/modules/$(shell uname -r)/build M=$(PWD) modules 
	sudo insmod get_mem.ko
clean: 
	make -C /lib/modules/$(shell uname -r)/build M=$(PWD) clean