pkgname=meowsn
pkgver=0.10.2
pkgrel=2
pkgdesc="Cross-platform MSNP11 client"
arch=('aarch64' 'x86_64')
url="https://github.com/campos02/meowsn"
license=()
makedepends=('rust')
depends=()
source=('git+https://github.com/RandomHuman2020/meowsn.git')
sha256sums=('SKIP')

prepare() {
  export RUSTUP_TOOLCHAIN=stable
  cd meowsn
  cargo fetch --manifest-path Cargo.toml --locked --target host-tuple --verbose
}

build() {
  export RUSTUP_TOOLCHAIN=stable
  export CARGO_TARGET_DIR=target
  cd meowsn
  cargo build --frozen --release
}

check() {
  export RUSTUP_TOOLCHAIN=stable
  cd meowsn
  cargo test --frozen
}

package() {
  install -Dm0755 -t "${pkgdir}/usr/bin/" "meowsn/target/release/${pkgname}"
  install -Dm0644 "meowsn/LICENSE" "${pkgdir}/usr/share/licenses/${pkgname}/LICENSE"
  install -Dm0644 "meowsn/assets/meowsn.svg" "${pkgdir}/usr/share/icons/hicolor/scalable/apps/${pkgname}.svg"
  install -Dm0644 "meowsn/assets/meowsn.desktop" "${pkgdir}/usr/share/applications/${pkgname}.desktop"
  install -Dm0644 "meowsn/assets/meowsn.metainfo.xml" "${pkgdir}/usr/share/metainfo/${pkgname}.metainfo.xml"
}
