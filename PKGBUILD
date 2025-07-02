# Maintainer: Mimikyudev <mail@mimikyu.dev>
# Contributor: Luan T. <original author>
# Contributor: Gemini <rust rewrite>

pkgname=pokescript-git
_pkgname=pokescript
pkgver() {
    cd "${_pkgname}"
    printf "r%s.g%s" "$(git rev-list --count HEAD)" "$(git rev-parse --short HEAD)"
}
pkgrel=1
pkgdesc="A CLI utility to print out unicode images of pokemon in your shell (Rust version)"
arch=('x86_64' 'aarch64')
url="https://github.com/YOUR_USERNAME/pokemon-colorscripts-rs"
license=('MIT')
depends=()
makedepends=('cargo' 'git')
provides=("${_pkgname}")
conflicts=("${_pkgname}" "pokemon-colorscripts-git")
source=("${_pkgname}::git+${url}.git")
sha256sums=('SKIP')

build() {
    cd "${_pkgname}"
    cargo build --release --locked
}

check() {
    cd "${_pkgname}"
    cargo test --locked
}

package() {
    cd "${_pkgname}"
    install -Dm755 "target/release/${_pkgname}" -t "${pkgdir}/usr/bin/"
    install -Dm644 "LICENSE.txt" -t "${pkgdir}/usr/share/licenses/${pkgname}/"
    install -Dm644 "pokescript.1" -t "${pkgdir}/usr/share/man/man1/"
}
