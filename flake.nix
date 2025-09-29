{
  description = "Rust + Bevy development environment with VSCodium support";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # Rust toolchain с необходимыми компонентами для Bevy
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ 
            "rust-src" 
            "rust-analyzer" 
            "clippy" 
            "rustfmt" 
          ];
        };

        # Зависимости для Bevy (Linux)
        bevyDeps = with pkgs; [
          # Системные библиотеки для Bevy
          pkg-config
          udev
          alsa-lib
          vulkan-loader
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr
          libxkbcommon
          wayland
          
          # OpenGL поддержка
          libGL
          
          # Дополнительные зависимости
          fontconfig
          freetype
        ];

        # Инструменты разработки
        devTools = with pkgs; [
          # Редактор
          vscodium
          
          # Git и утилиты
          git
          just # Make-подобный инструмент
          
          # Отладка и профилирование
          gdb
          valgrind
          
          # Дополнительные инструменты
          cargo-watch    # Автоматическая пересборка
          cargo-edit     # Редактирование Cargo.toml
          cargo-audit    # Проверка безопасности
          cargo-outdated # Проверка устаревших зависимостей
        ];

      in
      {
        # Development shell
        devShells.default = pkgs.mkShell {
          buildInputs = [ rustToolchain ] ++ bevyDeps ++ devTools;
          
          # Переменные окружения
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath bevyDeps;
          
          # Переменные для Vulkan
          VULKAN_SDK = "${pkgs.vulkan-headers}";
          VK_LAYER_PATH = "${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d";
          
          # Включаем динамическую линковку для быстрой компиляции в dev режиме
          BEVY_DYNAMIC_LINKING = "1";
          
          shellHook = ''
            echo "🦀 Rust + Bevy development environment"
            echo "📝 VSCodium доступен через команду 'codium'"
            echo "🔧 Rust version: $(rustc --version)"
            echo "📦 Cargo version: $(cargo --version)"
            echo "🎯 rust-analyzer доступен для VSCode"
            echo ""
            echo "🚀 Полезные команды:"
            echo "  just dev      - запуск с динамической линковкой"
            echo "  just build    - сборка проекта"
            echo "  just test     - запуск тестов"
            echo "  just watch    - автоматическая пересборка"
            echo "  codium .      - открыть VSCodium"
            echo ""
            '';
        };

        # Сборка релизной версии
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "bevy-app";
          version = "0.1.0";
          src = ./.;
          
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = bevyDeps;
          
          # Отключаем динамическую линковку для релиза
          cargoBuildFlags = [ "--no-default-features" ];
          
          # Копируем assets если есть
          postInstall = ''
            if [ -d "assets" ]; then
              cp -r assets $out/bin/
            fi
          '';
          
          meta = with pkgs.lib; {
            description = "Bevy game developed with Nix";
            license = licenses.mit;
          };
        };

        # Дополнительные пакеты
        packages.dev-container = pkgs.dockerTools.buildImage {
          name = "bevy-dev";
          tag = "latest";
          
          contents = [ self.devShells.${system}.default ];
          
          config = {
            Cmd = [ "${pkgs.bash}/bin/bash" ];
            WorkingDir = "/workspace";
          };
        };
      }
    );
}
