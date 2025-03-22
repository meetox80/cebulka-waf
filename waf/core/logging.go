// cebulka-waf/core/logging.go
package logging

import (
	"fmt"
	"time"
)

type Status string

const (
	InfoStatus  Status = "INFO"
	WarnStatus  Status = "WARN"
	OkStatus    Status = "OK"
	ErrStatus   Status = "ERR"
	BuildStatus Status = "BUILD"
	DebugStatus Status = "DEBUG"
)

const (
	_TimeLayout = "15:04:05"
)

func Log(Status Status, Title string, Message string) {
	Timestamp := time.Now().Format(_TimeLayout)
	fmt.Printf("[%s] [%-5s] [%-15s]: %s\n", Timestamp, Status, Title, Message)
}
