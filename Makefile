.PHONY: all sync build cli-build cli-install cli-test python-test test e2e-smoke clean upgrade-superpowers

CLI_BIN := cli/target/release/superpowers-trae

all: sync build cli-build

sync:
	bash scripts/sync-upstream.sh

build:
	bash scripts/build.sh

cli-build:
	cd cli && cargo build --release

cli-install:
	cd cli && cargo install --path .

cli-test:
	cd cli && cargo test

python-test:
	python3 -m unittest discover tests

test: python-test cli-test

e2e-smoke: cli-build
	@TMP=$$(mktemp -d) && \
	$(CLI_BIN) init --dir "$$TMP" --addons ddd && \
	$(CLI_BIN) status --dir "$$TMP" && \
	$(CLI_BIN) upgrade --dir "$$TMP" --addons ddd && \
	$(CLI_BIN) status --dir "$$TMP" && \
	test -f "$$TMP/DOMAIN.md" && \
	test -f "$$TMP/.trae/skills/ddd/ddd-storm/SKILL.md" && \
	test -f "$$TMP/README-DDD-HARNESS.md" && \
	rm -rf "$$TMP" && \
	echo "✓ e2e smoke passed (with ddd)"

upgrade-superpowers: sync build cli-build
	@echo "✓ Maintainer pipeline done. Review with: git diff dist/ && git diff cli/"

clean:
	rm -rf dist/
	cd cli && cargo clean
