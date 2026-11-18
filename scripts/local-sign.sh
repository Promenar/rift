#!/bin/sh
# 用本机可用的代码签名身份签署 Rift 二进制。
#
# 背景：ad-hoc 签名的 designated requirement 是 cdhash，重新构建就会变化，
# macOS 辅助功能授权随之失效，每次迭代都要手动重新授权。改用证书签名后，
# 授权要求变成「标识符 + 证书」，同一路径重建可以继承授权。
#
# 用法：
#   scripts/local-sign.sh path/to/rift path/to/rift-cli
#   RIFT_SIGN_IDENTITY="Apple Development: ..." scripts/local-sign.sh ...
set -eu

identity="${RIFT_SIGN_IDENTITY:-}"
if [ -z "$identity" ]; then
	identity=$(security find-identity -v -p codesigning |
		sed -n 's/^[[:space:]]*[0-9]*) [0-9A-F]* "\(.*\)"$/\1/p' | head -n 1)
fi
if [ -z "$identity" ]; then
	echo "local-sign: 未找到可用的代码签名身份；设置 RIFT_SIGN_IDENTITY 或安装开发证书" >&2
	exit 1
fi

[ "$#" -gt 0 ] || {
	echo "local-sign: 需要至少一个二进制路径" >&2
	exit 1
}

for bin in "$@"; do
	[ -f "$bin" ] || {
		echo "local-sign: 文件不存在: $bin" >&2
		exit 1
	}
	case "$(basename "$bin")" in
	rift-cli) identifier=git.acsandmann.rift.cli ;;
	*) identifier=git.acsandmann.rift ;;
	esac
	codesign --force --sign "$identity" --identifier "$identifier" --timestamp=none "$bin"
done
