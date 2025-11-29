FROM rust:1.91-slim-trixie

# RUN - https://docs.docker.jp/engine/reference/builder.html#run
# パッケージの更新とインストール
# apt-get update                  更新を取得
# apt-get install -y <packages>   パッケージをインストール ここで必要なパッケージを指定
# apt-get clean                   aptキャッシュをクリーン
# rm -rf /var/lib/apt/lists/*     キャッシュされている全パッケージリストを削除
# https://docs.docker.jp/engine/articles/dockerfile_best-practice.html
RUN apt-get update; \
    apt-get install -y \
        git \
        pkg-config \
        libssl-dev \
        mingw-w64 \
        # eframe用ライブラリ
        libxcb-render0-dev \
        libxcb-shape0-dev \
        libxcb-xfixes0-dev \
        libxkbcommon-dev \
        libgtk-3-dev \
        libwayland-dev \
        # --- 追加: 日本語フォント ---
        fonts-noto-cjk \
        ; \
    apt-get clean; \
    rm -rf /var/lib/apt/lists/*;

RUN rustup target add x86_64-pc-windows-gnu; \
    rustup component add rustc rust-analysis rust-src rustfmt clippy; \
    cargo install cargo-edit cargo-watch cargo-update;
