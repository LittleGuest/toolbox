# Maintainer: LittleGuest <2190975784@qq.com>
pkgname=toolbox
pkgver=1.0.0
pkgrel=1
pkgdesc="离线多功能工具箱，基于 GPUI 和 Rust 开发"
arch=('x86_64')
url="https://github.com/LittleGuest/toolbox"
license=('MIT')
options=('!strip' '!debug')
_source_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
_bin="${_source_dir}/target/release/toolbox"
source=(
  "toolbox::file://${_bin}"
  "toolbox.desktop::file://${_source_dir}/toolbox.desktop"
  "toolbox.svg::file://${_source_dir}/icons/toolbox.svg"
  "LICENSE::file://${_source_dir}/LICENSE"
  "README.md::file://${_source_dir}/README.md"
)
sha256sums=(
  'SKIP'
  'SKIP'
  'SKIP'
  'SKIP'
  'SKIP'
)

package() {
  install -Dm755 "${srcdir}/toolbox" "${pkgdir}/usr/bin/toolbox"

  install -Dm644 "${srcdir}/toolbox.desktop" \
    "${pkgdir}/usr/share/applications/toolbox.desktop"

  install -Dm644 "${srcdir}/toolbox.svg" \
    "${pkgdir}/usr/share/icons/hicolor/scalable/apps/toolbox.svg"

  install -Dm644 "${srcdir}/LICENSE" \
    "${pkgdir}/usr/share/licenses/${pkgname}/LICENSE"

  install -Dm644 "${srcdir}/README.md" \
    "${pkgdir}/usr/share/doc/${pkgname}/README.md"
}
