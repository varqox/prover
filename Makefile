CARGO = $(shell which cargo > /dev/null 2> /dev/null && echo cargo || echo '$$HOME/.cargo/bin/cargo')

FO-prover: setup_cargo
	$(CARGO) build --release && cp target/release/prover $@

setup_cargo:
	@which cargo > /dev/null 2> /dev/null || curl https://sh.rustup.rs -sSf | sh -s -- '-y'

clean:
	$(CARGO) clean
	rm -f FO-prover
