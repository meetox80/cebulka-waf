// cebulka-waf/modules/response_size.go
package modules

import (
	"net/http"
)

const _MaxResponseSize = 1024 * 1024 * 8 // 8MB

func init() {
	RegisterModule(Module{
		Name:        "ResponseSizeLimiter",
		Description: "Limits response size to 5MB",
		Version:     "1.0.1",
		Priority:    20,
		Modifier:    _LimitResponseSize,
	})
}

func _LimitResponseSize(Content []byte, _ http.Header) []byte {
	if len(Content) > _MaxResponseSize {
		return Content[:_MaxResponseSize]
	}
	return Content
}
