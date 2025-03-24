// cebulka-waf/modules/brotli.go
package modules

import (
	"bytes"
	"net/http"
	"strings"

	"github.com/andybalholm/brotli"
)

func init() {
	RegisterModule(Module{
		Name:        "BrotliCompressor",
		Description: "Compresses responses with Brotli",
		Version:     "1.0.0",
		Priority:    1000,
		Modifier:    BrotliModifier,
	})
}

func BrotliModifier(Content []byte, Headers http.Header) []byte {
	if Headers.Get("Content-Encoding") != "" {
		return Content
	}

	ContentType := Headers.Get("Content-Type")
	if !IsCompressible(ContentType) {
		return Content
	}

	var Buffer bytes.Buffer
	Writer := brotli.NewWriterLevel(&Buffer, brotli.DefaultCompression)
	defer Writer.Close()

	if _, Err := Writer.Write(Content); Err != nil {
		return Content
	}
	if Err := Writer.Flush(); Err != nil {
		return Content
	}

	Headers.Set("Content-Encoding", "br")
	Headers.Add("Vary", "Accept-Encoding")
	return Buffer.Bytes()
}

func IsCompressible(ContentType string) bool {
	switch {
	case strings.HasPrefix(ContentType, "text/"):
		return true
	case ContentType == "application/json",
		ContentType == "application/javascript",
		ContentType == "application/xml",
		ContentType == "image/svg+xml":
		return true
	default:
		return false
	}
}
