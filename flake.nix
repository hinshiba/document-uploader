{
	description = "flake for document-uploader";

	inputs = {
		nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
		rust-overlay = {
			url = "github:oxalica/rust-overlay";
			inputs.nixpkgs.follows = "nixpkgs";
		};
	};

	outputs = {
		self,
		nixpkgs,
		rust-overlay,
		...
	}:
	let
		system = "x86_64-linux";
		overlays = [ (import rust-overlay) ];
		pkgs = import nixpkgs {
			inherit system overlays;
		};
	in
	{
		devShells.${system} = rec {
			default = pkgs.mkShell {
				inputsFrom = [ frontend backend ];
			};
			backend = pkgs.mkShell {
				packages = [
					(pkgs.rust-bin.stable.latest.default.override {
						extensions = [ "rust-src" "rust-analyzer" ];
					})
					pkgs.sqlx-cli
				];
			};
			frontend = pkgs.mkShell {
				packages = [
					pkgs.bun
					pkgs.tera-cli
					pkgs.superhtml
				];
				shellHook = ''
					# Run safe-chain bash initialisation script
					if [ -d "./.local/safe-chain" ]; then
						source ./.local/safe-chain/scripts/init-posix.sh
					fi
				'';
			};
		};
	};
}
