.PHONY: all, release, windows

all:
	WINIT_UNIX_BACKEND=x11 LIBGL_ALWAYS_SOFTWARE=1 cargo run

release:
	cargo build --release

windows:
	cargo build --release --target x86_64-pc-windows-gnu
