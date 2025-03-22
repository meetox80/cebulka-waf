// cebulka-waf/modules/ipv4_block.go
package modules

import "regexp"

var _IPv4Pattern = regexp.MustCompile(`(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)`)

func init() {
	RegisterModule(Module{
		Name:        "IPv4Blocker",
		Description: "Blocks IPv4 addresses in content",
		Version:     "1.0.0",
		Modifier:    _BlockIPv4,
	})
}

func _BlockIPv4(_Content []byte) []byte {
	return _IPv4Pattern.ReplaceAll(_Content, []byte("[REDACTED]"))
}
