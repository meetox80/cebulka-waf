// cebulka-waf/modules/response_size.go
package modules

import (
	logging "cebulka-waf/core"
	"fmt"
)

const (
	_MaxResponseBytes = 8 * 1024 * 1024 // 8MB
)

func init() {
	RegisterModule(Module{
		Name:        "ResponseSizeLimiter",
		Description: "Enforces maximum response size",
		Version:     "1.0.0",
		Modifier:    _LimitResponseSize,
	})
}

func _LimitResponseSize(Content []byte) []byte {
	if len(Content) > _MaxResponseBytes {
		logging.Log(logging.WarnStatus, "SizeLimiter", "Truncated "+formatBytes(len(Content)))
		return Content[:_MaxResponseBytes]
	}
	return Content
}

func formatBytes(Bytes int) string {
	const Unit = 1024
	if Bytes < Unit {
		return fmt.Sprintf("%dB", Bytes)
	}
	Div, Exp := int64(Unit), 0
	for n := Bytes / Unit; n >= Unit; n /= Unit {
		Div *= Unit
		Exp++
	}
	return fmt.Sprintf("%.1f%cB", float64(Bytes)/float64(Div), "KMGTPE"[Exp])
}
