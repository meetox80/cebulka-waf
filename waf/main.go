// waf/main.go
package main

import (
	"bytes"
	logging "cebulka-waf/core"
	"cebulka-waf/modules"
	"io"
	"log"
	"net/http"
	"net/http/httputil"
)

const (
	_ProxyEndpoint  = "127.0.0.1:1337"
	_BackendAddress = "127.0.0.1:2025"
	_HttpScheme     = "http"
)

type BufferCloser struct {
	*bytes.Buffer
}

func (BufferCloser) Close() error {
	return nil
}

func main() {
	logging.Log(logging.BuildStatus, "Bootstrapper", "Initializing WAF proxy")

	Proxy := &httputil.ReverseProxy{
		Director: func(Req *http.Request) {
			logging.Log(logging.DebugStatus, "RequestDirector", "Proxying "+Req.Method+" "+Req.URL.Path)
			Req.URL.Scheme = _HttpScheme
			Req.URL.Host = _BackendAddress
			Req.Host = _BackendAddress
		},
		ModifyResponse: ModifyResponse,
	}

	logging.Log(logging.OkStatus, "Runtime", "Serving on "+_ProxyEndpoint)
	Err := http.ListenAndServe(_ProxyEndpoint, Proxy)
	if Err != nil {
		logging.Log(logging.ErrStatus, "FatalError", Err.Error())
		log.Fatal(Err)
	}
}

func ModifyResponse(Res *http.Response) error {
	if Res.Body == nil || Res.Body == http.NoBody {
		logging.Log(logging.DebugStatus, "ResponseHandler", "Skipping empty body")
		return nil
	}

	defer Res.Body.Close()
	BodyContent, Err := io.ReadAll(Res.Body)
	if Err != nil {
		logging.Log(logging.ErrStatus, "BodyReader", "Failed: "+Err.Error())
		return Err
	}

	ModifiedContent := modules.ApplyModules(BodyContent, http.Header{})

	Res.Body = BufferCloser{bytes.NewBuffer(ModifiedContent)}
	Res.ContentLength = int64(len(ModifiedContent))

	if len(ModifiedContent) == 0 {
		Res.StatusCode = http.StatusNoContent
		Res.Status = http.StatusText(http.StatusNoContent)
		logging.Log(logging.WarnStatus, "ContentSanitizer", "Generated empty response")
	}

	Res.Header.Del("Content-Length")
	Res.Header.Del("Content-Encoding")
	return nil
}
