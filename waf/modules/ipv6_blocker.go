// cebulka-waf/modules/ipv6_blocker.go
package modules

import (
	"net/http"
	"regexp"
)

var _IPv6Pattern = regexp.MustCompile(`(?i)(([0-9a-f]{1,4}:){7}[0-9a-f]{1,4}|::([0-9a-f]{1,4}:){0,6}[0-9a-f]{1,4}|([0-9a-f]{1,4}:){1,6}:[0-9a-f]{1,4}|([0-9a-f]{1,4}:){1,5}(:[0-9a-f]{1,4}){1,2}|([0-9a-f]{1,4}:){1,4}(:[0-9a-f]{1,4}){1,3}|([0-9a-f]{1,4}:){1,3}(:[0-9a-f]{1,4}){1,4}|([0-9a-f]{1,4}:){1,2}(:[0-9a-f]{1,4}){1,5}|[0-9a-f]{1,4}:((:[0-9a-f]{1,4}){1,6})|:((:[0-9a-f]{1,4}){1,7}|:))`)

func init() {
	RegisterModule(Module{
		Name:        "IPv6Blocker",
		Description: "Blocks IPv6 addresses in content",
		Version:     "1.0.1",
		Priority:    10,
		Modifier:    _BlockIPv6,
	})
}

func _BlockIPv6(Content []byte, _ http.Header) []byte {
	return _IPv6Pattern.ReplaceAll(Content, []byte("[REDACTED]"))
}
