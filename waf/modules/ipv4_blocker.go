// cebulka-waf/modules/ipv4_blocker.go
package modules

import (
	"net/http"
	"regexp"
)

var _IPv4Pattern = regexp.MustCompile(`(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)`)

func init() {
	RegisterModule(Module{
		Name:        "IPv4Blocker",
		Description: "Blocks IPv4 addresses in content",
		Version:     "1.0.1",
		Priority:    10,
		Modifier:    BlockIPv4,
	})
}

func BlockIPv4(Content []byte, _ http.Header) []byte {
	return _IPv4Pattern.ReplaceAll(Content, []byte("[REDACTED]"))
}
