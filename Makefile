all: kernel_module_build cargo_run

kernel_module_build:
	@echo "Building kernel module..."
	@$(MAKE) -C kernel_module

cargo_run:
	@echo "Running Rust application..."
	@cd $(shell pwd)/memory_management_for_chrome && cargo run

clean:
	@echo "Cleaning kernel module..."
	@$(MAKE) -C kernel_module clean
	@echo "Cleaning Rust build..."
	@cargo clean

grafana:
	sudo systemctl restart grafana-server
